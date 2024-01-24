use cpal::{
    self,
    traits::{DeviceTrait, HostTrait},
    Device, SampleFormat, SizedSample, Stream, StreamConfig,
};
use dasp_sample::{FromSample, ToSample};
use log::{error, warn};

use crate::{
    errors::CreateAudioStreamError, sample_ring_buffer::BufferHandle, util::write_silence,
};

pub fn create_default_input_stream(sink: BufferHandle) -> Result<Stream, CreateAudioStreamError> {
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
        SampleFormat::U8 => build_input_stream::<u8>(sink, &device, &config),
        SampleFormat::F32 => build_input_stream::<f32>(sink, &device, &config),
        _ => panic!("Unsupported input sample format: {sample_format}"),
    }
}

pub fn create_default_output_stream(
    source: BufferHandle,
) -> Result<Stream, CreateAudioStreamError> {
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
        SampleFormat::U8 => build_output_stream::<u8>(source, &device, &config),
        SampleFormat::F32 => build_output_stream::<f32>(source, &device, &config),
        _ => panic!("Unsupported output sample format: {sample_format}"),
    }
}

fn build_input_stream<T: SizedSample + ToSample<u8>>(
    sink: BufferHandle,
    device: &Device,
    config: &StreamConfig,
) -> Result<Stream, CreateAudioStreamError> {
    let data_fn = move |data: &[T], _: &_| {
        if data.is_empty() {
            warn!("CPAL provided empty input slice");
            return;
        }

        match sink.lock() {
            Ok(mut locked) => {
                locked.push_all_with_transform(data.iter(), |value| value.to_sample());
            }
            Err(e) => error!("Failed to push input packet of size {}: {}", data.len(), e),
        };
    };

    let err_fn = |err| error!("Error reading from input stream: {err}");

    Ok(device.build_input_stream(config, data_fn, err_fn, None)?)
}

fn build_output_stream<T: SizedSample + FromSample<u8>>(
    source: BufferHandle,
    device: &Device,
    config: &StreamConfig,
) -> Result<Stream, CreateAudioStreamError> {
    let data_fn = move |data: &mut [T], _: &_| {
        if data.is_empty() {
            warn!("CPAL provided empty output slice");
            return;
        }

        write_silence(data);

        match source.lock() {
            Ok(mut locked) => {
                locked.pop_into_with_transform(data.iter_mut(), |sample| T::from_sample(*sample));
            }
            Err(e) => error!("Error reading output packet: {e}"),
        };
    };

    let err_fn = |err| error!("Error writing to output stream: {err}");

    Ok(device.build_output_stream(config, data_fn, err_fn, None)?)
}
