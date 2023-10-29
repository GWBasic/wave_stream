// Influenced by https://github.com/kujirahand/wav_io/blob/main/src/header.rs

use std::io::{Error, ErrorKind, Read, Result, Write};

use crate::{ReadEx, WriteEx};

/// Sample Format, sample bit depth
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SampleFormat {
    /// 8-bit. Audio quality equivalent to a cassette without noise reduction. Noise shaping and/or dithering is needed
    /// for acceptable audio quality.
    Int8,
    /// 16-bit. Same as audio CD. Quantization will be noticible on quiet sounds, unless noise shaping and/or dithering
    /// is used.
    Int16,
    /// 24-bit. Generally exceeds the range of human hearing, except when played at levels that exceed the threshold of pain
    Int24,
    /// Floating point. Generally exceeds the range of human hearing. Recommended when additional processing is anticipated
    Float,
}

pub trait SampleFormatSize {
    /// The number of bits in each sample
    fn bits_per_sample(&self) -> u16;

    /// The number of bytes in each sample
    fn bytes_per_sample(&self) -> u16;
}

impl SampleFormatSize for SampleFormat {
    fn bits_per_sample(&self) -> u16 {
        self.bytes_per_sample() * 8
    }

    fn bytes_per_sample(&self) -> u16 {
        match self {
            SampleFormat::Float => 4,
            SampleFormat::Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1,
        }
    }
}

// Flags of all of the channels present in the file
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Channels {
    pub front_left: bool,
    pub front_right: bool,
    pub front_center: bool,
    pub low_frequency: bool,
    pub back_left: bool,
    pub back_right: bool,
    pub front_left_of_center: bool,
    pub front_right_of_center: bool,
    pub back_center: bool,
    pub side_left: bool,
    pub side_right: bool,
    pub top_center: bool,
    pub top_front_left: bool,
    pub top_front_center: bool,
    pub top_front_right: bool,
    pub top_back_left: bool,
    pub top_back_center: bool,
    pub top_back_right: bool,
}

impl Channels {
    pub fn new() -> Channels {
        Channels {
            front_left: false,
            front_right: false,
            front_center: false,
            low_frequency: false,
            back_left: false,
            back_right: false,
            front_left_of_center: false,
            front_right_of_center: false,
            back_center: false,
            side_left: false,
            side_right: false,
            top_center: false,
            top_front_left: false,
            top_front_center: false,
            top_front_right: false,
            top_back_left: false,
            top_back_center: false,
            top_back_right: false,
        }
    }

    pub fn front_left(mut self) -> Channels {
        self.front_left = true;

        self
    }

    pub fn front_right(mut self) -> Channels {
        self.front_right = true;

        self
    }

    pub fn front_center(mut self) -> Channels {
        self.front_center = true;

        self
    }

    pub fn low_frequency(mut self) -> Channels {
        self.low_frequency = true;

        self
    }

    pub fn back_left(mut self) -> Channels {
        self.back_left = true;

        self
    }

    pub fn back_right(mut self) -> Channels {
        self.back_right = true;

        self
    }

    pub fn front_left_of_center(mut self) -> Channels {
        self.front_left_of_center = true;

        self
    }

    pub fn front_right_of_center(mut self) -> Channels {
        self.front_right_of_center = true;

        self
    }

    pub fn back_center(mut self) -> Channels {
        self.back_center = true;

        self
    }

    pub fn side_left(mut self) -> Channels {
        self.side_left = true;

        self
    }

    pub fn side_right(mut self) -> Channels {
        self.side_right = true;

        self
    }

    pub fn top_center(mut self) -> Channels {
        self.top_center = true;

        self
    }

    pub fn top_front_left(mut self) -> Channels {
        self.top_front_left = true;

        self
    }

    pub fn top_front_center(mut self) -> Channels {
        self.top_front_center = true;

        self
    }

    pub fn top_front_right(mut self) -> Channels {
        self.top_front_right = true;

        self
    }

    pub fn top_back_left(mut self) -> Channels {
        self.top_back_left = true;

        self
    }

    pub fn top_back_center(mut self) -> Channels {
        self.top_back_center = true;

        self
    }

    pub fn top_back_right(mut self) -> Channels {
        self.top_back_right = true;

        self
    }
}

// Wav file header. Used to specify wav parameters when creating a wav, or to query wav parameters when reading a wav
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WavHeader {
    /// The sample format
    pub sample_format: SampleFormat,
    /// The channels present in the file
    pub channels: Channels,
    /// The sample rate
    pub sample_rate: u32,
    // Note: This may be needed to signal that a wav is an oddball bits per second: 12, 20, ect
    // (Samples are always aligned on the byte, IE, that's why 8-bit, 16-bit, and 24-bit int, and 32-bit float are supported)
    //pub bits_per_sample: u16
    pub max_samples: usize,
}

impl WavHeader {
    /// Reads a header from a Read struct
    ///
    /// # Arguments
    ///
    /// * 'reader' - A Read struct. It is strongly recommended that this struct implement some form of buffering, such as via a BufReader
    /// * 'subchunk_size' - Out value, set to the size of the header, or undefined if there is an IO error
    pub fn from_reader(reader: &mut impl Read, subchunk_size: &mut usize) -> Result<WavHeader> {
        reader.assert_str("fmt ", ErrorKind::Unsupported, "Not a WAVE file")?;

        *subchunk_size = reader.read_u32()? as usize;
        if *subchunk_size < 16 {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Invalid header. fmt header must be size 16 or larger, actual value: {}",
                    subchunk_size
                ),
            ));
        }

        let audio_format = reader.read_u16()?; // 2

        if audio_format == 1 || audio_format == 3 {
            Self::from_reader_classic(reader, subchunk_size)
        // wFormatTag: WAVE_FORMAT_EXTENSIBLE, https://www.mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html
        } else if audio_format == 0xFFFE {
            Self::from_reader_extensible(reader, subchunk_size)
        } else {
            Err(Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported audio format: {}", audio_format),
            ))
        }
    }

    fn from_reader_classic(reader: &mut impl Read, subchunk_size: &mut usize) -> Result<WavHeader> {
        let num_channels = reader.read_u16()?; // 4
        let sample_rate = reader.read_u32()?; // 8

        let _bytes_per_sec = reader.read_u32()?; // 12
        let _data_block_size = reader.read_u16()?; // 14

        // This supports oddball situations, like 12-bit, or 20-bit
        // Normally, those are rounded up with least-significant-bit 0ed out
        // (12-bit written as 16-bit, 20-bit written as 24-bit)
        let bits_per_sample = reader.read_u16()?; // 16
        let sample_format = if bits_per_sample == 32 {
            SampleFormat::Float
        } else if bits_per_sample <= 8 {
            SampleFormat::Int8
        } else if bits_per_sample <= 16 {
            SampleFormat::Int16
        } else if bits_per_sample <= 24 {
            SampleFormat::Int24
        } else {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("{} bits per sample unsupported", bits_per_sample),
            ));
        };

        // Skip additional ignored headers
        // (By now we're read 16 bytes)
        reader.skip((*subchunk_size - 16) as usize)?;

        let channels = Channels {
            front_left: num_channels >= 1,
            front_right: num_channels >= 2,
            front_center: num_channels >= 3,
            low_frequency: num_channels >= 4,
            back_left: num_channels >= 5,
            back_right: num_channels >= 6,
            front_left_of_center: num_channels >= 7,
            front_right_of_center: num_channels >= 8,
            back_center: num_channels >= 9,
            side_left: num_channels >= 10,
            side_right: num_channels >= 11,
            top_center: num_channels >= 12,
            top_front_left: num_channels >= 13,
            top_front_center: num_channels >= 14,
            top_front_right: num_channels >= 15,
            top_back_left: num_channels >= 16,
            top_back_center: num_channels >= 17,
            top_back_right: num_channels >= 18,
        };

        let max_samples = calculate_max_samples(&channels, sample_format);

        Ok(WavHeader {
            sample_format,
            channels,
            sample_rate,
            max_samples,
        })
    }

    fn from_reader_extensible(
        reader: &mut impl Read,
        subchunk_size: &mut usize,
    ) -> Result<WavHeader> {
        let num_channels = reader.read_u16()?; // 4
        let sample_rate = reader.read_u32()?; // 8

        let _bytes_per_sec = reader.read_u32()?; // 12
        let _data_block_size = reader.read_u16()?; // 14

        // This supports oddball situations, like 12-bit, or 20-bit
        // Normally, those are rounded up with least-significant-bit 0ed out
        // (12-bit written as 16-bit, 20-bit written as 24-bit)
        let bits_per_sample = reader.read_u16()?; // 16
        let sample_format = if bits_per_sample == 32 {
            SampleFormat::Float
        } else if bits_per_sample <= 8 {
            SampleFormat::Int8
        } else if bits_per_sample <= 16 {
            SampleFormat::Int16
        } else if bits_per_sample <= 24 {
            SampleFormat::Int24
        } else {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("{} bits per sample unsupported", bits_per_sample),
            ));
        };

        // Ignore cbSize
        let _cb_size = reader.read_u16()?;

        // Ignore wValidBitsPerSample
        let _w_valid_bits_per_sample = reader.read_u16()?;

        let channel_mask = reader.read_u32()?;

        // Skip additional ignored headers
        // (By now we're read 24 bytes)
        reader.skip((*subchunk_size - 24) as usize)?;

        let channels = Channels {
            front_left: channel_mask & 0x1 == 0x1,
            front_right: channel_mask & 0x2 == 0x2,
            front_center: channel_mask & 0x4 == 0x4,
            low_frequency: channel_mask & 0x8 == 0x8,
            back_left: channel_mask & 0x10 == 0x10,
            back_right: channel_mask & 0x20 == 0x20,
            front_left_of_center: channel_mask & 0x40 == 0x40,
            front_right_of_center: channel_mask & 0x80 == 0x80,
            back_center: channel_mask & 0x100 == 0x100,
            side_left: channel_mask & 0x200 == 0x200,
            side_right: channel_mask & 0x400 == 0x400,
            top_center: channel_mask & 0x800 == 0x800,
            top_front_left: channel_mask & 0x1000 == 0x1000,
            top_front_center: channel_mask & 0x2000 == 0x2000,
            top_front_right: channel_mask & 0x4000 == 0x4000,
            top_back_left: channel_mask & 0x8000 == 0x8000,
            top_back_center: channel_mask & 0x10000 == 0x10000,
            top_back_right: channel_mask & 0x20000 == 0x20000,
        };

        if num_channels != channels.count() {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Mismatch between number of channels specified in the header, and channel mask",
            ));
        }

        let max_samples = calculate_max_samples(&channels, sample_format);

        Ok(WavHeader {
            sample_format,
            channels,
            sample_rate,
            max_samples,
        })
    }

    /// Writes a header to a Write stuct
    ///
    /// # Arguments
    ///
    /// * 'writer' - The Write struct to write the wav header into
    pub fn to_writer(writer: &mut impl Write, header: &WavHeader) -> Result<()> {
        let num_channels = header.channels.count();

        // Write WAVEFORMATEX
        writer.write(b"fmt ")?;
        writer.write_u32(18 + 22)?;

        // wFormatTag: WAVE_FORMAT_EXTENSIBLE, https://www.mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html
        writer.write_u16(0xFFFE)?;
        // nChannels
        writer.write_u16(num_channels)?;
        // nSamplesPerSec
        writer.write_u32(header.sample_rate)?;

        let bytes_per_sample: u16 = match header.sample_format {
            SampleFormat::Float => 4,
            SampleFormat::Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1,
        };

        // nAvgBytesPerSec
        let bytes_per_sec: u32 = header.sample_rate * ((num_channels * bytes_per_sample) as u32);
        writer.write_u32(bytes_per_sec)?;

        // nBlockAlign
        let data_block_size: u16 = (num_channels as u16) * (bytes_per_sample as u16);
        writer.write_u16(data_block_size)?;

        // wBitsPerSample
        let bits_per_sample: u16 = bytes_per_sample * 8;
        writer.write_u16(bits_per_sample)?;

        // cbSize
        writer.write_u16(22)?;

        // wValidBitsPerSample
        writer.write_u16(bits_per_sample)?;

        // dwChannelMask
        writer.write_u32(header.channels.channel_mask())?;

        let audio_format: u16 = match header.sample_format {
            SampleFormat::Float => 3,
            _ => 1,
        };

        // SubFormat (See Extensible Format in https://www.mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html)
        writer.write_u16(audio_format)?;
        writer.write(b"\x00\x00\x00\x00\x10\x00\x80\x00\x00\xAA\x00\x38\x9B\x71")?;

        Ok(())
    }
}

pub fn calculate_max_samples(channels: &Channels, sample_format: SampleFormat) -> usize {
    let channels_count = channels.count() as u32;
    let bytes_per_sample = sample_format.bytes_per_sample() as u32;
    let max_samples = (u32::MAX - 32 + 8) / channels_count / bytes_per_sample;

    max_samples as usize
}

impl Channels {
    pub fn count(&self) -> u16 {
        let mut count = 0;

        if self.front_left {
            count += 1;
        }

        if self.front_right {
            count += 1;
        }

        if self.front_center {
            count += 1;
        }

        if self.low_frequency {
            count += 1;
        }

        if self.back_left {
            count += 1;
        }

        if self.back_right {
            count += 1;
        }

        if self.front_left_of_center {
            count += 1;
        }

        if self.front_right_of_center {
            count += 1;
        }

        if self.back_center {
            count += 1;
        }

        if self.side_left {
            count += 1;
        }

        if self.side_right {
            count += 1;
        }

        if self.top_center {
            count += 1;
        }

        if self.top_front_left {
            count += 1;
        }

        if self.top_front_center {
            count += 1;
        }

        if self.top_front_right {
            count += 1;
        }

        if self.top_back_left {
            count += 1;
        }

        if self.top_back_center {
            count += 1;
        }

        if self.top_back_right {
            count += 1;
        }

        count
    }

    pub fn channel_mask(&self) -> u32 {
        let mut channel_mask = 0;

        if self.front_left {
            channel_mask |= 0x1;
        }

        if self.front_right {
            channel_mask |= 0x2;
        }

        if self.front_center {
            channel_mask |= 0x4;
        }

        if self.low_frequency {
            channel_mask |= 0x8;
        }

        if self.back_left {
            channel_mask |= 0x10;
        }

        if self.back_right {
            channel_mask |= 0x20;
        }

        if self.front_left_of_center {
            channel_mask |= 0x40;
        }

        if self.front_right_of_center {
            channel_mask |= 0x80;
        }

        if self.back_center {
            channel_mask |= 0x100;
        }

        if self.side_left {
            channel_mask |= 0x200;
        }

        if self.side_right {
            channel_mask |= 0x400;
        }

        if self.top_center {
            channel_mask |= 0x800;
        }

        if self.top_front_left {
            channel_mask |= 0x1000;
        }

        if self.top_front_center {
            channel_mask |= 0x2000;
        }

        if self.top_front_right {
            channel_mask |= 0x4000;
        }

        if self.top_back_left {
            channel_mask |= 0x8000;
        }

        if self.top_back_center {
            channel_mask |= 0x10000;
        }

        if self.top_back_right {
            channel_mask |= 0x20000;
        }

        channel_mask
    }
}

#[cfg(test)]
mod tests {
    use crate::Channels;
    use crate::SampleFormat;
    use super::calculate_max_samples;

    #[test]
    fn calculate_max_samples_sanity() {
        let channels = Channels {
            front_left: true,
            front_right: true,
            front_center: true,
            low_frequency: true,
            back_left: true,
            back_right: true,
            front_left_of_center: true,
            front_right_of_center: true,
            back_center: true,
            side_left: true,
            side_right: true,
            top_center: true,
            top_front_left: true,
            top_front_center: true,
            top_front_right: true,
            top_back_left: true,
            top_back_center: true,
            top_back_right: true,
        };

        let max_samples = calculate_max_samples(&channels, SampleFormat::Float);

        // 4294967295
        // (u32::MAX - 32 + 8) / channels_count / bytes_per_sample;
        // 4294967271 / 18 / 4
        // 59652323
        assert_eq!(59652323, max_samples);

    }
}