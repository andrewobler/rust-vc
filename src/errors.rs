use cpal::{BuildStreamError, SupportedStreamConfigsError};

#[derive(Debug)]
pub enum CreateAudioStreamError {
    NoDefaultDevice,
    SupportedStreamConfigs(SupportedStreamConfigsError),
    NoSupportedConfigs,
    BuildStream(BuildStreamError),
}

impl From<SupportedStreamConfigsError> for CreateAudioStreamError {
    fn from(value: SupportedStreamConfigsError) -> Self {
        CreateAudioStreamError::SupportedStreamConfigs(value)
    }
}

impl From<cpal::BuildStreamError> for CreateAudioStreamError {
    fn from(value: BuildStreamError) -> Self {
        CreateAudioStreamError::BuildStream(value)
    }
}
