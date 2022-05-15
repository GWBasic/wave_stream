use std::fs::File;
use std::io::{ BufReader, ErrorKind, Read, Result, Seek };
use std::str;

pub mod reader;
pub mod wave_header;
pub mod wave_reader;

use reader::ReadEx;
use wave_header::{SampleFormat, WavHeader};
use wave_reader::{OpenWav, RandomAccessWavReader};

pub fn from_file(file_path: &str) -> Result<OpenWav<BufReader<File>>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    from_reader(reader)
}

pub fn from_reader<TReader: Read + Seek>(mut reader: TReader) -> Result<OpenWav<TReader>> {
    // Verify that this is a RIFF file
    reader.assert_str("RIFF", ErrorKind::InvalidInput, "Not a WAVE file (Missing RIFF Header)")?;
    let _file_length = reader.read_u32()?;
    reader.assert_str("WAVE", ErrorKind::Unsupported, "Not a WAVE file (Missing WAVE header)")?;

    let header = WavHeader::from_reader(&mut reader)?;

    Ok(OpenWav::new(reader, header)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_sanity() {
        let open_wav = from_file("test_data/short_float.wav").unwrap();
        assert_eq!(SampleFormat::Float, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(32, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = from_file("test_data/short_24.wav").unwrap();
        assert_eq!(SampleFormat::Int24, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(24, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = from_file("test_data/short_16.wav").unwrap();
        assert_eq!(SampleFormat::Int16, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(16, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());
    }

    #[test]
    fn read_float_sanity() {
        let open_wav = from_file("test_data/short_float.wav").unwrap();
        let wave_reader_float = open_wav.as_random_access_float().unwrap();
        assert_eq!(SampleFormat::Float, wave_reader_float.info().sample_format());
        assert_eq!(1, wave_reader_float.info().channels());
        assert_eq!(32, wave_reader_float.info().bits_per_sample());
        assert_eq!(48000, wave_reader_float.info().sample_rate());
        assert_eq!(1267, wave_reader_float.info().len_samples());

        let open_wav = from_file("test_data/short_24.wav").unwrap();
        let read_float_result = open_wav.as_random_access_float();
        match read_float_result {
            Result::Err(_) => {},
            _ => panic!("Should not be able to read file")
        }

        let open_wav = from_file("test_data/short_16.wav").unwrap();
        let read_float_result = open_wav.as_random_access_float();
        match read_float_result {
            Result::Err(_) => {},
            _ => panic!("Should not be able to read file")
        }
    }

    #[test]
    fn read_float() {
        let open_wav = from_file("test_data/short_float.wav").unwrap();
        let mut wave_reader_float = open_wav.as_random_access_float().unwrap();

        let expected_sample = f32::from_le_bytes([0x6D, 0xB4, 0xA7, 0xBC]);
        let actual_sample = wave_reader_float.read_sample(0, 0).unwrap();
        assert_eq!(expected_sample, actual_sample, "Wrong sample read at sample 0, channel 0");

        let expected_sample = f32::from_le_bytes([0x02, 0xC6, 0x81, 0xBC]);
        let actual_sample = wave_reader_float.read_sample(1, 0).unwrap();
        assert_eq!(expected_sample, actual_sample, "Wrong sample read at sample 0, channel 0");

        let expected_sample = f32::from_le_bytes([0xA0, 0xB5, 0x31, 0xBC]);
        let actual_sample = wave_reader_float.read_sample(wave_reader_float.info().len_samples() - 1, 0).unwrap();
        assert_eq!(expected_sample, actual_sample, "Wrong sample read at sample 0, channel 0");
    }
}
