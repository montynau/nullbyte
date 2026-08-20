//! Garso resampling: core'o `av_info.timing.sample_rate` → įrenginio sample rate (P3.3,
//! CLAUDE.md §8.6). Naudoja `rubato::SincFixedIn` — aukštos kokybės windowed-sinc
//! interpoliacija, kad išvengtume aliasing artefaktų (P3.3 acceptance).
//!
//! **Resampling vyksta emuliavimo gijoje**, ne audio callback'e (P3.3 „Ką daryti") — čia
//! neapribota CLAUDE.md §3.2 taisyklė #3 (ta galioja tik realaus laiko audio callback'ui,
//! žr. `audio/ring.rs`/`audio/output.rs`). Vis dėlto laikomasi tos pačios disciplinos: po
//! pirmo [`AudioResampler::new`] kvietimo visi vidiniai buferiai iš anksto paskirstyti
//! (`input_buffer_allocate`/`output_buffer_allocate`), tad [`AudioResampler::process`]
//! nealokuoja pakartotinai — tenkina „< 1 ms per kadrą" acceptance kriterijų.
//!
//! `rubato` dirba su NEINTERLEAVED (planar) `Vec<Vec<f32>>` duomenimis — po vieną buferį
//! kiekvienam kanalui. Mūsų pipeline (libretro `retro_audio_sample_batch_t`, `audio/ring.rs`)
//! visur naudoja INTERLEAVED `i16`. Šis modulis atlieka abu konvertavimus (interleave ↔
//! planar, `i16` ↔ `f32`) ir kaupia likutį vidiniame buferyje tarp kvietimų, nes
//! `SincFixedIn` reikalauja FIKSUOTO įvesties kadrų kiekio (`chunk_size`) kiekvienam
//! `process_into_buffer` kvietimui, o core'o `audio_sample_batch_cb` chunk'ų dydžiai
//! nesutampa su juo.
//!
//! [`AudioResampler::adjust_ratio`] (P3.4) įgyvendina CLAUDE.md §8.6 dinaminį rate
//! control'ą — nedidelė (±`MAX_DELTA`) resampling ratio korekcija pagal audio ring buferio
//! occupancy, kad garso plokštė taptų emuliavimo laikrodžiu (`core::runner`).

#![allow(dead_code)] // occupancy-pagrįstas pacing'as pilnai išnaudojamas tik core::runner (P3.4).

use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType};

use crate::error::CoreError;

/// Windowed-sinc filtro ilgis — kompromisas tarp kokybės (aukštesnė reikšmė = mažiau
/// aliasing/aukšto dažnio roll-off) ir CPU kaštų. 128 — solidi kokybė, patikrinta
/// benchmark'u, kad tenkina „< 1 ms per kadrą" (žr. testus).
const SINC_LEN: usize = 128;
/// Kiek įvesties kadrų imama per vieną `process_into_buffer` kvietimą. Mažesnė reikšmė —
/// mažesnis pridėtas latency (greičiau pasirodo pirmas išvesties kadras), didesnė —
/// šiek tiek efektyvesnis CPU naudojimas. 512 apytiksliai atitinka vieno žaidimo kadro
/// audio batch'o dydį prie tipinių core sample rate'ų (~500–750 kadrų/60fps).
const CHUNK_SIZE: usize = 512;
/// Kuklus rezervas dinaminiam rate control'ui (P3.4, CLAUDE.md §8.6 `MAX_DELTA = 0.005`) —
/// leidžia vėliau kviesti `set_resample_ratio_relative` be resampler'io perkūrimo.
const MAX_RELATIVE_RATIO: f64 = 1.1;
/// Didžiausias leidžiamas ratio nuokrypis nuo bazinio (CLAUDE.md §8.6) — 0.5%, nepastebima
/// ausiai, bet pakanka ilgainiui centruoti ring buferio occupancy apie 50%.
const MAX_DELTA: f64 = 0.005;

/// Vieno core'o sesijos resampler'is: core sample rate → įrenginio sample rate, interleaved
/// `i16` į interleaved `i16`.
pub struct AudioResampler {
    resampler: SincFixedIn<f32>,
    channels: usize,
    chunk_size: usize,
    /// Deinterleaved, dar nepilnai sukauptas įvesties likutis (po vieną `Vec` kanalui).
    input_staging: Vec<Vec<f32>>,
    /// Iš anksto paskirstyti `process_into_buffer` buferiai — pakartotinai naudojami,
    /// jokios alokacijos po `new()`.
    process_in: Vec<Vec<f32>>,
    process_out: Vec<Vec<f32>>,
    /// Interleaved `i16` išvestis, grąžinama iš [`AudioResampler::process`]. Išvaloma
    /// (`clear()`, ne realokuojama) kiekvieno kvietimo pradžioje.
    output_i16: Vec<i16>,
}

impl AudioResampler {
    /// Sukuria naują resampler'į. `input_rate`/`output_rate` — Hz (core → įrenginys),
    /// `channels` — kanalų kiekis (visi šiuo metu palaikomi core'ai — stereo, 2).
    pub fn new(input_rate: f64, output_rate: f64, channels: usize) -> Result<Self, CoreError> {
        if input_rate <= 0.0 || output_rate <= 0.0 || channels == 0 {
            return Err(CoreError::Other(format!(
                "neteisingi resampler'io parametrai: input_rate={input_rate} output_rate={output_rate} channels={channels}"
            )));
        }

        let parameters = SincInterpolationParameters {
            sinc_len: SINC_LEN,
            f_cutoff: 0.95,
            oversampling_factor: 128,
            interpolation: SincInterpolationType::Cubic,
            window: rubato::WindowFunction::BlackmanHarris2,
        };

        let resample_ratio = output_rate / input_rate;
        let resampler = SincFixedIn::<f32>::new(
            resample_ratio,
            MAX_RELATIVE_RATIO,
            parameters,
            CHUNK_SIZE,
            channels,
        )
        .map_err(|e| CoreError::Other(format!("nepavyko sukurti resampler'io: {e}")))?;

        let process_in = resampler.input_buffer_allocate(true);
        let process_out = resampler.output_buffer_allocate(true);

        Ok(Self {
            resampler,
            channels,
            chunk_size: CHUNK_SIZE,
            input_staging: vec![Vec::with_capacity(CHUNK_SIZE * 2); channels],
            process_in,
            process_out,
            output_i16: Vec::with_capacity(CHUNK_SIZE * 4),
        })
    }

    /// Priima interleaved `i16` sample'us `input_rate` dažniu, grąžina interleaved `i16`
    /// sample'us `output_rate` dažniu. Rezultatas gali būti tuščias, jei dar nesukaupta
    /// pilno `chunk_size` kadrų — likutis saugomas vidiniame buferyje kitam kvietimui.
    pub fn process(&mut self, interleaved_in: &[i16]) -> Result<&[i16], CoreError> {
        self.output_i16.clear();

        let frames_in = interleaved_in.len() / self.channels;
        for frame in 0..frames_in {
            for (ch, staging) in self.input_staging.iter_mut().enumerate() {
                let sample = interleaved_in[frame * self.channels + ch];
                staging.push(f32::from(sample) / f32::from(i16::MAX));
            }
        }

        while self.input_staging[0].len() >= self.chunk_size {
            for (ch, staging) in self.input_staging.iter().enumerate() {
                self.process_in[ch][..self.chunk_size].copy_from_slice(&staging[..self.chunk_size]);
            }

            let (_, out_frames) = self
                .resampler
                .process_into_buffer(&self.process_in, &mut self.process_out, None)
                .map_err(|e| CoreError::Other(format!("resampling nepavyko: {e}")))?;

            for frame in 0..out_frames {
                for channel_out in &self.process_out[..self.channels] {
                    let sample = channel_out[frame];
                    let scaled = (sample * f32::from(i16::MAX))
                        .clamp(f32::from(i16::MIN), f32::from(i16::MAX));
                    self.output_i16.push(scaled as i16);
                }
            }

            for staging in &mut self.input_staging {
                staging.drain(..self.chunk_size);
            }
        }

        Ok(&self.output_i16)
    }

    /// Nedidelis (±`MAX_DELTA`) resampling ratio koregavimas pagal ring buferio occupancy
    /// (CLAUDE.md §8.6, P3.4). `deviation` — `(occupancy - 0.5) * 2.0`, apytiksliai
    /// `-1.0..=1.0`. Glotniai (`ramp = true`) pereina prie naujo ratio per kitą chunk'ą —
    /// vengia staigių, girdimų dažnio šuolių.
    pub fn adjust_ratio(&mut self, deviation: f64) -> Result<(), CoreError> {
        let relative = 1.0 + MAX_DELTA * deviation;
        self.resampler
            .set_resample_ratio_relative(relative, true)
            .map_err(|e| CoreError::Other(format!("nepavyko koreguoti resampling ratio: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const CHANNELS: usize = 2;

    /// Generuoja `frames` kadrų mono sinusoidę (abu kanalai vienodi), `frequency` Hz,
    /// `sample_rate` dažniu, interleaved `i16`.
    fn generate_sine(frequency: f64, sample_rate: f64, frames: usize) -> Vec<i16> {
        let mut out = Vec::with_capacity(frames * CHANNELS);
        for i in 0..frames {
            let t = i as f64 / sample_rate;
            let value = (2.0 * PI * frequency * t).sin();
            let sample = (value * i16::MAX as f64 * 0.5) as i16; // 0.5 — vengiam clipping'o
            for _ in 0..CHANNELS {
                out.push(sample);
            }
        }
        out
    }

    /// Goertzel algoritmas — vieno dažnio bino energijos įvertis, be FFT priklausomybės.
    /// Naudojamas TIK testuose (tono/aliasing patikroms), ne produkciniame kelyje.
    fn goertzel_magnitude(samples: &[f32], sample_rate: f64, target_freq: f64) -> f64 {
        let n = samples.len() as f64;
        let k = (n * target_freq / sample_rate).round();
        let omega = 2.0 * PI * k / n;
        let cos_omega = omega.cos();
        let coeff = 2.0 * cos_omega;

        let (mut s_prev, mut s_prev2) = (0.0f64, 0.0f64);
        for &sample in samples {
            let s = f64::from(sample) + coeff * s_prev - s_prev2;
            s_prev2 = s_prev;
            s_prev = s;
        }
        (s_prev2.powi(2) + s_prev.powi(2) - coeff * s_prev * s_prev2).sqrt() / n
    }

    /// Vieno kanalo mono `f32` sample'ų ištraukimas iš interleaved `i16` (testų analizei).
    fn deinterleave_mono(interleaved: &[i16]) -> Vec<f32> {
        interleaved
            .iter()
            .step_by(CHANNELS)
            .map(|&s| f32::from(s) / f32::from(i16::MAX))
            .collect()
    }

    /// Perleidžia visą signalą per resampler'į vienu kvietimu (testams pakanka —
    /// tikras runner.rs naudos mažesnius, kadrais suskirstytus batch'us P3.4 metu).
    fn resample_all(input_rate: f64, output_rate: f64, input: &[i16]) -> Vec<i16> {
        let mut resampler = AudioResampler::new(input_rate, output_rate, CHANNELS)
            .expect("resampler'is turėtų sėkmingai sukurti");
        let output = resampler.process(input).expect("process() neturėtų klysti");
        output.to_vec()
    }

    /// Patikrina, kad `frequency` Hz tonas, perleistas per resampler'į, išlieka TA PAČIA
    /// dažnio verte (ne per aukštai/žemai) — P3.3 acceptance „garsas skamba teisingu tonu".
    fn assert_pitch_preserved(input_rate: f64, output_rate: f64, frequency: f64) {
        // 0.5s signalo — pakanka Goertzel analizei ir keliems pilnams chunk'ams.
        let frames_in = (input_rate * 0.5) as usize;
        let input = generate_sine(frequency, input_rate, frames_in);
        let output = resample_all(input_rate, output_rate, &input);

        assert!(
            !output.is_empty(),
            "resampler'is turėjo grąžinti bent vieną chunk'ą iš 0.5s įvesties"
        );

        let mono = deinterleave_mono(&output);
        let magnitude_at_freq = goertzel_magnitude(&mono, output_rate, frequency);

        // Palyginimui — energija per klaidingą dažnį (pvz. jei ratio būtų apverstas).
        let wrong_freq = frequency * input_rate / output_rate;
        let magnitude_at_wrong_freq = goertzel_magnitude(&mono, output_rate, wrong_freq);

        assert!(
            magnitude_at_freq > magnitude_at_wrong_freq * 3.0,
            "{frequency}Hz energija ({magnitude_at_freq}) turėtų aiškiai dominuoti prieš \
             klaidingo santykio dažnį {wrong_freq}Hz ({magnitude_at_wrong_freq}) — \
             {input_rate}→{output_rate}"
        );
    }

    #[test]
    fn snes_rate_preserves_440hz_pitch() {
        assert_pitch_preserved(32040.0, 48000.0, 440.0);
    }

    #[test]
    fn genesis_rate_preserves_440hz_pitch() {
        assert_pitch_preserved(44100.0, 48000.0, 440.0);
    }

    #[test]
    fn gba_rate_preserves_440hz_pitch() {
        assert_pitch_preserved(32768.0, 48000.0, 440.0);
    }

    /// P3.3 acceptance „Nėra aliasing artefaktų": aukšto dažnio (arti įvesties Nyquist)
    /// tonas po resampling'o neturėtų sukurti reikšmingos energijos veidrodiniame
    /// (alias) dažnyje.
    #[test]
    fn near_nyquist_tone_does_not_alias() {
        let input_rate = 32040.0;
        let output_rate = 48000.0;
        let test_freq = input_rate * 0.45; // arti (bet ne per arti) įvesties Nyquist (0.5)

        let frames_in = (input_rate * 0.5) as usize;
        let input = generate_sine(test_freq, input_rate, frames_in);
        let output = resample_all(input_rate, output_rate, &input);
        let mono = deinterleave_mono(&output);

        let magnitude_at_signal = goertzel_magnitude(&mono, output_rate, test_freq);
        // Klasikinis aliasing veidrodis dėl blogo anti-aliasing filtro būtų ties
        // (input_rate - test_freq) dažniu, atspindėtu į naują sample rate.
        let alias_freq = input_rate - test_freq;
        let magnitude_at_alias = goertzel_magnitude(&mono, output_rate, alias_freq);

        assert!(
            magnitude_at_signal > magnitude_at_alias * 10.0,
            "signalo energija ({magnitude_at_signal} @ {test_freq}Hz) turėtų DAUG viršyti \
             alias energiją ({magnitude_at_alias} @ {alias_freq}Hz)"
        );
    }

    /// P3.3 acceptance: resampling < 1 ms per kadrą (tipinis core audio batch'as).
    #[test]
    fn resampling_under_1ms_per_frame() {
        let input_rate = 32040.0;
        let output_rate = 48000.0;
        // ~1 kadro audio batch'as 60fps (534 kadrai) — realistiškas dydis iš runner.rs.
        let frame_batch = generate_sine(440.0, input_rate, 534);

        let mut resampler = AudioResampler::new(input_rate, output_rate, CHANNELS)
            .expect("resampler'is turėtų sėkmingai sukurti");

        // Apšilimas.
        for _ in 0..10 {
            let _ = resampler.process(&frame_batch);
        }

        let iterations = 100;
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = resampler
                .process(&frame_batch)
                .expect("process() neturėtų klysti");
        }
        let per_call = start.elapsed() / iterations;

        let limit_ms = if cfg!(debug_assertions) { 10.0 } else { 1.0 };
        assert!(
            per_call.as_secs_f64() * 1000.0 < limit_ms,
            "resampling per lėtas: {per_call:?} (limitas {limit_ms} ms, debug={})",
            cfg!(debug_assertions)
        );
    }

    #[test]
    fn invalid_parameters_return_error_not_panic() {
        assert!(AudioResampler::new(0.0, 48000.0, 2).is_err());
        assert!(AudioResampler::new(32040.0, 0.0, 2).is_err());
        assert!(AudioResampler::new(32040.0, 48000.0, 0).is_err());
    }

    /// P3.4: occupancy-pagrįsta ratio korekcija (CLAUDE.md §8.6) turėtų priimti visą
    /// realistinį `deviation` diapazoną (`-1.0..=1.0`) be klaidos — patvirtina, kad
    /// `MAX_DELTA=0.005` visada telpa į `MAX_RELATIVE_RATIO=1.1` ribas.
    #[test]
    fn adjust_ratio_accepts_full_deviation_range() {
        let mut resampler =
            AudioResampler::new(32040.0, 48000.0, CHANNELS).expect("turėtų sukurti");
        for deviation in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            assert!(
                resampler.adjust_ratio(deviation).is_ok(),
                "deviation={deviation} turėtų būti priimtas"
            );
        }
    }

    #[test]
    fn leftover_samples_carry_over_between_calls() {
        let mut resampler =
            AudioResampler::new(32040.0, 48000.0, CHANNELS).expect("turėtų sukurti");

        // Mažiau nei chunk_size kadrų — neturėtų grąžinti nieko iš karto.
        let small_batch = generate_sine(440.0, 32040.0, 100);
        let first = resampler.process(&small_batch).expect("neturėtų klysti");
        assert!(first.is_empty(), "nepakanka duomenų pilnam chunk'ui");

        // Papildomas batch'as, kartu su ankstesniu likučiu, turėtų viršyti chunk_size.
        let more = generate_sine(440.0, 32040.0, 1000);
        let second = resampler.process(&more).expect("neturėtų klysti");
        assert!(
            !second.is_empty(),
            "sukaupus > chunk_size kadrų turėjo pasirodyti išvestis"
        );
    }

    /// Rankinis klausomas P3.3 acceptance testas: 440Hz tonas SNES core rate'u (32040Hz),
    /// resample'intas į realaus įrenginio rate ir grojamas per `audio/output.rs` — patikrina
    /// „garsas skamba teisingu tonu" iš tikrųjų ausimis, ne vien skaičiais. Paleisti rankiniu
    /// būdu: `cargo test --release plays_resampled_440hz_snes_tone -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn plays_resampled_440hz_snes_tone() {
        use crate::audio::output::{default_config, AudioOutput};

        let (device_rate, device_channels) = default_config().expect("garso įrenginys turėtų būti");
        let snes_rate = 32040.0;

        // 6s 440Hz tono SNES rate'u, resample'inta VISA iš karto (testui pakanka; runner.rs
        // per-kadrą batch'us sujungs P3.4).
        let input = generate_sine(440.0, snes_rate, (snes_rate * 6.0) as usize);
        let mut resampler =
            AudioResampler::new(snes_rate, f64::from(device_rate), device_channels as usize)
                .expect("resampler'is turėtų sukurti");
        let resampled = resampler
            .process(&input)
            .expect("process() neturėtų klysti")
            .to_vec();

        // Closure'as PILNAI valdo savo būvį (paprastas moved iterator, jokio Mutex/Arc) —
        // audio callback'e negalima imti Mutex (CLAUDE.md §3.2 taisyklė #3).
        let mut samples = resampled.into_iter();
        let output = AudioOutput::open(move |buf: &mut [f32], _channels: u16| {
            for slot in buf.iter_mut() {
                *slot = samples
                    .next()
                    .map(|s| f32::from(s) / f32::from(i16::MAX))
                    .unwrap_or(0.0);
            }
        })
        .expect("audio srautas turėtų atsidaryti");

        std::thread::sleep(std::time::Duration::from_secs(7));
        drop(output);
    }
}
