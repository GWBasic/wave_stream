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

/// Wav file header. Used to specify wav parameters when creating a wav, or to query wav parameters when reading a wav
pub struct WavHeader {
    /// The sample format
    pub sample_format: SampleFormat,
    /// The number of channels
    pub channels: u16,
    /// The sample rate
    pub sample_rate: u32,
    // Note: This may be needed to signal that a wav is an oddball bits per second: 12, 20, ect
    // (Samples are always aligned on the byte, IE, that's why 8-bit, 16-bit, and 24-bit int, and 32-bit float are supported)
    //pub bits_per_sample: u16
}

impl WavHeader {
    /// Reads a header from a Read struct
    ///
    /// # Arguments
    ///
    /// * 'reader' - A Read struct. It is strongly recommended that this struct implement some form of buffering, such as via a BufReader
    /// * 'subchunk_size' - Out value, set to the size of the header, or undefined if there is an IO error
    pub fn from_reader(reader: &mut impl Read, subchunk_size: &mut u32) -> Result<WavHeader> {
        reader.assert_str("fmt ", ErrorKind::Unsupported, "Not a WAVE file")?;

        *subchunk_size = reader.read_u32()?;
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
        if !(audio_format == 1 || audio_format == 3) {
            return Err(Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported audio format: {}", subchunk_size),
            ));
        }

        let channels = reader.read_u16()?; // 4
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

        Ok(WavHeader {
            sample_format,
            channels,
            sample_rate,
        })
    }

    /// Writes a header to a Write stuct
    ///
    /// # Arguments
    ///
    /// * 'writer' - The Write struct to write the wav header into
    pub fn to_writer(writer: &mut impl Write, header: &WavHeader) -> Result<()> {
        writer.write(b"fmt ")?;
        writer.write_u32(16)?;

        let audio_format: u16 = match header.sample_format {
            SampleFormat::Float => 3,
            _ => 1,
        };

        writer.write_u16(audio_format)?;
        writer.write_u16(header.channels)?;
        writer.write_u32(header.sample_rate)?;

        let bytes_per_sample: u16 = match header.sample_format {
            SampleFormat::Float => 4,
            SampleFormat::Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1,
        };

        let bytes_per_sec: u32 = header.sample_rate * ((header.channels * bytes_per_sample) as u32);
        writer.write_u32(bytes_per_sec)?;

        let data_block_size: u16 = (header.channels as u16) * (bytes_per_sample as u16);
        writer.write_u16(data_block_size)?;

        let bits_per_sample: u16 = bytes_per_sample * 8;
        writer.write_u16(bits_per_sample)?;

        Ok(())
    }
}
