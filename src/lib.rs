use std::fs::File;
use std::io::{ BufReader, ErrorKind, Read, Result, Seek };
use std::str;

pub mod reader;
pub mod wave_header;
pub mod wave_reader;

use reader::ReadEx;
use wave_header::{SampleFormat, WavHeader};
use wave_reader::{OpenWav, WavInfo};

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
        let wave_reader = from_file("test_data/short_float.wav").unwrap();
        assert_eq!(SampleFormat::Float, wave_reader.sample_format());
        assert_eq!(1, wave_reader.channels());
        assert_eq!(32, wave_reader.bits_per_sample());
        assert_eq!(48000, wave_reader.sample_rate());
        assert_eq!(1267, wave_reader.len_samples());

        let wave_reader = from_file("test_data/short_24.wav").unwrap();
        assert_eq!(SampleFormat::Int24, wave_reader.sample_format());
        assert_eq!(1, wave_reader.channels());
        assert_eq!(24, wave_reader.bits_per_sample());
        assert_eq!(48000, wave_reader.sample_rate());
        assert_eq!(1267, wave_reader.len_samples());

        let wave_reader = from_file("test_data/short_16.wav").unwrap();
        assert_eq!(SampleFormat::Int16, wave_reader.sample_format());
        assert_eq!(1, wave_reader.channels());
        assert_eq!(16, wave_reader.bits_per_sample());
        assert_eq!(48000, wave_reader.sample_rate());
        assert_eq!(1267, wave_reader.len_samples());
    }
}
