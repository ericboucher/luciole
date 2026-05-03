//! Capture microphone input to a WAV file (PCM 16-bit).
//!
//! Uses device native sample rate; whisper.cpp accepts common rates when built with FFmpeg.

use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{InputCallbackInfo, SampleFormat, StreamConfig};

/// RMS brut du dernier buffer (float bits), mis à jour depuis le callback cpal (sans allocation lourde).
fn atomic_store_rms(slot: &AtomicU32, rms: f32) {
    slot.store(rms.to_bits(), Ordering::Relaxed);
}

fn atomic_load_rms(slot: &AtomicU32) -> f32 {
    f32::from_bits(slot.load(Ordering::Relaxed))
}

/// RMS mono sur un chunk f32 entrant (multi-canaux → moyenne par frame).
fn chunk_rms_f32(data: &[f32], channels: usize) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0f32;
    let mut n = 0u32;
    if channels <= 1 {
        for &s in data {
            sum += s * s;
            n += 1;
        }
    } else {
        for frame in data.chunks(channels) {
            let m = frame.iter().sum::<f32>() / channels as f32;
            sum += m * m;
            n += 1;
        }
    }
    (sum / n.max(1) as f32).sqrt()
}

/// RMS mono sur un chunk i16 entrant.
fn chunk_rms_i16(data: &[i16], channels: usize) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let scale = 1.0 / 32768.0f32;
    let mut sum = 0.0f32;
    let mut n = 0u32;
    if channels <= 1 {
        for &s in data {
            let x = s as f32 * scale;
            sum += x * x;
            n += 1;
        }
    } else {
        for frame in data.chunks(channels) {
            let m = frame.iter().map(|&x| x as f32).sum::<f32>() / channels as f32 * scale;
            sum += m * m;
            n += 1;
        }
    }
    (sum / n.max(1) as f32).sqrt()
}

/// Niveau d’affichage 0..1 à partir du RMS (parole typique ~0.005–0.05).
fn rms_to_visual_level(rms: f32) -> f32 {
    const GAIN: f32 = 18.0;
    let v = (rms * GAIN).clamp(0.0, 1.0);
    v * v
}

/// API sans suivi de niveau (réutilisable hors UI onde).
#[allow(dead_code)]
pub fn record_wav_mono(path: &Path, duration: Duration) -> anyhow::Result<()> {
    record_wav_mono_with_levels(path, duration, |_| {})
}

/// Enregistre en mono ; `on_level` est appelé ~30–60×/s avec un niveau 0..1 (animation onde).
pub fn record_wav_mono_with_levels<F>(path: &Path, duration: Duration, mut on_level: F) -> anyhow::Result<()>
where
    F: FnMut(f32),
{
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("aucun périphérique d'entrée audio"))?;

    let supported = device.default_input_config()?;
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();

    let channels = config.channels as usize;
    let samples = Arc::new(Mutex::new(Vec::<i16>::new()));
    let last_rms = Arc::new(AtomicU32::new(0.0f32.to_bits()));

    let stream = match sample_format {
        SampleFormat::F32 => {
            let samples_clone = samples.clone();
            let rms_slot = last_rms.clone();
            device.build_input_stream(
                &config,
                move |data: &[f32], _: &InputCallbackInfo| {
                    atomic_store_rms(&rms_slot, chunk_rms_f32(data, channels));
                    let mut v = samples_clone.lock().unwrap();
                    append_mono_f32(&mut v, data, channels);
                },
                |e| tracing::error!("cpal: {e}"),
                None,
            )?
        }
        SampleFormat::I16 => {
            let samples_clone = samples.clone();
            let rms_slot = last_rms.clone();
            device.build_input_stream(
                &config,
                move |data: &[i16], _: &InputCallbackInfo| {
                    atomic_store_rms(&rms_slot, chunk_rms_i16(data, channels));
                    let mut v = samples_clone.lock().unwrap();
                    append_mono_i16(&mut v, data, channels);
                },
                |e| tracing::error!("cpal: {e}"),
                None,
            )?
        }
        f => anyhow::bail!("format audio non supporté: {:?} (attendu f32 ou i16)", f),
    };

    stream.play()?;
    let start = Instant::now();
    let mut last_emit = Instant::now();
    let mut smoothed = 0.0f32;
    while start.elapsed() < duration {
        let raw = atomic_load_rms(&last_rms);
        smoothed = smoothed * 0.82 + raw * 0.18;
        let level = rms_to_visual_level(smoothed);
        if last_emit.elapsed() >= Duration::from_millis(16) {
            on_level(level);
            last_emit = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(8));
    }
    on_level(0.0);
    drop(stream);

    let all = samples.lock().unwrap().clone();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for s in all {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(())
}

fn append_mono_f32(out: &mut Vec<i16>, data: &[f32], channels: usize) {
    if channels <= 1 {
        for &s in data {
            out.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
        }
        return;
    }
    for chunk in data.chunks(channels) {
        let sum: f32 = chunk.iter().sum::<f32>() / channels as f32;
        out.push((sum.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
}

fn append_mono_i16(out: &mut Vec<i16>, data: &[i16], channels: usize) {
    if channels <= 1 {
        out.extend_from_slice(data);
        return;
    }
    for chunk in data.chunks(channels) {
        let sum: i32 = chunk.iter().map(|&x| x as i32).sum();
        out.push((sum / channels as i32) as i16);
    }
}
