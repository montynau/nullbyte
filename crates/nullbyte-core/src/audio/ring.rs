//! Lock-free ring buferis garso sample'ams tarp emuliavimo gijos (producer) ir cpal audio
//! callback'o (consumer) — P3.2, CLAUDE.md §3.2 taisyklė #3, §8.6.
//!
//! Naudoja `rtrb::RingBuffer<i16>` (SPSC, lock-free, be alokacijos push/pop metu). `i16`
//! pasirinktas TIESIOGIAI atitinka libretro `retro_audio_sample_batch_t` interleaved
//! sample'ų formatą (žr. `core/callbacks.rs`) — jokio papildomo konvertavimo emuliavimo gijoje.
//!
//! **Underrun** (consumer greitesnis už producer'į, buferis tuščias): trūkstama dalis
//! užpildoma tyla. Real-time audio callback'e NEGALIMA kviesti `tracing`/alokuoti (CLAUDE.md
//! §3.2 taisyklė #3) — todėl čia tik ATOMIŠKAI didinamas skaitliukas
//! ([`AudioConsumer::underrun_count`]); throttled `tracing::warn!` log'inimas atliekamas IŠ
//! AUKŠČIAU, pvz. `core::runner`'io periodiniame 5s statistikos log'e (ta pati gija, kuri jau
//! logina video/audio kadrų statistiką — P1.7).
//!
//! **Overrun** (producer greitesnis už consumer'į, buferis pilnas): `rtrb::Producer` API
//! NETURI jokio būdo pašalinti jau įrašytus (dar neperskaitytus) sample'us — tik consumer'is
//! (KITA gija!) gali juos paimti per `pop`/`read_chunk`. Todėl „mesk seniausius sample'us"
//! realizuojama PER ĮEINANTĮ chunk'ą: jei laisvos vietos nepakanka VISIEMS naujiems
//! sample'ams, paliekamas tik naujausias (chunk'o pabaigos) segmentas, kuris tilpsta —
//! senesnė (šio chunk'o pradžios) dalis išmetama. Tai išsaugo aktualiausią (naujausią) garso
//! informaciją A/V sinchronizacijai, neliečiant consumer'io pusės iš kitos gijos.

#![allow(dead_code)] // pilnai išnaudos P3.3/P3.4 — P3.2 metu naudoja tik testai.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rtrb::chunks::ChunkError;
use rtrb::RingBuffer;

/// Sukuria ring buffer porą. `capacity` — bendras sample'ų (NE kadrų, jau padaugintas iš
/// kanalų kiekio) skaičius; žr. [`recommended_capacity`] tinkamam dydžiui apskaičiuoti.
pub fn new(capacity: usize) -> (AudioProducer, AudioConsumer) {
    let (producer, consumer) = RingBuffer::<i16>::new(capacity);
    let underrun_count = Arc::new(AtomicU64::new(0));
    let overrun_count = Arc::new(AtomicU64::new(0));
    (
        AudioProducer {
            producer,
            overrun_count,
        },
        AudioConsumer {
            consumer,
            underrun_count,
        },
    )
}

/// Rekomenduojama talpa (sample'ų, jau padauginta iš kanalų) — ~4x tikslinio audio buferio
/// dydžio (P3.2 „Ką daryti"), kad [`AudioProducer::occupancy`] turėtų prasmingą diapazoną
/// dinaminiam rate control'ui (P3.4, CLAUDE.md §8.6).
pub fn recommended_capacity(sample_rate: u32, channels: u16, target_latency_ms: u32) -> usize {
    let frames_per_buffer = (u64::from(sample_rate) * u64::from(target_latency_ms) / 1000).max(1);
    ((frames_per_buffer as usize) * channels as usize * 4).max(channels as usize * 4)
}

/// Rašymo pusė — naudoja emuliavimo gija (CLAUDE.md §3.2). Ne `Clone`.
pub struct AudioProducer {
    producer: rtrb::Producer<i16>,
    overrun_count: Arc<AtomicU64>,
}

/// Skaitymo pusė — naudoja cpal audio callback. Ne `Clone`. `fill()` NIEKADA nealokuoja ir
/// nekviečia I/O (CLAUDE.md §3.2 taisyklė #3) — saugu real-time audio gijoje.
pub struct AudioConsumer {
    consumer: rtrb::Consumer<i16>,
    underrun_count: Arc<AtomicU64>,
}

impl AudioProducer {
    /// Įrašo `samples` į buferį. Jei laisvos vietos nepakanka visiems — paliekamas tik
    /// naujausias (galo) segmentas, kuris tilpsta; senesnė dalis išmetama (overrun, žr.
    /// modulio doc). Grąžina, kiek sample'ų realiai įrašyta.
    pub fn push_samples(&mut self, samples: &[i16]) -> usize {
        if samples.is_empty() {
            return 0;
        }
        match self.producer.write_chunk_uninit(samples.len()) {
            Ok(chunk) => chunk.fill_from_iter(samples.iter().copied()),
            Err(ChunkError::TooFewSlots(available)) => {
                self.overrun_count.fetch_add(1, Ordering::Relaxed);
                if available == 0 {
                    return 0;
                }
                let newest = &samples[samples.len() - available..];
                match self.producer.write_chunk_uninit(available) {
                    Ok(chunk) => chunk.fill_from_iter(newest.iter().copied()),
                    // Consumer'is spėjo dar paskaityti tarp dviejų bandymų (retas, nekenksmingas
                    // race) — laisvos vietos gali tik PADAUGĖTI tarp bandymų, ne sumažėti, tad
                    // ši šaka praktiškai nepasiekiama; laikoma tik saugumo dėlei.
                    Err(_) => 0,
                }
            }
        }
    }

    /// Kiek sample'ų šiuo metu laukia perskaitymo, dalis nuo talpos (`0.0..=1.0`) — dinaminiam
    /// rate control'ui (P3.4, CLAUDE.md §8.6).
    pub fn occupancy(&self) -> f64 {
        let capacity = self.producer.buffer().capacity();
        if capacity == 0 {
            return 0.0;
        }
        let free = self.producer.slots();
        (capacity - free) as f64 / capacity as f64
    }

    /// Kiek kartų įvyko overrun (buferis pilnas rašant) — stats/log'inimui iš aukščiau.
    pub fn overrun_count(&self) -> u64 {
        self.overrun_count.load(Ordering::Relaxed)
    }
}

impl AudioConsumer {
    /// Užpildo `out` (interleaved, jau teisingo kanalų išdėstymo) konvertuotais `f32`
    /// sample'ais iš buferio. Jei trūksta — likusi dalis užpildoma tyla (underrun, žr.
    /// modulio doc).
    pub fn fill(&mut self, out: &mut [f32]) {
        if out.is_empty() {
            return;
        }

        let written = match self.consumer.read_chunk(out.len()) {
            Ok(chunk) => {
                let mut written = 0;
                for sample in chunk {
                    out[written] = f32::from(sample) / f32::from(i16::MAX);
                    written += 1;
                }
                written
            }
            Err(ChunkError::TooFewSlots(available)) => {
                self.underrun_count.fetch_add(1, Ordering::Relaxed);
                if available == 0 {
                    0
                } else if let Ok(chunk) = self.consumer.read_chunk(available) {
                    let mut written = 0;
                    for sample in chunk {
                        out[written] = f32::from(sample) / f32::from(i16::MAX);
                        written += 1;
                    }
                    written
                } else {
                    0
                }
            }
        };

        for slot in &mut out[written..] {
            *slot = 0.0;
        }
    }

    /// Kiek kartų įvyko underrun (buferis tuščias skaitant) — stats/log'inimui iš aukščiau
    /// (throttled, NE šioje real-time gijoje — žr. modulio doc).
    pub fn underrun_count(&self) -> u64 {
        self.underrun_count.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn push_and_fill_round_trip_exact_fit() {
        let (mut producer, mut consumer) = new(8);
        assert_eq!(producer.push_samples(&[1, 2, 3, 4]), 4);

        let mut out = [0.0f32; 4];
        consumer.fill(&mut out);
        assert_eq!(
            out,
            [
                1.0 / i16::MAX as f32,
                2.0 / i16::MAX as f32,
                3.0 / i16::MAX as f32,
                4.0 / i16::MAX as f32,
            ]
        );
        assert_eq!(consumer.underrun_count(), 0);
        assert_eq!(producer.overrun_count(), 0);
    }

    #[test]
    fn underrun_fills_silence_and_counts() {
        let (mut producer, mut consumer) = new(8);
        assert_eq!(producer.push_samples(&[100, 200]), 2);

        let mut out = [1.0f32; 4]; // ne nulis — įsitikinam, kad tikrai perrašoma
        consumer.fill(&mut out);
        assert_eq!(out[0], 100.0 / i16::MAX as f32);
        assert_eq!(out[1], 200.0 / i16::MAX as f32);
        assert_eq!(out[2], 0.0, "trūkstama dalis turi būti tyla");
        assert_eq!(out[3], 0.0, "trūkstama dalis turi būti tyla");
        assert_eq!(consumer.underrun_count(), 1);
    }

    #[test]
    fn overrun_keeps_newest_samples_and_counts() {
        let (mut producer, _consumer) = new(4);
        // Talpa 4, bet bandome įrašyti 6 — tik naujausi 4 turėtų likti.
        let written = producer.push_samples(&[1, 2, 3, 4, 5, 6]);
        assert_eq!(written, 4);
        assert_eq!(producer.overrun_count(), 1);
        assert!((producer.occupancy() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overrun_then_consumer_reads_only_newest() {
        let (mut producer, mut consumer) = new(4);
        producer.push_samples(&[1, 2, 3, 4, 5, 6]); // 4 telpa: turėtų likti [3,4,5,6]

        let mut out = [0.0f32; 4];
        consumer.fill(&mut out);
        assert_eq!(
            out,
            [
                3.0 / i16::MAX as f32,
                4.0 / i16::MAX as f32,
                5.0 / i16::MAX as f32,
                6.0 / i16::MAX as f32,
            ],
            "overrun turėjo išmesti SENIAUSIUS (1,2), palikdamas naujausius (3,4,5,6)"
        );
    }

    #[test]
    fn no_allocation_marker_capacity_matches_recommended() {
        // recommended_capacity aritmetikos sveiko proto patikra: SNES ~800 kadrų/s * 2ch *
        // 50ms * 4x atsarga turėtų būti keli tūkstančiai sample'ų, ne nulis/absurdiškai didelis.
        let capacity = recommended_capacity(32040, 2, 50);
        assert!(capacity > 1000 && capacity < 50_000, "capacity={capacity}");
    }

    /// Greitas (kelių sekundžių) sanity testas su ta pačia faze-persijungimo logika kaip
    /// pilnas 60s soak testas (žr. žemiau) — kad įprastas `cargo test` liktų greitas.
    /// `assert_both_phases_triggered=false` — 2s trukmėje lygiai DVI ~1s fazės neturi jokios
    /// atsargos CI VM scheduling svyravimams (patikrinta REALIAI: pravalė macos-latest CI po
    /// P4.0.3, kai CI pirmą kartą realiai pasiekė šį testą apkrautoje bendrai naudojamoje
    /// mašinoje). Testo PATS savo doc komentaras („svarbiausia: testas apskritai pasiekė čia,
    /// nė viena pusė nepanikavo/neužstrigo") jau sakė, kad crash-saugumas, ne tikslus
    /// underrun/overrun skaičius, yra šio GREITO varianto tikslas — griežtas skaičių
    /// patikrinimas priklauso 60s variantui, kuriam laiko atsargos pakanka.
    #[test]
    fn producer_and_consumer_at_different_speeds_short() {
        run_different_speeds_test(Duration::from_secs(2), false);
    }

    /// P3.2 acceptance: producer/consumer skirtingais greičiais 60s, be crash'o/pakibimo, IR
    /// abi fazės realiai suveikė bent kartą. Paleisti rankiniu būdu (pilnas trukmės
    /// patikrinimas):
    /// `cargo test producer_and_consumer_at_different_speeds_for_60_seconds -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn producer_and_consumer_at_different_speeds_for_60_seconds() {
        run_different_speeds_test(Duration::from_secs(60), true);
    }

    /// Producer'io greitis kas ~1s persijungia tarp „greitesnis už consumer'į" (sukelia
    /// overrun) ir „lėtesnis už consumer'į" (sukelia underrun) fazių. `assert_both_phases_
    /// triggered` — ar reikalauti, kad abi fazės REALIAI sukeltų bent po vieną underrun/
    /// overrun (griežta patikra, reikalauja laiko atsargos — žr. `..._short` doc). Abiem
    /// atvejais svarbiausia: gija nesulaužo srauto (nepanikuoja/neužstringa).
    fn run_different_speeds_test(duration: Duration, assert_both_phases_triggered: bool) {
        let (mut producer, mut consumer) = new(recommended_capacity(48000, 2, 50));

        let producer_thread = std::thread::spawn(move || {
            let deadline = Instant::now() + duration;
            let mut counter: i16 = 0;
            let mut phase_start = Instant::now();
            let mut fast_phase = true;
            while Instant::now() < deadline {
                if phase_start.elapsed() >= Duration::from_millis(1000) {
                    fast_phase = !fast_phase;
                    phase_start = Instant::now();
                }
                let chunk: Vec<i16> = (0..64).map(|i| counter.wrapping_add(i)).collect();
                producer.push_samples(&chunk);
                counter = counter.wrapping_add(64);
                // Consumer'is skaito pastoviu ~300us/32 sample'ų greičiu (žemiau) — greitoje
                // fazėje producer'is rašo sparčiau (overrun), lėtoje — lėčiau (underrun).
                std::thread::sleep(if fast_phase {
                    Duration::from_micros(100)
                } else {
                    Duration::from_micros(2000)
                });
            }
            producer.overrun_count()
        });

        let deadline = Instant::now() + duration;
        let mut out = [0.0f32; 32];
        while Instant::now() < deadline {
            consumer.fill(&mut out);
            std::thread::sleep(Duration::from_micros(300));
        }

        let overrun_count = producer_thread
            .join()
            .expect("producer gija neturėtų panikuoti");
        let underrun_count = consumer.underrun_count();

        // Svarbiausia VISADA: testas apskritai pasiekė čia, nė viena pusė nepanikavo/
        // neužstrigo (žr. `producer_thread.join().expect(...)` aukščiau — jei gija
        // panikavo, testas jau būtų nutrūkęs ten). Griežtas abiejų fazių pasireiškimo
        // patikrinimas — TIK kai duota pakankamai laiko atsargos (žr. funkcijos doc).
        if assert_both_phases_triggered {
            assert!(
                underrun_count > 0,
                "lėtoji fazė turėjo sukelti bent vieną underrun'ą"
            );
            assert!(
                overrun_count > 0,
                "greitoji fazė turėjo sukelti bent vieną overrun'ą"
            );
        }
        eprintln!(
            "testas ({duration:?}) baigtas: underrun={underrun_count} overrun={overrun_count}"
        );
    }
}
