//! Lock-free triple buffer video kadrams tarp emuliavimo ir UI (render) gijų
//! (CLAUDE.md §4, P2.2).
//!
//! Klasikinis triple-buffering algoritmas (Boost/žaidimų varikliukų technika): trys buferiai,
//! producer'is (emu gija) ir consumer'is (UI/render gija) kiekvienas išskirtinai valdo savo
//! privatų indeksą (`write_idx`/`read_idx`), o trečiasis buferis „plūduriuoja" viename
//! atominiame kintamajame (`shared`). Kiekvienas `swap()` transakciškai apkeičia savo turimą
//! indeksą su tuo, kas šiuo metu yra `shared`, todėl abi pusės visada turi savo išskirtinai
//! valdomą buferį — producer'is niekada NELAUKIA consumer'io, o consumer'is visada mato
//! naujausią PILNĄ kadrą (niekada pusiau parašytą).
//!
//! Invariantas (įrodymas žr. commit'o aprašyme): bet kuriuo metu aibė
//! `{write_idx, read_idx, shared_index}` yra kokia nors `{0, 1, 2}` permutacija — kiekvienas
//! iš 3 buferių bet kuriuo metu priklauso lygiai vienai pusei (arba „plūduriuoja" kaip laisvas).

// Naudos video::renderer (P2.3/P2.4) — kol jų nėra, pilnai išnaudojamas tik testuose.
#![allow(dead_code)]

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const NEW_FLAG: usize = 0b100;
const INDEX_MASK: usize = 0b011;

/// Vieno kadro duomenys + metaduomenys.
#[derive(Default)]
pub struct VideoFrameData {
    pub width: u32,
    pub height: u32,
    /// Monotoniškai didėjantis skaitliukas — leidžia consumer'iui atpažinti, ar tai tas
    /// pats kadras, kurį jau matė (pvz. praleisti nereikalingą GPU texture upload).
    pub generation: u64,
    /// RGBA8 baitai, `width * height * 4` ilgio.
    pub data: Vec<u8>,
}

struct Slot(UnsafeCell<VideoFrameData>);

impl Default for Slot {
    fn default() -> Self {
        Self(UnsafeCell::new(VideoFrameData::default()))
    }
}

// SAFETY: `Slot` pasiekiamas tik per `FrameProducer::write_frame` (write_idx) arba
// `FrameConsumer` (read_idx) — abu indeksai yra kiekvienos pusės PRIVATUS būvis, o
// `shared` atominis handoff'as garantuoja, kad joks buferis tuo pačiu metu nepriklauso
// abiem pusėms (žr. modulio doc invariantą). Todėl tarpgijinis prieigos konfliktas
// (data race) negalimas, nors `UnsafeCell` pats savaime nėra `Sync`.
unsafe impl Sync for Slot {}

struct TripleBuffer {
    slots: [Slot; 3],
    shared: AtomicUsize,
}

/// Rašymo pusė (emuliavimo gija). Ne `Clone`, ne `Sync` — turi priklausyti vienai gijai.
pub struct FrameProducer {
    buffer: Arc<TripleBuffer>,
    write_idx: usize,
    generation: u64,
}

/// Skaitymo pusė (UI/render gija). Ne `Clone`, ne `Sync` — turi priklausyti vienai gijai.
pub struct FrameConsumer {
    buffer: Arc<TripleBuffer>,
    read_idx: usize,
}

/// Sukuria naują triple buffer porą. `FrameProducer` atiduodamas emuliavimo gijai,
/// `FrameConsumer` — UI/render gijai.
pub fn new() -> (FrameProducer, FrameConsumer) {
    let buffer = Arc::new(TripleBuffer {
        slots: [Slot::default(), Slot::default(), Slot::default()],
        shared: AtomicUsize::new(2),
    });

    (
        FrameProducer {
            buffer: buffer.clone(),
            write_idx: 0,
            generation: 0,
        },
        FrameConsumer {
            buffer,
            read_idx: 1,
        },
    )
}

impl FrameProducer {
    /// Parašo naują kadrą. `fill` gauna jau teisingo dydžio (`width * height * 4`) baitų
    /// buferį — pakartotinis kvietimas su tuo pačiu `width`/`height` NEALOKUOJA iš naujo.
    /// Niekada nelaukia consumer'io — grąžina iškart.
    pub fn write_frame(&mut self, width: u32, height: u32, fill: impl FnOnce(&mut [u8])) {
        self.generation += 1;
        let needed = width as usize * height as usize * 4;

        // SAFETY: `write_idx` buferis šiuo metu priklauso IŠSKIRTINAI šiam producer'iui —
        // žr. modulio doc invariantą. Joks kitas kodas (consumer'is) jo neliečia, kol jis
        // nebus paskelbtas per žemiau esantį `swap`.
        let slot = unsafe { &mut *self.buffer.slots[self.write_idx].0.get() };
        slot.width = width;
        slot.height = height;
        slot.generation = self.generation;
        if slot.data.len() != needed {
            slot.data.resize(needed, 0);
        }
        fill(&mut slot.data);

        let published = self.write_idx | NEW_FLAG;
        let previous = self.buffer.shared.swap(published, Ordering::AcqRel);
        self.write_idx = previous & INDEX_MASK;
    }

    /// Paskutinio parašyto kadro generacijos numeris.
    pub fn generation(&self) -> u64 {
        self.generation
    }
}

impl FrameConsumer {
    /// Jei yra naujas kadras — persijungia į jį (`true`). Jei ne — `false`, ir toliau
    /// prieinamas senas kadras per [`FrameConsumer::current`].
    pub fn update(&mut self) -> bool {
        let peek = self.buffer.shared.load(Ordering::Acquire);
        if peek & NEW_FLAG == 0 {
            return false;
        }

        let previous = self.buffer.shared.swap(self.read_idx, Ordering::AcqRel);
        self.read_idx = previous & INDEX_MASK;
        true
    }

    /// Nuoroda į šiuo metu turimą kadrą (paskutinį, gautą per [`FrameConsumer::update`]).
    pub fn current(&self) -> &VideoFrameData {
        // SAFETY: `read_idx` buferis šiuo metu priklauso IŠSKIRTINAI šiam consumer'iui —
        // žr. modulio doc invariantą.
        unsafe { &*self.buffer.slots[self.read_idx].0.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::time::{Duration, Instant};

    #[test]
    fn consumer_sees_latest_frame_after_update() {
        let (mut producer, mut consumer) = new();

        producer.write_frame(2, 2, |data| data.fill(0xAA));
        assert!(consumer.update());
        assert_eq!(consumer.current().generation, 1);
        assert_eq!(consumer.current().data, vec![0xAA; 16]);

        producer.write_frame(2, 2, |data| data.fill(0xBB));
        producer.write_frame(2, 2, |data| data.fill(0xCC));
        // Consumer'is „pramiega" tarpinį kadrą — turi matyti tik PASKUTINĮ (0xCC), ne 0xBB.
        assert!(consumer.update());
        assert_eq!(consumer.current().generation, 3);
        assert_eq!(consumer.current().data, vec![0xCC; 16]);
    }

    #[test]
    fn update_returns_false_when_no_new_frame() {
        let (mut producer, mut consumer) = new();
        producer.write_frame(1, 1, |data| data.fill(1));
        assert!(consumer.update());
        assert!(
            !consumer.update(),
            "antras update() be naujo kadro turėtų grąžinti false"
        );
    }

    #[test]
    fn resize_between_frames_does_not_corrupt_data() {
        let (mut producer, mut consumer) = new();
        producer.write_frame(4, 4, |data| data.fill(0x11)); // 64 baitai
        producer.write_frame(2, 2, |data| data.fill(0x22)); // 16 baitų — mažesnis kadras

        consumer.update();
        let frame = consumer.current();
        assert_eq!(frame.width, 2);
        assert_eq!(frame.height, 2);
        assert_eq!(frame.data.len(), 16);
        assert!(frame.data.iter().all(|&b| b == 0x22));
    }

    fn validate_frame(
        frame: &VideoFrameData,
        width: u32,
        height: u32,
        max_generation_seen: &mut u64,
        frames_observed: &mut u64,
    ) {
        let expected_marker = (frame.generation % 256) as u8;
        assert!(
            frame.data.iter().all(|&b| b == expected_marker),
            "kadras generacija={} turi nenuoseklius baitus (suplėšytas skaitymas!)",
            frame.generation
        );
        assert_eq!(frame.width, width);
        assert_eq!(frame.height, height);
        *max_generation_seen = (*max_generation_seen).max(frame.generation);
        *frames_observed += 1;
    }

    /// P2.2 acceptance: 2 gijos, 10 000 kadrų, jokio data race. Kiekvieno kadro visi baitai
    /// užpildomi ta pačia reikšme (generacijos numeris mod 256) — jei triple buffer
    /// protokolas turėtų klaidą (pvz. leistų matyti „suplėšytą" kadrą, kur dalis baitų iš
    /// vieno rašymo, dalis iš kito), šis nuoseklumo patikrinimas tai iškart pagautų.
    #[test]
    fn stress_10_000_frames_across_two_threads_no_tearing() {
        const FRAME_COUNT: u64 = 10_000;
        const WIDTH: u32 = 64;
        const HEIGHT: u32 = 64;

        let (mut producer, mut consumer) = new();
        let done = Arc::new(AtomicBool::new(false));
        let done_writer = done.clone();

        let producer_thread = std::thread::spawn(move || {
            let mut max_duration = Duration::ZERO;
            for gen in 1..=FRAME_COUNT {
                let marker = (gen % 256) as u8;
                let start = Instant::now();
                producer.write_frame(WIDTH, HEIGHT, |data| data.fill(marker));
                max_duration = max_duration.max(start.elapsed());
            }
            // Release: viskas, ką producer'is parašė iki šiol (įskaitant paskutinį swap),
            // tampa matoma consumer'iui, kai jis pamato `done == true` per Acquire load
            // (žr. paaiškinimą žemiau prieš paskutinę patikrą).
            done_writer.store(true, Ordering::Release);
            max_duration
        });

        let mut max_generation_seen = 0u64;
        let mut frames_observed = 0u64;
        loop {
            if consumer.update() {
                validate_frame(
                    consumer.current(),
                    WIDTH,
                    HEIGHT,
                    &mut max_generation_seen,
                    &mut frames_observed,
                );
                continue;
            }

            if done.load(Ordering::Acquire) {
                // Galimas lenktynių langas: paskutinis kadras galėjo būti paskelbtas TARP
                // aukščiau esančio update() ir šio done patikrinimo. Kadangi `done`
                // (Release) rašomas TIK po paskutinio swap (program order tame pačiame
                // gijoje), o mes ką tik pamatėme `done == true` (Acquire) — happens-before
                // tranzityvumas garantuoja, kad paskutinis swap jau ĮVYKO. Todėl ŠI VIENA
                // papildoma update() patikra tikrai pagaus paskutinį kadrą, jei jis dar
                // nebuvo pagautas aukščiau.
                if consumer.update() {
                    validate_frame(
                        consumer.current(),
                        WIDTH,
                        HEIGHT,
                        &mut max_generation_seen,
                        &mut frames_observed,
                    );
                }
                break;
            }
        }

        let max_write_duration = producer_thread
            .join()
            .expect("producer gija neturėtų panikuoti");

        assert_eq!(
            max_generation_seen, FRAME_COUNT,
            "consumer'is turėjo pamatyti paskutinį (10000-ąjį) kadrą"
        );
        assert!(frames_observed > 0);
        assert!(
            max_write_duration < Duration::from_millis(5),
            "write_frame per lėtas (blokavosi?): {max_write_duration:?}"
        );

        eprintln!(
            "stress testas: {frames_observed} kadrų pamatyta iš {FRAME_COUNT} parašytų, \
             ilgiausias write_frame: {max_write_duration:?}"
        );
    }
}
