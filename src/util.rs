use cpal::SizedSample;
use dasp_sample::Sample;

pub fn write_silence<T: SizedSample>(data: &mut [T]) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}
