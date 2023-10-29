//! wave_stream is a library that allows reading and writing wave files. Random-access reading and writing
//! is supported, as well as reading via an enumerable and writing via an enumerable. Unlike other wave
//! libraries, the whole file is not sucked into memory.
//!
//! Read through the example below to understand how to read a wav's metadata, read the wav, and write a wav.
//!
//! It's generally reccomended that you read and write wav files as float (f32). (Unless you're cropping and
//! appending existing 16-bit, 24-bit, or 8-bit waves files.) wave_stream does not implement any way to
//! noise-shape or dither floating-point or 24-bit samples to 16-bit or 8-bit. (The author reccomends using
//! sox to convert floating-point wave files to lower bits-per-sample, as sox implements great noise shaping.)
//!
//! Note: The wav file format is limited to no more then 4GB. Wave_stream does not support proposed extensions
//! to the wav file format that exceed this limitation.
//!
//! # Example
//!
//! ``` rust
//! use std::f32::consts::{PI, TAU};
//! use std::io::Result;
//! use std::path::Path;
//!
//! use wave_stream::open_wav::OpenWav;
//! use wave_stream::samples_by_channel::SamplesByChannel;
//! use wave_stream::wave_header::{Channels, SampleFormat, WavHeader};
//! use wave_stream::wave_reader::{RandomAccessOpenWavReader, StreamOpenWavReader};
//! use wave_stream::{read_wav_from_file_path, write_wav_to_file_path};
//!
//! fn main() {
//!     let open_wav = read_wav_from_file_path(Path::new("some.wav")).unwrap();
//!
//!     // Inspect metadata
//!     // ******************************
//!     println!("Number of channels: {0}, samples per second: {1}, bits per sample: {2}, length in samples: {3}",
//!         open_wav.num_channels(),
//!         open_wav.bits_per_sample(),
//!         open_wav.sample_rate(),
//!         open_wav.len_samples());
//!
//!     // Read via random access
//!     // ******************************
//!     let mut random_access_wave_reader = open_wav.get_random_access_f32_reader().unwrap();
//!     let first_sample = random_access_wave_reader.read_sample(0).unwrap();
//!     println!(
//!         "First sample, front_left: {0}",
//!         first_sample.front_left.expect("front_left missing")
//!     );
//!
//!     // Read via an enumerable: Find the loudest sample in the wave file
//!     // ******************************
//!     let open_wav = read_wav_from_file_path(Path::new("some.wav")).unwrap();
//!     let mut loudest_sample = f32::MIN;
//!
//!     // Note that the wave is read as f32 values in this example.
//!     // Reading as 8-bit, (i8) 16-bit, (i16) and 24-bit (i32) is also supported.
//!     // Upsampling during reads is supported. (You can read an 8-bit wav file as f32)
//!     // Downsampling during reads isn't supported. (You can't read a floating point wav file as i8)
//!     //
//!     // In general:
//!     // - For audio manipulation: f32 is *strongly* reccomended
//!     // - Only use i16, i32, (or i8), when cutting existing audio without manipulation
//!     let iterator = open_wav.get_stream_f32_reader().unwrap().into_iter();
//!
//!     for samples_result in iterator {
//!         let samples = samples_result.unwrap();
//!
//!         for sample in samples.to_vec() {
//!             loudest_sample = f32::max(loudest_sample, sample);
//!         }
//!     }
//!
//!     println!("Loudest sample: {0}", loudest_sample);
//!
//!     // Write via random access
//!     // ******************************
//!     let sample_rate = 96000;
//!     let header = WavHeader {
//!         sample_format: SampleFormat::Float,
//!         channels: Channels::new()
//!             .front_left(),
//!         sample_rate,
//!     };
//!
//!     let open_wav = write_wav_to_file_path(Path::new("ramp.wav"), header).unwrap();
//!
//!     // Note that the wave is written as f32 (32-bit float). 8-bit (i8), 16-bit (i16), and 24-bit (i32) integer are
//!     // also supprted.
//!     // Downconverting (IE, float -> 16-bit) is *not* supported. In general, it's best to perform audio manipulation
//!     // using f32. Outputting to an integer format like 16-bit (CD quality) will only sound good if you implement your
//!     // own noise shaper or dithering algorithm. A command-line tool like sox will perform excellent noise shaping if
//!     // you write a 32-bit float wav, and then use sox to convert it to 16-bit.
//!     let mut random_access_wave_writer = open_wav.get_random_access_f32_writer().unwrap();
//!
//!     let samples_in_ramp = 2000;
//!     let samples_in_ramp_f32 = samples_in_ramp as f32;
//!     for sample in 0usize..((sample_rate * 3) as usize) {
//!         // Write 3 seconds of samples
//!         let modulo = (sample % samples_in_ramp) as f32;
//!         let sample_value = (2f32 * modulo / samples_in_ramp_f32) - 1f32;
//!         random_access_wave_writer
//!             .write_samples(
//!                 sample,
//!                 SamplesByChannel::new()
//!                     .front_left(sample_value))
//!             .unwrap();
//!     }
//!
//!     random_access_wave_writer.flush().unwrap();
//!
//!     // Write via iterator
//!     // ******************************
//!     let header = WavHeader {
//!         sample_format: SampleFormat::Float,
//!         channels: Channels::new()
//!             .front_left(),
//!         sample_rate,
//!     };
//!
//!     let open_wav = write_wav_to_file_path(Path::new("sine.wav"), header).unwrap();
//!     let sine_iterator = SineIterator {
//!         period: (sample_rate / 60) as f32,
//!         current_sample: PI, // Start at 0 crossing
//!     };
//!     let sine_iterator_three_seconds = sine_iterator.take((sample_rate * 3u32) as usize); // Write 3 seconds
//!     open_wav.write_all_f32(sine_iterator_three_seconds).unwrap();
//! }
//!
//! // Used when writing via iterator
//! struct SineIterator {
//!     period: f32,
//!     current_sample: f32,
//! }
//!
//! // Used when writing via iterator
//! impl Iterator for SineIterator {
//!     type Item = Result<SamplesByChannel<f32>>;
//!
//!     fn next(&mut self) -> Option<Result<SamplesByChannel<f32>>> {
//!         let result = (self.current_sample / self.period * TAU).sin();
//!         self.current_sample += 1f32;
//!
//!         if self.current_sample > self.period {
//!             self.current_sample = 0f32;
//!         }
//!
//!         return Some(Ok(SamplesByChannel::new()
//!             .front_left(result)));
//!     }
//! }
//!
//! ```

use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Result, Seek, Write};
use std::path::Path;

pub mod open_wav;
pub mod reader;
pub mod wave_header;
pub mod wave_reader;
pub mod wave_writer;
pub mod writer;

mod assertions;
mod constants;
pub mod samples_by_channel;
mod upconvert;

use reader::ReadEx;
use wave_header::*;
use wave_reader::*;
use wave_writer::*;
use writer::WriteEx;

/// Reads a wav from a given path
///
/// # Arguments
///
/// * 'file_path' - A Path that is the path to the wav file to read
///
pub fn read_wav_from_file_path(file_path: &Path) -> Result<OpenWavReader<BufReader<File>>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    read_wav(reader)
}

/// Reads a wav from a Read struct
///
/// # Arguments
///
/// * 'reader' - A Read struct. It is strongly recommended that this struct implement some form of buffering, such as via a BufReader
pub fn read_wav<TReader: 'static + Read>(mut reader: TReader) -> Result<OpenWavReader<TReader>> {
    // Verify that this is a RIFF file
    reader.assert_str(
        "RIFF",
        ErrorKind::InvalidInput,
        "Not a WAVE file (Missing RIFF Header)",
    )?;
    let _file_length = reader.read_u32()?;
    reader.assert_str(
        "WAVE",
        ErrorKind::Unsupported,
        "Not a WAVE file (Missing WAVE header)",
    )?;

    // file position is 12

    let mut subchunk_size = 0usize;
    let header = WavHeader::from_reader(&mut reader, &mut subchunk_size)?;

    // subchunk size doesn't include 4-letter prefix and 4-byte length

    OpenWavReader::new(reader, header, 20 + subchunk_size)
}

/// Starts writing a wav to a Path. Returns an OpenWavWriter struct that is used to write the contents of the wav
///
/// # Arguments
///
/// * 'file_path' - The path to where the wav will be written
/// * 'header' - The header information in the wav. This specifies things like sampling rate, sample bit depth, ect
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// use wave_stream::samples_by_channel::SamplesByChannel;
/// use wave_stream::wave_header::{Channels, SampleFormat, WavHeader};
/// use wave_stream::{read_wav_from_file_path, write_wav_to_file_path};
///
/// let header = WavHeader {
///     sample_format: SampleFormat::Float,
///     channels: Channels {
///             front_left: true,
///             front_right: true,
///             front_center: false,
///             low_frequency: false,
///             back_left: false,
///             back_right: false,
///             front_left_of_center: false,
///             front_right_of_center: false,
///             back_center: false,
///             side_left: false,
///             side_right: false,
///             top_center: false,
///             top_front_left: false,
///             top_front_center: false,
///             top_front_right: false,
///             top_back_left: false,
///             top_back_center: false,
///             top_back_right: false,
///         },
///     sample_rate: 96000,
/// };
/// let mut open_wav = write_wav_to_file_path(Path::new("some.wav"), header).unwrap();
/// let mut writer = open_wav.get_random_access_f32_writer().unwrap();
///
/// // Sample 0
/// writer.write_samples(0, SamplesByChannel {
///             front_left: Some(0.0),
///             front_right: Some(0.0),
///             front_center: None,
///             low_frequency: None,
///             back_left: None,
///             back_right: None,
///             front_left_of_center: None,
///             front_right_of_center: None,
///             back_center: None,
///             side_left: None,
///             side_right: None,
///             top_center: None,
///             top_front_left: None,
///             top_front_center: None,
///             top_front_right: None,
///             top_back_left: None,
///             top_back_center: None,
///             top_back_right: None,
///         }).unwrap();
///
/// // Sample 1
/// writer.write_samples(1, SamplesByChannel {
///             front_left: Some(0.0),
///             front_right: Some(0.0),
///             front_center: None,
///             low_frequency: None,
///             back_left: None,
///             back_right: None,
///             front_left_of_center: None,
///             front_right_of_center: None,
///             back_center: None,
///             side_left: None,
///             side_right: None,
///             top_center: None,
///             top_front_left: None,
///             top_front_center: None,
///             top_front_right: None,
///             top_back_left: None,
///             top_back_center: None,
///             top_back_right: None,
///         }).unwrap();
///
/// // Sample 2
/// writer.write_samples(2, SamplesByChannel {
///             front_left: Some(0.0),
///             front_right: Some(0.0),
///             front_center: None,
///             low_frequency: None,
///             back_left: None,
///             back_right: None,
///             front_left_of_center: None,
///             front_right_of_center: None,
///             back_center: None,
///             side_left: None,
///             side_right: None,
///             top_center: None,
///             top_front_left: None,
///             top_front_center: None,
///             top_front_right: None,
///             top_back_left: None,
///             top_back_center: None,
///             top_back_right: None,
///         }).unwrap();
///
/// writer.flush().unwrap();
/// ```
pub fn write_wav_to_file_path(file_path: &Path, header: WavHeader) -> Result<OpenWavWriter> {
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    write_wav(writer, header)
}

/// Starts writing a wav to a (Write + Seek) struct. Returns an OpenWavWriter struct that is used to write the contents of the wav
///
/// # Arguments
///
/// * 'writer' - The (Write + Seek) struct to write the wav into. It is strongly recommended that this struct implement some form of buffering, such as via a BufWriter
/// * 'header' - The header information in the wav. This specifies things like sampling rate, sample bit depth, ect
pub fn write_wav<TWriter: 'static + Write + Seek>(
    mut writer: TWriter,
    header: WavHeader,
) -> Result<OpenWavWriter> {
    // Write RIFF header and format
    writer.write(b"RIFF    WAVE")?;

    WavHeader::to_writer(&mut writer, &header)?;

    OpenWavWriter::new(writer, header)
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::i8;
    use std::io::Take;

    use tempfile::tempdir;

    use super::*;
    use crate::open_wav::OpenWav;
    use crate::samples_by_channel::SamplesByChannel;
    use crate::upconvert::{
        INT_16_DIVIDE_FOR_FLOAT, INT_24_DIVIDE_FOR_FLOAT, INT_8_ADD_FOR_FLOAT_ABS,
        INT_8_DIVIDE_FOR_FLOAT,
    };

    #[test]
    fn open_sanity() {
        let open_wav = read_wav_from_file_path(Path::new("test_data/short_float.wav")).unwrap();
        assert_eq!(SampleFormat::Float, open_wav.sample_format());
        assert_eq!(1, open_wav.num_channels());
        assert_eq!(32, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = read_wav_from_file_path(Path::new("test_data/short_24.wav")).unwrap();
        assert_eq!(SampleFormat::Int24, open_wav.sample_format());
        assert_eq!(1, open_wav.num_channels());
        assert_eq!(24, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = read_wav_from_file_path(Path::new("test_data/short_16.wav")).unwrap();
        assert_eq!(SampleFormat::Int16, open_wav.sample_format());
        assert_eq!(1, open_wav.num_channels());
        assert_eq!(16, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());
    }

    #[test]
    fn read_float_sanity() {
        let open_wav = read_wav_from_file_path(Path::new("test_data/short_float.wav")).unwrap();
        let wave_reader_float = open_wav.get_random_access_f32_reader().unwrap();
        assert_eq!(
            SampleFormat::Float,
            wave_reader_float.info().sample_format()
        );
        assert_eq!(1, wave_reader_float.info().num_channels());
        assert_eq!(32, wave_reader_float.info().bits_per_sample());
        assert_eq!(48000, wave_reader_float.info().sample_rate());
        assert_eq!(1267, wave_reader_float.info().len_samples());
    }

    #[test]
    fn read_random_i8() {
        read_random(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i8_reader()),
            i8::from_le_bytes([0x7D]),
            i8::from_le_bytes([0x7F]),
            i8::from_le_bytes([0x7A]),
        )
        .unwrap();
    }

    #[test]
    fn read_random_i8_as_i16() {
        read_random(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
            32255,
            32767,
            31487,
        )
        .unwrap();
    }

    #[test]
    fn read_random_i8_as_i24() {
        read_random(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            8257535,
            8388607,
            8060927,
        )
        .unwrap();
    }

    #[test]
    fn read_random_i8_as_f32() {
        read_random(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            0.9843137,
            1.0,
            0.9607843,
        )
        .unwrap();
    }

    #[test]
    fn read_random_i16() {
        read_random(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
            i16::from_le_bytes([0x61, 0xFD]),
            i16::from_le_bytes([0xF9, 0xFD]),
            i16::from_le_bytes([0x9C, 0xFE]),
        )
        .unwrap();
    }

    #[test]
    fn read_random_i16_as_i24() {
        read_random(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            -171776,
            -132864,
            -91136,
        )
        .unwrap();
    }

    #[test]
    fn read_random_i16_as_f32() {
        read_random(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            -0.020462334,
            -0.015823603,
            -0.010849178,
        )
        .unwrap();
    }

    #[test]
    fn read_random_i24() {
        read_random(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            i32::from_le_bytes([0x00, 0x2E, 0x61, 0xFD]) >> 8,
            i32::from_le_bytes([0x00, 0xE7, 0xF8, 0xFD]) >> 8,
            i32::from_le_bytes([0x00, 0x94, 0x9C, 0xFE]) >> 8,
        )
        .unwrap();
    }

    #[test]
    fn read_random_i24_as_f32() {
        read_random(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            -0.020471752,
            -0.015841544,
            -0.010846555,
        )
        .unwrap();
    }

    #[test]
    fn read_random_f32() {
        read_random(
            Path::new("test_data/short_float.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            f32::from_le_bytes([0x6D, 0xB4, 0xA7, 0xBC]),
            f32::from_le_bytes([0x02, 0xC6, 0x81, 0xBC]),
            f32::from_le_bytes([0xA0, 0xB5, 0x31, 0xBC]),
        )
        .unwrap();
    }

    fn read_random<T: Debug + PartialEq>(
        path: &Path,
        get_random_access_reader: Box<
            dyn FnOnce(OpenWavReader<BufReader<File>>) -> Result<RandomAccessWavReader<T>>,
        >,
        expected_sample_0: T,
        expected_sample_1: T,
        expected_sample_end: T,
    ) -> Result<()> {
        let open_wav = read_wav_from_file_path(path)?;
        let mut wave_reader = get_random_access_reader(open_wav)?;

        let actual_sample = wave_reader.read_sample(0)?;
        assert_eq!(
            expected_sample_0,
            actual_sample.front_left.expect("Front left missing"),
            "Wrong sample read at sample 0, channel 0"
        );

        let actual_sample = wave_reader.read_sample(1)?;
        assert_eq!(
            expected_sample_1,
            actual_sample.front_left.expect("Front left missing"),
            "Wrong sample read at sample 1, channel 0"
        );

        let actual_sample = wave_reader.read_sample(wave_reader.info().len_samples() - 1)?;
        assert_eq!(
            expected_sample_end,
            actual_sample.front_left.expect("Front left missing"),
            "Wrong sample read at sample 1266, channel 0"
        );

        Ok(())
    }

    #[test]
    fn read_stream_f32_sanity() {
        let file = File::open(Path::new("test_data/short_float.wav")).unwrap();
        let reader = BufReader::new(file).take(u64::MAX); // calling "take" forces reader to be just a Read, instead of a Read + Seek

        let open_wav = read_wav(reader).unwrap();
        let wave_reader_float = open_wav.get_stream_f32_reader().unwrap();
        assert_eq!(
            SampleFormat::Float,
            wave_reader_float.info().sample_format()
        );
        assert_eq!(1, wave_reader_float.info().num_channels());
        assert_eq!(32, wave_reader_float.info().bits_per_sample());
        assert_eq!(48000, wave_reader_float.info().sample_rate());
        assert_eq!(1267, wave_reader_float.info().len_samples());
    }

    #[test]
    fn read_stream_i8() {
        read_stream(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_stream_i8_reader()),
            i8::from_le_bytes([0x7D]),
            i8::from_le_bytes([0x7F]),
            i8::from_le_bytes([0x7A]),
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i8_as_i16() {
        read_stream(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            32255,
            32767,
            31487,
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i8_as_i24() {
        read_stream(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            8257535,
            8388607,
            8060927,
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i8_as_f32() {
        read_stream(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            0.9843137,
            1.0,
            0.9607843,
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i16() {
        read_stream(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            i16::from_le_bytes([0x61, 0xFD]),
            i16::from_le_bytes([0xF9, 0xFD]),
            i16::from_le_bytes([0x9C, 0xFE]),
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i16_as_i24() {
        read_stream(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            -171776,
            -132864,
            -91136,
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i16_as_f32() {
        read_stream(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            -0.020462334,
            -0.015823603,
            -0.010849178,
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i24() {
        read_stream(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            i32::from_le_bytes([0x00, 0x2E, 0x61, 0xFD]) >> 8,
            i32::from_le_bytes([0x00, 0xE7, 0xF8, 0xFD]) >> 8,
            i32::from_le_bytes([0x00, 0x94, 0x9C, 0xFE]) >> 8,
        )
        .unwrap();
    }

    #[test]
    fn read_stream_i24_as_f32() {
        read_stream(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            -0.020471752,
            -0.015841544,
            -0.010846555,
        )
        .unwrap();
    }

    #[test]
    fn read_stream_f32() {
        read_stream(
            Path::new("test_data/short_float.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            f32::from_le_bytes([0x6D, 0xB4, 0xA7, 0xBC]),
            f32::from_le_bytes([0x02, 0xC6, 0x81, 0xBC]),
            f32::from_le_bytes([0xA0, 0xB5, 0x31, 0xBC]),
        )
        .unwrap();
    }

    fn read_stream<T: Debug + PartialEq + Default + Clone>(
        path: &Path,
        get_stream_reader: Box<
            dyn FnOnce(OpenWavReader<Take<BufReader<File>>>) -> Result<StreamWavReader<T>>,
        >,
        expected_sample_0: T,
        expected_sample_1: T,
        expected_sample_end: T,
    ) -> Result<()> {
        let mut current_sample: usize = 0;

        let file = File::open(path)?;
        // calling "take" forces reader to be just a Read, instead of a Read + Seek
        let reader = BufReader::new(file).take(u64::MAX);

        let open_wav = read_wav(reader)?;

        let iterator = get_stream_reader(open_wav)?.into_iter();
        for samples_result in iterator {
            let samples = samples_result?;

            assert!(samples.front_left.is_some(), "Front left sample not read");
            assert_eq!(None, samples.front_right, "Sample should not be read");
            assert_eq!(None, samples.front_center, "Sample should not be read");
            assert_eq!(None, samples.low_frequency, "Sample should not be read");
            assert_eq!(None, samples.back_left, "Sample should not be read");
            assert_eq!(None, samples.back_right, "Sample should not be read");
            assert_eq!(
                None, samples.front_left_of_center,
                "Sample should not be read"
            );
            assert_eq!(
                None, samples.front_right_of_center,
                "Sample should not be read"
            );
            assert_eq!(None, samples.back_center, "Sample should not be read");
            assert_eq!(None, samples.side_left, "Sample should not be read");
            assert_eq!(None, samples.side_right, "Sample should not be read");
            assert_eq!(None, samples.top_center, "Sample should not be read");
            assert_eq!(None, samples.top_front_left, "Sample should not be read");
            assert_eq!(None, samples.top_front_center, "Sample should not be read");
            assert_eq!(None, samples.top_front_right, "Sample should not be read");
            assert_eq!(None, samples.top_back_left, "Sample should not be read");
            assert_eq!(None, samples.top_back_center, "Sample should not be read");
            assert_eq!(None, samples.top_back_right, "Sample should not be read");

            if current_sample == 0 {
                assert_eq!(
                    expected_sample_0,
                    samples.front_left.unwrap(),
                    "Wrong sample read at sample 0, channel 0"
                );
            } else if current_sample == 1 {
                assert_eq!(
                    expected_sample_1,
                    samples.front_left.unwrap(),
                    "Wrong sample read at sample 1, channel 0"
                );
            } else if current_sample == 1266 {
                assert_eq!(
                    expected_sample_end,
                    samples.front_left.unwrap(),
                    "Wrong sample read at sample 1266, channel 0"
                );
            }

            current_sample += 1;
        }

        assert_eq!(1267, current_sample, "Wrong number of samples read");

        Ok(())
    }

    fn test_with_file(file_test_callback: Box<dyn FnOnce(&Path) -> Result<()>>) {
        let temp_dir = tempdir().unwrap();

        {
            let path = temp_dir.path().join("tempwav.wav");
            file_test_callback(&path).unwrap();
        }
    }

    #[test]
    fn write_sanity() {
        test_with_file(Box::new(|path| {
            let header = WavHeader {
                sample_format: SampleFormat::Float,
                channels: Channels::new()
                    .front_left()
                    .front_right()
                    .front_center()
                    .low_frequency()
                    .back_left()
                    .back_right(),
                sample_rate: 96000,
                max_samples: 9600,
            };
            let mut open_wav = write_wav_to_file_path(path, header)?;

            assert_eq!(
                SampleFormat::Float,
                open_wav.sample_format(),
                "Wrong sample format"
            );
            assert_eq!(6, open_wav.num_channels(), "Wrong channels");
            assert_eq!(96000, open_wav.sample_rate(), "Wrong sampling rate");
            assert_eq!(4, open_wav.bytes_per_sample(), "Wrong bytes per sample");
            assert_eq!(32, open_wav.bits_per_sample(), "Wrong bits per sample");

            open_wav.flush()?;

            let open_wav = read_wav_from_file_path(path)?;

            assert_eq!(
                SampleFormat::Float,
                open_wav.sample_format(),
                "Wrong sample format when reading"
            );
            assert_eq!(6, open_wav.num_channels(), "Wrong channels when reading");
            assert_eq!(
                96000,
                open_wav.sample_rate(),
                "Wrong sampling rate when reading"
            );
            assert_eq!(
                4,
                open_wav.bytes_per_sample(),
                "Wrong bytes per sample when reading"
            );
            assert_eq!(
                32,
                open_wav.bits_per_sample(),
                "Wrong bits per sample when reading"
            );
            assert_eq!(0, open_wav.len_samples(), "Wrong length when reading");

            Ok(())
        }));
    }

    #[test]
    fn write_random_i8() {
        write_random(
            SampleFormat::Int8,
            Box::new(|open_wav| open_wav.get_random_access_i8_reader()),
            Box::new(|sample_value| sample_value),
            Box::new(|open_wav| open_wav.get_random_access_i8_writer()),
            Box::new(|sample_value| sample_value as i8),
        );
    }

    #[test]
    fn write_random_i8_as_i16() {
        write_random(
            SampleFormat::Int16,
            // Wav is upconverted from 8-bit to 16-bit on read
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
            Box::new(|sample_value| {
                let sample_value = sample_value as i32;
                if sample_value > 0 {
                    return (((sample_value + 1) / 256) - 1) as i8;
                } else {
                    // sample_value < 0 {
                    return (sample_value / 256) as i8;
                }
            }),
            Box::new(|open_wav| open_wav.get_random_access_i8_writer()),
            Box::new(|sample_value| sample_value as i8),
        );
    }

    #[test]
    fn write_random_i8_as_i24() {
        write_random(
            SampleFormat::Int24,
            // Wav is upconverted from 8-bit to 24-bit on read
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            Box::new(|sample_value| {
                if sample_value > 0 {
                    return (((sample_value + 1) / 65536) - 1) as i8;
                } else {
                    // sample_value < 0 {
                    return (sample_value / 65536) as i8;
                }
            }),
            Box::new(|open_wav| open_wav.get_random_access_i8_writer()),
            Box::new(|sample_value| sample_value as i8),
        );
    }

    #[test]
    fn write_random_i8_as_f32() {
        write_random(
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            Box::new(|sample_value| {
                let sample_int_8_abs = (sample_value + 1.0) * INT_8_DIVIDE_FOR_FLOAT;
                let sample_int_8_as_float = sample_int_8_abs - INT_8_ADD_FOR_FLOAT_ABS;
                return sample_int_8_as_float as i8;
            }),
            Box::new(|open_wav| open_wav.get_random_access_i8_writer()),
            Box::new(|sample_value| sample_value as i8),
        );
    }

    #[test]
    fn write_random_i16() {
        write_random(
            SampleFormat::Int16,
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
            Box::new(|sample_value| sample_value),
            Box::new(|open_wav| open_wav.get_random_access_i16_writer()),
            Box::new(|sample_value| sample_value as i16),
        );
    }

    #[test]
    fn write_random_i16_as_i24() {
        write_random(
            SampleFormat::Int24,
            // Wav is upconverted from 16-bit to 24-bit on read
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            Box::new(|sample_value| {
                if sample_value > 0 {
                    return (((sample_value + 1) / 256) - 1) as i16;
                } else {
                    // sample_value < 0 {
                    return (sample_value / 256) as i16;
                }
            }),
            Box::new(|open_wav| open_wav.get_random_access_i16_writer()),
            Box::new(|sample_value| sample_value as i16),
        );
    }

    #[test]
    fn write_random_i16_as_f32() {
        write_random(
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            Box::new(|sample_value| (sample_value * INT_16_DIVIDE_FOR_FLOAT) as i16),
            Box::new(|open_wav| open_wav.get_random_access_i16_writer()),
            Box::new(|sample_value| sample_value as i16),
        );
    }

    #[test]
    fn write_random_i24() {
        write_random(
            SampleFormat::Int24,
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            Box::new(|sample_value| sample_value),
            Box::new(|open_wav| open_wav.get_random_access_i24_writer()),
            Box::new(|sample_value| sample_value as i32),
        );
    }

    #[test]
    fn write_random_i24_as_f32() {
        write_random(
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            Box::new(|sample_value| (sample_value * INT_24_DIVIDE_FOR_FLOAT - 0.5) as i32),
            Box::new(|open_wav| open_wav.get_random_access_i24_writer()),
            Box::new(|sample_value| sample_value as i32),
        );
    }

    #[test]
    fn write_random_f32() {
        write_random(
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            Box::new(|sample_value| sample_value),
            Box::new(|open_wav| open_wav.get_random_access_f32_writer()),
            Box::new(|sample_value| sample_value as f32),
        );
    }

    fn write_random<T: Debug + PartialEq + 'static, TFile: Debug + PartialEq + 'static>(
        sample_format: SampleFormat,
        get_random_access_reader: Box<
            dyn FnOnce(OpenWavReader<BufReader<File>>) -> Result<RandomAccessWavReader<TFile>>,
        >,
        convert_sample_to_read: Box<dyn Fn(TFile) -> T>,
        get_random_access_writer: Box<
            dyn FnOnce(OpenWavWriter) -> Result<RandomAccessWavWriter<T>>,
        >,
        convert_sample_to_write: Box<dyn Fn(i32) -> T>,
    ) {
        test_with_file(Box::new(move |path| {
            let header = WavHeader {
                sample_format,
                channels: Channels {
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
                },
                sample_rate: 96000,
                max_samples: 9600,
            };
            let open_wav = write_wav_to_file_path(path, header)?;
            let mut writer = get_random_access_writer(open_wav)?;

            for sample_inv in 0..100usize {
                let sample = 99 - sample_inv;
                let sample_value = (sample as i32) * 18;
                let samples_by_channel = SamplesByChannel::<T> {
                    front_left: Some(convert_sample_to_write(sample_value + 0i32)),
                    front_right: Some(convert_sample_to_write(sample_value + 1)),
                    front_center: Some(convert_sample_to_write(sample_value + 2)),
                    low_frequency: Some(convert_sample_to_write(sample_value + 3)),
                    back_left: Some(convert_sample_to_write(sample_value + 4)),
                    back_right: Some(convert_sample_to_write(sample_value + 5)),
                    front_left_of_center: Some(convert_sample_to_write(sample_value + 6)),
                    front_right_of_center: Some(convert_sample_to_write(sample_value + 7)),
                    back_center: Some(convert_sample_to_write(sample_value + 8)),
                    side_left: Some(convert_sample_to_write(sample_value + 9)),
                    side_right: Some(convert_sample_to_write(sample_value + 10)),
                    top_center: Some(convert_sample_to_write(sample_value + 11)),
                    top_front_left: Some(convert_sample_to_write(sample_value + 12)),
                    top_front_center: Some(convert_sample_to_write(sample_value + 13)),
                    top_front_right: Some(convert_sample_to_write(sample_value + 14)),
                    top_back_left: Some(convert_sample_to_write(sample_value + 15)),
                    top_back_center: Some(convert_sample_to_write(sample_value + 16)),
                    top_back_right: Some(convert_sample_to_write(sample_value + 17)),
                };
                writer.write_samples(sample, samples_by_channel)?;
            }

            writer.flush()?;

            let open_wav = read_wav_from_file_path(path)?;
            assert_eq!(100, open_wav.len_samples(), "Wrong length of samples");

            let mut reader = get_random_access_reader(open_wav)?;

            for sample in 0..100usize {
                let samples_by_channel = reader.read_sample(sample)?;

                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (0 as i32)),
                    convert_sample_to_read(samples_by_channel.front_left.expect("front_left")),
                    "Wrong sample read at {sample}, channel front_left"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (1 as i32)),
                    convert_sample_to_read(samples_by_channel.front_right.expect("front_right")),
                    "Wrong sample read at {sample}, channel front_right"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (2 as i32)),
                    convert_sample_to_read(samples_by_channel.front_center.expect("front_center")),
                    "Wrong sample read at {sample}, channel front_center"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (3 as i32)),
                    convert_sample_to_read(
                        samples_by_channel.low_frequency.expect("low_frequency")
                    ),
                    "Wrong sample read at {sample}, channel low_frequency"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (4 as i32)),
                    convert_sample_to_read(samples_by_channel.back_left.expect("back_left")),
                    "Wrong sample read at {sample}, channel back_left"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (5 as i32)),
                    convert_sample_to_read(samples_by_channel.back_right.expect("back_right")),
                    "Wrong sample read at {sample}, channel back_right"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (6 as i32)),
                    convert_sample_to_read(
                        samples_by_channel
                            .front_left_of_center
                            .expect("front_left_of_center")
                    ),
                    "Wrong sample read at {sample}, channel front_left_of_center"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (7 as i32)),
                    convert_sample_to_read(
                        samples_by_channel
                            .front_right_of_center
                            .expect("front_right_of_center")
                    ),
                    "Wrong sample read at {sample}, channel front_right_of_center"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (8 as i32)),
                    convert_sample_to_read(samples_by_channel.back_center.expect("back_center")),
                    "Wrong sample read at {sample}, channel back_center"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (9 as i32)),
                    convert_sample_to_read(samples_by_channel.side_left.expect("side_left")),
                    "Wrong sample read at {sample}, channel side_left"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (10 as i32)),
                    convert_sample_to_read(samples_by_channel.side_right.expect("side_right")),
                    "Wrong sample read at {sample}, channel side_right"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (11 as i32)),
                    convert_sample_to_read(samples_by_channel.top_center.expect("top_center")),
                    "Wrong sample read at {sample}, channel top_center"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (12 as i32)),
                    convert_sample_to_read(
                        samples_by_channel.top_front_left.expect("top_front_left")
                    ),
                    "Wrong sample read at {sample}, channel top_front_left"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (13 as i32)),
                    convert_sample_to_read(
                        samples_by_channel
                            .top_front_center
                            .expect("top_front_center")
                    ),
                    "Wrong sample read at {sample}, channel top_front_center"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (14 as i32)),
                    convert_sample_to_read(
                        samples_by_channel.top_front_right.expect("top_front_right")
                    ),
                    "Wrong sample read at {sample}, channel top_front_right"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (15 as i32)),
                    convert_sample_to_read(
                        samples_by_channel.top_back_left.expect("top_back_left")
                    ),
                    "Wrong sample read at {sample}, channel top_back_left"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (16 as i32)),
                    convert_sample_to_read(
                        samples_by_channel.top_back_center.expect("top_back_center")
                    ),
                    "Wrong sample read at {sample}, channel top_back_center"
                );
                assert_eq!(
                    convert_sample_to_write((sample as i32) * 18 + (17 as i32)),
                    convert_sample_to_read(
                        samples_by_channel.top_back_right.expect("top_back_right")
                    ),
                    "Wrong sample read at {sample}, channel top_back_right"
                );
            }

            Ok(())
        }));
    }

    #[test]
    fn write_random_max_samples() {
        test_with_file(Box::new(move |path| {
            let header = WavHeader {
                sample_format: SampleFormat::Int8,
                channels: Channels {
                    front_left: false,
                    front_right: false,
                    front_center: true,
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
                },
                sample_rate: 96000,
                max_samples: 1,
            };
            let open_wav = write_wav_to_file_path(path, header)?;
            let mut writer = open_wav.get_random_access_i8_writer()?;

            let samples_by_channel = SamplesByChannel::<i8> {
                front_left: None,
                front_right: None,
                front_center: Some(1),
                low_frequency: None,
                back_left: None,
                back_right: None,
                front_left_of_center: None,
                front_right_of_center: None,
                back_center: None,
                side_left: None,
                side_right: None,
                top_center: None,
                top_front_left: None,
                top_front_center: None,
                top_front_right: None,
                top_back_left: None,
                top_back_center: None,
                top_back_right: None,
            };
            writer.write_samples(0, samples_by_channel.clone())?;

            let err = writer
                .write_samples(1, samples_by_channel.clone())
                .expect_err("Writing at the max length should fail");

            assert_eq!(ErrorKind::Unsupported, err.kind());

            let err = writer
                .write_samples(2, samples_by_channel)
                .expect_err("Writing beyond the max length should fail");

            assert_eq!(ErrorKind::Unsupported, err.kind());

            Ok(())
        }));
    }

    #[test]
    fn write_stream_i8() {
        write_stream(
            Path::new("test_data/short_8.wav"),
            SampleFormat::Int8,
            Box::new(|open_wav| open_wav.get_stream_i8_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i8(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i8_reader()),
        )
    }

    #[test]
    fn write_stream_i8_as_i16() {
        write_stream(
            Path::new("test_data/short_8.wav"),
            SampleFormat::Int16,
            Box::new(|open_wav| open_wav.get_stream_i8_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i8(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
        )
    }

    #[test]
    fn write_stream_i8_as_i24() {
        write_stream(
            Path::new("test_data/short_8.wav"),
            SampleFormat::Int24,
            Box::new(|open_wav| open_wav.get_stream_i8_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i8(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
        )
    }

    #[test]
    fn write_stream_i8_as_f32() {
        write_stream(
            Path::new("test_data/short_8.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_i8_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i8(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
        )
    }

    #[test]
    fn write_stream_i16() {
        write_stream(
            Path::new("test_data/short_16.wav"),
            SampleFormat::Int16,
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i16(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
        )
    }

    #[test]
    fn write_stream_i16_as_i24() {
        write_stream(
            Path::new("test_data/short_16.wav"),
            SampleFormat::Int24,
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i16(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
        )
    }

    #[test]
    fn write_stream_i16_as_f32() {
        write_stream(
            Path::new("test_data/short_16.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i16(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
        )
    }

    #[test]
    fn write_stream_i24() {
        write_stream(
            Path::new("test_data/short_24.wav"),
            SampleFormat::Int24,
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i24(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
        )
    }

    #[test]
    fn write_stream_i24_as_f32() {
        write_stream(
            Path::new("test_data/short_24.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i24(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
        )
    }

    #[test]
    fn write_stream_f32() {
        write_stream(
            Path::new("test_data/short_float.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_f32(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
        )
    }

    fn write_stream<
        T: Debug + PartialEq + Default + Clone + 'static,
        TFile: Debug + PartialEq + Default + Clone + 'static,
    >(
        read_path: &Path,
        sample_format: SampleFormat,
        get_stream_reader: Box<
            dyn FnOnce(OpenWavReader<BufReader<File>>) -> Result<StreamWavReader<T>>,
        >,
        write_all: Box<dyn FnOnce(OpenWavWriter, StreamWavReaderIterator<T>) -> Result<()>>,
        get_random_access_reader: Box<
            dyn Fn(OpenWavReader<BufReader<File>>) -> Result<RandomAccessWavReader<TFile>>,
        >,
    ) {
        let read_path_buf = read_path.to_path_buf();

        test_with_file(Box::new(move |path| {
            let read_path = read_path_buf.as_path();
            let source_wav = read_wav_from_file_path(read_path)?;

            let header = WavHeader {
                sample_format,
                channels: source_wav.channels().clone(),
                sample_rate: source_wav.sample_rate(),
                max_samples: 9600,
            };
            let open_wav = write_wav_to_file_path(path, header)?;

            let read_samples_iter = get_stream_reader(source_wav)?.into_iter();
            write_all(open_wav, read_samples_iter)?;

            let expected_wav = read_wav_from_file_path(read_path)?;
            let actual_wav = read_wav_from_file_path(path)?;

            assert_eq!(expected_wav.num_channels(), actual_wav.num_channels());
            assert_eq!(expected_wav.len_samples(), actual_wav.len_samples());

            let len_samples = expected_wav.len_samples();

            let mut expected_wav_reader = get_random_access_reader(expected_wav)?;
            let mut actual_wav_reader = get_random_access_reader(actual_wav)?;

            for sample_ctr in 0..len_samples {
                let expected_samples = expected_wav_reader.read_sample(sample_ctr)?;
                let actual_samples = actual_wav_reader.read_sample(sample_ctr)?;

                assert_eq!(
                    expected_samples, actual_samples,
                    "Wrong value for sample {sample_ctr}"
                );
            }

            Ok(())
        }));
    }
}
