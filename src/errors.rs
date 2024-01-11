#[derive(Debug)]
pub enum CreateAudioStreamError {
    NoDefaultDevice,
    SupportedStreamConfigs(cpal::SupportedStreamConfigsError),
    NoSupportedConfigs,
    BuildStream(cpal::BuildStreamError),
}

impl From<cpal::SupportedStreamConfigsError> for CreateAudioStreamError {
    fn from(value: cpal::SupportedStreamConfigsError) -> Self {
        CreateAudioStreamError::SupportedStreamConfigs(value)
    }
}

impl From<cpal::BuildStreamError> for CreateAudioStreamError {
    fn from(value: cpal::BuildStreamError) -> Self {
        CreateAudioStreamError::BuildStream(value)
    }
}
