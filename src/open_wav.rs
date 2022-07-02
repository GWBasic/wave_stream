use crate::SampleFormat;

pub trait OpenWav {
    fn sample_format(&self) -> SampleFormat;
    fn channels(&self) -> u16;
    fn sample_rate(&self) -> u32;
    fn bits_per_sample(&self) -> u16;
    fn bytes_per_sample(&self) -> u16;
    fn len_samples(&self) -> u32;
}