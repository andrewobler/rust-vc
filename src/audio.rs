use std::{cmp::min, sync::mpsc};

use cpal::{
    self,
    traits::{DeviceTrait, HostTrait},
};
use log::{error, warn};

use crate::errors::CreateAudioStreamError;

pub fn create_default_input_stream(
    sink: mpsc::Sender<Vec<u8>>,
) -> Result<cpal::Stream, CreateAudioStreamError> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .ok_or(CreateAudioStreamError::NoDefaultDevice)?;

    let mut supported_configs = device.supported_input_configs()?;

    let supported_config = supported_configs
        .next()
        .ok_or(CreateAudioStreamError::NoSupportedConfigs)?;

    let sample_format = supported_config.sample_format();

    let config = supported_config.with_max_sample_rate().into();

    match sample_format {
        cpal::SampleFormat::U8 => build_input_stream::<u8>(sink, &device, &config),
        cpal::SampleFormat::F32 => build_input_stream::<f32>(sink, &device, &config),
        _ => panic!("Unsupported input sample format: {sample_format}"),
    }
}

pub fn create_default_output_stream(
    source: mpsc::Receiver<Vec<u8>>,
) -> Result<cpal::Stream, CreateAudioStreamError> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or(CreateAudioStreamError::NoDefaultDevice)?;

    let mut supported_configs = device.supported_output_configs()?;
    let supported_config = supported_configs
        .next()
        .ok_or(CreateAudioStreamError::NoSupportedConfigs)?;

    let sample_format = supported_config.sample_format();

    let config = supported_config.with_max_sample_rate().into();

    match sample_format {
        cpal::SampleFormat::U8 => build_output_stream::<u8>(source, &device, &config),
        cpal::SampleFormat::F32 => build_output_stream::<f32>(source, &device, &config),
        _ => panic!("Unsupported output sample format: {sample_format}"),
    }
}

fn build_input_stream<T: cpal::SizedSample + dasp_sample::ToSample<u8>>(
    sink: mpsc::Sender<Vec<u8>>,
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<cpal::Stream, CreateAudioStreamError> {
    let data_fn = move |data: &[T], _: &_| {
        if data.is_empty() {
            warn!("CPAL provided empty input slice");
            return;
        }

        let normalized = data.iter().map(|value| value.to_sample()).collect();

        if let Err(e) = sink.send(normalized) {
            error!("Failed to send input packet of size {}", e.0.len());
        }
    };

    let err_fn = |err| error!("Error reading from input stream: {err}");

    Ok(device.build_input_stream(config, data_fn, err_fn, None)?)
}

fn build_output_stream<T: cpal::SizedSample + dasp_sample::FromSample<u8>>(
    source: mpsc::Receiver<Vec<u8>>,
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<cpal::Stream, CreateAudioStreamError> {
    let data_fn = move |data: &mut [T], _: &_| {
        if data.is_empty() {
            warn!("CPAL provided empty output slice");
            return;
        }

        match source.try_recv() {
            Ok(vec) => {
                let limit = min(data.len(), vec.len());
                let (to_fill, rest) = data.split_at_mut(limit);
                for (dst, src) in to_fill.iter_mut().zip(vec.iter()) {
                    *dst = T::from_sample(*src);
                }

                write_silence(rest);
            }
            Err(mpsc::TryRecvError::Empty) => {
                warn!("No available input in source");
                write_silence(data);
            }
            Err(mpsc::TryRecvError::Disconnected) => error!("Input source disconnected"),
        };
    };

    let err_fn = |err| error!("Error writing to output stream: {err}");

    Ok(device.build_output_stream(config, data_fn, err_fn, None)?)
}

fn write_silence<T: cpal::SizedSample>(data: &mut [T]) {
    for sample in data.iter_mut() {
        *sample = cpal::Sample::EQUILIBRIUM;
    }
}
