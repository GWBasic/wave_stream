use crate::{wave_header::Channels, SampleFormat};

/// Represents an open wav file
pub trait OpenWav {
    // The sample format
    fn sample_format(&self) -> SampleFormat;
    // The number of channels
    fn num_channels(&self) -> u16;
    // The channels that the wav contains
    fn channels(&self) -> &Channels;
    /// The samples per second
    fn sample_rate(&self) -> u32;
    /// The bits per sample
    fn bits_per_sample(&self) -> u16;
    /// The bytes per sample
    fn bytes_per_sample(&self) -> u16;
    /// The total number of samples in the wav file
    fn len_samples(&self) -> usize;
}
