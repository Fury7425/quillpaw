use std::collections::VecDeque;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use chrono::Utc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use hound::{SampleFormat as WavSampleFormat, WavSpec, WavWriter};
use serde::Serialize;
use tauri::Emitter;
use tokio::sync::{mpsc, oneshot, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::fs_manager;

// Wrapper to make cpal::Stream usable in a static context.
// cpal::Stream is !Send because of platform internals, but we only
// ever access it behind a Mutex on the same thread that created it
// (or to drop it). Using an unsafe Send/Sync wrapper is the accepted
// pattern for cpal stream handles stored in global state.
struct SendStream(cpal::Stream);
unsafe impl Send for SendStream {}
unsafe impl Sync for SendStream {}

static STT_STATE: OnceLock<Mutex<SttState>> = OnceLock::new();

#[derive(Default)]
struct SttState {
    active: bool,
    device_name: Option<String>,
    model_path: Option<String>,
    stream: Option<SendStream>,
    stop_tx: Option<oneshot::Sender<()>>,
}

#[derive(Serialize, Clone)]
struct SttEvent {
    text: String,
    is_final: bool,
    audio_path: Option<String>,
}

fn state() -> &'static Mutex<SttState> {
    STT_STATE.get_or_init(|| Mutex::new(SttState::default()))
}

/// Start the speech-to-text engine.
pub async fn start(app: tauri::AppHandle, vault_path: String) -> Result<(), String> {
    let mut guard = state().lock().await;
    if guard.active {
        return Ok(());
    }
    let model_path = guard
        .model_path
        .clone()
        .ok_or_else(|| "STT model path not set.".to_string())?;
    let device = select_device(&guard.device_name)?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let (tx, rx) = mpsc::channel::<Vec<f32>>(64);
    let (stop_tx, stop_rx) = oneshot::channel();
    let stream = build_stream(device, config.sample_format(), channels, tx)?;
    stream.0.play().map_err(|e| e.to_string())?;

    tokio::spawn(run_pipeline(
        app,
        vault_path,
        model_path,
        sample_rate,
        channels,
        rx,
        stop_rx,
    ));

    guard.active = true;
    guard.stream = Some(stream);
    guard.stop_tx = Some(stop_tx);
    Ok(())
}

/// Stop the speech-to-text engine.
pub async fn stop() -> Result<(), String> {
    let mut guard = state().lock().await;
    if !guard.active {
        return Ok(());
    }
    if let Some(stop_tx) = guard.stop_tx.take() {
        let _ = stop_tx.send(());
    }
    guard.stream = None;
    guard.active = false;
    Ok(())
}

/// List available audio input devices.
pub async fn list_devices() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(|| {
        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .map_err(|e| e.to_string())?
            .filter_map(|device| device.name().ok())
            .collect::<Vec<_>>();
        Ok(devices)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Set the active audio input device name.
pub async fn set_device(name: String) -> Result<(), String> {
    let mut guard = state().lock().await;
    guard.device_name = if name.trim().is_empty() {
        None
    } else {
        Some(name)
    };
    Ok(())
}

/// Set the active STT model path.
pub async fn set_model_path(path: String) -> Result<(), String> {
    let mut guard = state().lock().await;
    guard.model_path = if path.trim().is_empty() {
        None
    } else {
        Some(path)
    };
    Ok(())
}

fn select_device(name: &Option<String>) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    if let Some(target) = name {
        for device in host
            .input_devices()
            .map_err(|e| e.to_string())?
            .filter_map(|device| device.name().ok().map(|n| (n, device)))
        {
            if device.0 == *target {
                return Ok(device.1);
            }
        }
    }
    host.default_input_device()
        .ok_or_else(|| "No audio input device found.".to_string())
}

fn build_stream(
    device: cpal::Device,
    sample_format: SampleFormat,
    channels: usize,
    tx: mpsc::Sender<Vec<f32>>,
) -> Result<SendStream, String> {
    let config = device
        .default_input_config()
        .map_err(|e| e.to_string())?
        .config();
    let err_fn = |err: cpal::StreamError| eprintln!("audio stream error: {err}");
    let stream = match sample_format {
        SampleFormat::F32 => {
            let tx = tx;
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let _ = tx.try_send(interleave_to_mono(data, channels));
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            let tx = tx;
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let as_f32: Vec<f32> =
                        data.iter().map(|s| *s as f32 / i16::MAX as f32).collect();
                    let _ = tx.try_send(interleave_to_mono(&as_f32, channels));
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let tx = tx;
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let as_f32: Vec<f32> = data
                        .iter()
                        .map(|s| (*s as f32 / u16::MAX as f32) - 0.5)
                        .collect();
                    let _ = tx.try_send(interleave_to_mono(&as_f32, channels));
                },
                err_fn,
                None,
            )
        }
        _ => return Err("Unsupported sample format.".to_string()),
    }
    .map_err(|e| e.to_string())?;
    Ok(SendStream(stream))
}

async fn run_pipeline(
    app: tauri::AppHandle,
    vault_path: String,
    model_path: String,
    sample_rate: u32,
    _channels: usize,
    mut rx: mpsc::Receiver<Vec<f32>>,
    mut stop_rx: oneshot::Receiver<()>,
) {
    let mut buffer: VecDeque<f32> = VecDeque::new();
    let mut last_speech = tokio::time::Instant::now();
    let context = match WhisperContext::new_with_params(
        &model_path,
        whisper_rs::WhisperContextParameters::default(),
    ) {
        Ok(ctx) => ctx,
        Err(err) => {
            let _ = app.emit(
                "stt-text-chunk",
                SttEvent {
                    text: format!("STT error: {err}"),
                    is_final: true,
                    audio_path: None,
                },
            );
            return;
        }
    };
    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                break;
            }
            maybe_chunk = rx.recv() => {
                if let Some(chunk) = maybe_chunk {
                    buffer.extend(chunk);
                }
            }
        }

        if buffer.len() < (sample_rate as usize / 2) {
            continue;
        }

        let audio: Vec<f32> = buffer.drain(..).collect();
        let resampled = resample_linear(&audio, sample_rate, 16_000);
        if rms(&resampled) < 0.01 {
            continue;
        }
        last_speech = tokio::time::Instant::now();
        let transcript = match context.create_state().and_then(|mut state| {
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_language(Some("en"));
            params.set_translate(false);
            params.set_n_threads(4);
            state.full(params, &resampled)?;
            let mut text = String::new();
            let n_segments = state.full_n_segments().unwrap_or(0);
            for i in 0..n_segments {
                if let Ok(segment_text) = state.full_get_segment_text(i) {
                    text.push_str(&segment_text);
                }
            }
            Ok::<_, whisper_rs::WhisperError>(text)
        }) {
            Ok(text) => text,
            Err(_) => String::new(),
        };

        if !transcript.trim().is_empty() {
            let audio_path = save_audio_segment(&vault_path, &resampled).await.ok();
            let _ = app.emit(
                "stt-text-chunk",
                SttEvent {
                    text: transcript.trim().to_string(),
                    is_final: true,
                    audio_path,
                },
            );
        }

        if tokio::time::Instant::now().duration_since(last_speech) > Duration::from_secs(2) {
            tokio::time::sleep(Duration::from_millis(120)).await;
        }
    }
}

fn interleave_to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    let mut mono = Vec::with_capacity(data.len() / channels);
    for frame in data.chunks(channels) {
        let sum: f32 = frame.iter().sum();
        mono.push(sum / channels as f32);
    }
    mono
}

fn rms(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum: f32 = data.iter().map(|v| v * v).sum();
    (sum / data.len() as f32).sqrt()
}

fn resample_linear(input: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input_rate == output_rate || input.is_empty() {
        return input.to_vec();
    }
    let ratio = output_rate as f32 / input_rate as f32;
    let output_len = (input.len() as f32 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);
    for i in 0..output_len {
        let pos = i as f32 / ratio;
        let idx = pos.floor() as usize;
        let frac = pos - idx as f32;
        let a = input.get(idx).copied().unwrap_or(0.0);
        let b = input.get(idx + 1).copied().unwrap_or(a);
        output.push(a + (b - a) * frac);
    }
    output
}

async fn save_audio_segment(vault_path: &str, samples: &[f32]) -> Result<String, String> {
    let filename = format!("lecture-{}.wav", Utc::now().timestamp_millis());
    let path = Path::new(vault_path).join(".assets/audio").join(filename);
    fs_manager::ensure_vault_structure(vault_path).await?;
    let target = path.clone();
    let samples = samples.to_vec();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: WavSampleFormat::Int,
        };
        let mut writer =
            WavWriter::create(&target, spec).map_err(|e: hound::Error| e.to_string())?;
        for sample in samples {
            let value = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            writer
                .write_sample(value)
                .map_err(|e: hound::Error| e.to_string())?;
        }
        writer.finalize().map_err(|e: hound::Error| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(path.to_string_lossy().to_string())
}
