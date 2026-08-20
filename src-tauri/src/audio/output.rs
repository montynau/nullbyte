//! cpal garso išvesties srautas (P3.1, CLAUDE.md §3.2 taisyklė #3, §8.6).
//!
//! **Audio callback'e** (kviečiamas OS real-time audio gijos) GRIEŽTAI draudžiama: alokuoti
//! atmintį, imti `Mutex`, kviesti `println!`/`tracing` (I/O), blokuotis. `sample_source`
//! closure'as, perduodamas į [`AudioOutput::open`], veikia BŪTENT šioje gijoje — P3.2 ring
//! buferis bus vienintelis leistinas šaltinis produkcijoje; P3.1 testuose naudojamas grynai
//! skaičiuojamas sinusoidės generatorius (jokios alokacijos/I/O per callback'ą).
//!
//! Klaidų callback'as (`error_callback`, pvz. įrenginio dingimas) kviečiamas cpal vidinės,
//! NE real-time audio gijos — ten `tracing::error!` yra saugus (CLAUDE.md §3.2 taisyklė #3
//! galioja tik DUOMENŲ callback'ui, ne klaidų callback'ui).

// Pilnai išnaudos P3.2 ring buffer'is — P3.1 metu `sample_source` tiekia tik testai.
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, StreamConfig};

use crate::error::AppError;

/// Tikslinis buferio latency (P3.1 „Ką daryti": ~40–60 ms). `pub(crate)`, kad
/// `audio::ring`'o talpa (P3.2/P3.4, `recommended_capacity`) būtų skaičiuojama nuo TO PATIES
/// dydžio, kurį realiai naudoja cpal srautas — kitaip „~4x buferio dydžio" reikštų du
/// skirtingus, nesuderintus dydžius.
pub(crate) const TARGET_LATENCY_MS: u32 = 50;

/// Veikiantis garso išvesties srautas. `Drop` (per vidinį `cpal::Stream`) sustabdo srautą
/// automatiškai.
pub struct AudioOutput {
    stream: cpal::Stream,
    sample_rate: u32,
    channels: u16,
    device_lost: Arc<AtomicBool>,
}

/// Numatytojo įrenginio derybų rezultatas — kviesk PRIEŠ [`AudioOutput::open`], kad
/// `sample_source` closure'as žinotų tikrą sample rate/channels dar prieš srauto sukūrimą
/// (pvz. testinės sinusoidės fazės žingsniui apskaičiuoti).
pub fn default_config() -> Result<(u32, u16), AppError> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| {
        AppError::Other("nerastas numatytasis garso išvesties įrenginys".to_string())
    })?;
    let config = device.default_output_config().map_err(|e| {
        AppError::Other(format!(
            "nepavyko gauti garso įrenginio konfigūracijos: {e}"
        ))
    })?;
    Ok((config.sample_rate().0, config.channels()))
}

impl AudioOutput {
    /// Atidaro numatytąjį garso išvesties įrenginį ir paleidžia srautą su `sample_source`
    /// callback'u (kviečiamas real-time audio gijoje — žr. modulio doc apribojimus).
    /// `sample_source` gauna interleaved stereo/multi-channel `f32` buferį užpildymui.
    pub fn open<F>(sample_source: F) -> Result<Self, AppError>
    where
        F: FnMut(&mut [f32], u16) + Send + 'static,
    {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| {
            AppError::Other("nerastas numatytasis garso išvesties įrenginys".to_string())
        })?;
        let device_name = device.name().unwrap_or_else(|_| "?".to_string());

        let supported_config = device.default_output_config().map_err(|e| {
            AppError::Other(format!(
                "nepavyko gauti garso įrenginio konfigūracijos: {e}"
            ))
        })?;

        let sample_format = supported_config.sample_format();
        let channels = supported_config.channels();
        let sample_rate = supported_config.sample_rate().0;

        let mut config: StreamConfig = supported_config.into();
        let buffer_frames = (sample_rate * TARGET_LATENCY_MS / 1000).max(1);
        config.buffer_size = cpal::BufferSize::Fixed(buffer_frames);
        // 2x atsarga — jei backend'as callback'ą vis tiek iškviečia su kitokiu dydžiu nei
        // prašytas `Fixed`, scratch buferis (žr. `build_stream`) turi tilpti be alokacijos
        // hot path'e (per didelis kadras tiesiog nukerpamas, ne panikuoja/alokuoja).
        let scratch_capacity = buffer_frames as usize * channels as usize * 2;

        let device_lost = Arc::new(AtomicBool::new(false));
        let device_lost_for_err = device_lost.clone();
        let error_callback = move |err: cpal::StreamError| {
            tracing::error!(error = %err, "cpal audio srauto klaida (įrenginys dingo?)");
            device_lost_for_err.store(true, Ordering::Relaxed);
        };

        let stream = match sample_format {
            SampleFormat::F32 => build_stream::<f32>(
                &device,
                &config,
                channels,
                scratch_capacity,
                sample_source,
                error_callback,
            ),
            SampleFormat::I16 => build_stream::<i16>(
                &device,
                &config,
                channels,
                scratch_capacity,
                sample_source,
                error_callback,
            ),
            other => {
                return Err(AppError::Other(format!(
                    "nepalaikomas garso sample formatas: {other:?}"
                )))
            }
        }
        .map_err(|e| AppError::Other(format!("nepavyko sukurti audio srauto: {e}")))?;

        stream
            .play()
            .map_err(|e| AppError::Other(format!("nepavyko paleisti audio srauto: {e}")))?;

        tracing::info!(
            device = %device_name,
            sample_rate,
            channels,
            ?sample_format,
            latency_ms = TARGET_LATENCY_MS,
            "cpal audio srautas paleistas"
        );

        Ok(Self {
            stream,
            sample_rate,
            channels,
            device_lost,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// `true`, jei klaidų callback'as pranešė apie srauto/įrenginio klaidą (pvz. ausinių
    /// atjungimą). Aukštesnio lygio kodas (P3.4+) gali tuo remdamasis bandyti [`AudioOutput::open`]
    /// iš naujo naujam numatytajam įrenginiui — švarus atsistatymas, ne crash.
    pub fn is_device_lost(&self) -> bool {
        self.device_lost.load(Ordering::Relaxed)
    }
}

/// Sukuria cpal srautą konkrečiam įrenginio native sample tipui `T`. `sample_source` visada
/// generuoja `f32` — konvertavimas į `T` vyksta ČIA PAT callback'e (grynas skaičiavimas, be
/// alokacijos), naudojant VIENĄ kartą iš anksto paskirstytą `scratch` buferį.
fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: u16,
    scratch_capacity: usize,
    mut sample_source: impl FnMut(&mut [f32], u16) + Send + 'static,
    error_callback: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample + FromSample<f32>,
{
    let mut scratch = vec![0.0f32; scratch_capacity];
    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let n = data.len().min(scratch.len());
            sample_source(&mut scratch[..n], channels);
            for (out, &s) in data.iter_mut().zip(scratch[..n].iter()) {
                *out = T::from_sample(s);
            }
            for out in &mut data[n..] {
                *out = T::from_sample(0.0f32);
            }
        },
        error_callback,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;
    use std::time::Duration;

    /// Sukuria 440 Hz sinusoidės generatorių, žinantį TIKRĄ (ne spėjamą) įrenginio sample
    /// rate — gaunamą iš [`default_config`] PRIEŠ atidarant srautą.
    fn sine_source(sample_rate: u32) -> impl FnMut(&mut [f32], u16) {
        let increment = 440.0 / sample_rate as f32;
        let mut phase = 0.0f32;
        move |buf: &mut [f32], channels: u16| {
            for frame in buf.chunks_mut(channels as usize) {
                let value = (phase * TAU).sin() * 0.2; // 0.2 = kuklus garsumas testams
                for sample in frame.iter_mut() {
                    *sample = value;
                }
                phase = (phase + increment).fract();
            }
        }
    }

    /// Greitas sanity testas — pilnas 30s klausomas testas yra `#[ignore]`'intas
    /// `plays_440hz_sine_for_30_seconds` (žr. žemiau), kad įprastas `cargo test` liktų greitas.
    #[test]
    fn opens_and_closes_cleanly_without_panic() {
        let Ok((sample_rate, _channels)) = default_config() else {
            eprintln!("praleista: nerastas garso išvesties įrenginys (CI/headless aplinka)");
            return;
        };

        let output = match AudioOutput::open(sine_source(sample_rate)) {
            Ok(output) => output,
            Err(error) => {
                eprintln!("praleista: nepavyko atidaryti garso srauto ({error})");
                return;
            }
        };

        std::thread::sleep(Duration::from_millis(200));
        assert!(!output.is_device_lost());
        drop(output);
    }

    /// P3.1 acceptance: 440 Hz sinusoidė girdima švariai 30 sekundžių, be crash'o.
    /// Paleisti rankiniu būdu (garsiakalbiuose/ausinėse turi būti girdimas švarus tonas):
    /// `cargo test --release plays_440hz_sine_for_30_seconds -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn plays_440hz_sine_for_30_seconds() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let (sample_rate, _channels) = default_config().expect("garso įrenginys turėtų būti");
        let output =
            AudioOutput::open(sine_source(sample_rate)).expect("srautas turėtų atsidaryti");

        std::thread::sleep(Duration::from_secs(30));

        assert!(
            !output.is_device_lost(),
            "įrenginys pranešė apie klaidą per 30s groję"
        );
    }
}
