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

    let config = supported_config.with_max_sample_rate().into();

    let data_fn = move |data: &[u8], _: &_| {
        if data.is_empty() {
            warn!("CPAL provided empty input slice");
            return;
        }

        if let Err(e) = sink.send(data.to_vec()) {
            error!("Failed to send input packet of size {}", e.0.len());
        }
    };

    let err_fn = |err| error!("Error reading from input stream: {}", err);

    let stream = device.build_input_stream(&config, data_fn, err_fn, None)?;

    Ok(stream)
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

    let config = supported_config.with_max_sample_rate().into();

    let data_fn = move |data: &mut [u8], _: &_| {
        if data.is_empty() {
            warn!("CPAL provided empty output slice");
            return;
        }

        match source.try_recv() {
            Ok(vec) => {
                let limit = min(data.len(), vec.len());
                let (to_fill, rest) = data.split_at_mut(limit);
                for (dst, src) in to_fill.iter_mut().zip(vec.iter()) {
                    *dst = *src;
                }

                write_silence(rest);
            }
            Err(mpsc::TryRecvError::Empty) => {
                warn!("No available input in source");
                write_silence(data);
            }
            Err(mpsc::TryRecvError::Disconnected) => error!("Input source disconnected"),
        }
    };

    let err_fn = |err| error!("Error writing to output stream: {}", err);

    let stream = device.build_output_stream(&config, data_fn, err_fn, None)?;

    Ok(stream)
}

fn write_silence(data: &mut [u8]) {
    for sample in data.iter_mut() {
        *sample = cpal::Sample::EQUILIBRIUM;
    }
}
