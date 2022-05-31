use std::fs::File;
use std::io::{ BufReader, BufWriter, Error, ErrorKind, Read, Result, Write, Seek };
use std::str;

pub mod reader;
pub mod wave_header;
pub mod wave_reader;
pub mod wave_writer;
pub mod writer;

use reader::ReadEx;
use wave_header::*;
use wave_reader::*;
use wave_writer::*;
use writer::WriteEx;

pub fn read_wav_from_file_path(file_path: &str) -> Result<OpenWavReader<BufReader<File>>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    read_wav(reader)
}

pub fn read_wav<TReader: Read + Seek>(mut reader: TReader) -> Result<OpenWavReader<TReader>> {
    // Verify that this is a RIFF file
    reader.assert_str("RIFF", ErrorKind::InvalidInput, "Not a WAVE file (Missing RIFF Header)")?;
    let _file_length = reader.read_u32()?;
    reader.assert_str("WAVE", ErrorKind::Unsupported, "Not a WAVE file (Missing WAVE header)")?;

    let header = WavHeader::from_reader(&mut reader)?;

    OpenWavReader::new(reader, header)
}

pub fn write_wav_to_file_path(file_path: &str, header: WavHeader) -> Result<OpenWavWriter<BufWriter<File>>> {
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    write_wav(writer, header)
}

pub fn write_wav<TWriter: Write + Seek>(mut writer: TWriter, header: WavHeader) -> Result<OpenWavWriter<TWriter>> {

    // Write RIFF header and format
    writer.write(b"RIFF    WAVE")?;

    WavHeader::to_writer(&mut writer, &header)?;

    OpenWavWriter::new(writer, header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn open_sanity() {
        let open_wav = read_wav_from_file_path("test_data/short_float.wav").unwrap();
        assert_eq!(SampleFormat::Float, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(32, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = read_wav_from_file_path("test_data/short_24.wav").unwrap();
        assert_eq!(SampleFormat::Int24, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(24, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = read_wav_from_file_path("test_data/short_16.wav").unwrap();
        assert_eq!(SampleFormat::Int16, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(16, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());
    }

    #[test]
    fn read_float_sanity() {
        let open_wav = read_wav_from_file_path("test_data/short_float.wav").unwrap();
        let wave_reader_float = open_wav.get_random_access_float_reader().unwrap();
        assert_eq!(SampleFormat::Float, wave_reader_float.info().sample_format());
        assert_eq!(1, wave_reader_float.info().channels());
        assert_eq!(32, wave_reader_float.info().bits_per_sample());
        assert_eq!(48000, wave_reader_float.info().sample_rate());
        assert_eq!(1267, wave_reader_float.info().len_samples());

        let open_wav = read_wav_from_file_path("test_data/short_24.wav").unwrap();
        let read_float_result = open_wav.get_random_access_float_reader();
        match read_float_result {
            Result::Err(_) => {},
            _ => panic!("Should not be able to read file")
        }

        let open_wav = read_wav_from_file_path("test_data/short_16.wav").unwrap();
        let read_float_result = open_wav.get_random_access_float_reader();
        match read_float_result {
            Result::Err(_) => {},
            _ => panic!("Should not be able to read file")
        }
    }

    #[test]
    fn read_float() {
        let open_wav = read_wav_from_file_path("test_data/short_float.wav").unwrap();
        let mut wave_reader_float = open_wav.get_random_access_float_reader().unwrap();

        let expected_sample = f32::from_le_bytes([0x6D, 0xB4, 0xA7, 0xBC]);
        let actual_sample = wave_reader_float.read_sample(0, 0).unwrap();
        assert_eq!(expected_sample, actual_sample, "Wrong sample read at sample 0, channel 0");

        let expected_sample = f32::from_le_bytes([0x02, 0xC6, 0x81, 0xBC]);
        let actual_sample = wave_reader_float.read_sample(1, 0).unwrap();
        assert_eq!(expected_sample, actual_sample, "Wrong sample read at sample 1, channel 0");

        let expected_sample = f32::from_le_bytes([0xA0, 0xB5, 0x31, 0xBC]);
        let actual_sample = wave_reader_float.read_sample(wave_reader_float.info().len_samples() - 1, 0).unwrap();
        assert_eq!(expected_sample, actual_sample, "Wrong sample read at sample 1266, channel 0");
    }

    #[test]
    fn stream_float_sanity() {
        let open_wav = read_wav_from_file_path("test_data/short_float.wav").unwrap();
        let wave_reader_float = open_wav.get_stream_float_reader().unwrap();
        assert_eq!(SampleFormat::Float, wave_reader_float.info().sample_format());
        assert_eq!(1, wave_reader_float.info().channels());
        assert_eq!(32, wave_reader_float.info().bits_per_sample());
        assert_eq!(48000, wave_reader_float.info().sample_rate());
        assert_eq!(1267, wave_reader_float.info().len_samples());

        let open_wav = read_wav_from_file_path("test_data/short_24.wav").unwrap();
        let stream_float_result = open_wav.get_stream_float_reader();
        match stream_float_result {
            Result::Err(_) => {},
            _ => panic!("Should not be able to read file")
        }

        let open_wav = read_wav_from_file_path("test_data/short_16.wav").unwrap();
        let stream_float_result = open_wav.get_stream_float_reader();
        match stream_float_result {
            Result::Err(_) => {},
            _ => panic!("Should not be able to read file")
        }
    }

    #[test]
    fn stream_float() {
        let mut current_sample: usize = 0;
        let open_wav = read_wav_from_file_path("test_data/short_float.wav").unwrap();
        for samples_result in open_wav.get_stream_float_reader().unwrap().into_iter() {
            let samples = samples_result.unwrap();

            assert_eq!(1, samples.len(), "Wrong number of samples");

            if current_sample == 0 {
                let expected_sample = f32::from_le_bytes([0x6D, 0xB4, 0xA7, 0xBC]);
                assert_eq!(expected_sample, samples[0], "Wrong sample read at sample 0, channel 0");
        
            } else if current_sample == 1 {
                let expected_sample = f32::from_le_bytes([0x02, 0xC6, 0x81, 0xBC]);
                assert_eq!(expected_sample, samples[0], "Wrong sample read at sample 1, channel 0");
            } else if current_sample == 1266 {
                let expected_sample = f32::from_le_bytes([0xA0, 0xB5, 0x31, 0xBC]);
                assert_eq!(expected_sample, samples[0], "Wrong sample read at sample 1266, channel 0");
            }

            current_sample += 1;
        }

        assert_eq!(1267, current_sample, "Wrong number of samples read");
    }

    #[test]
    fn write_sanity() {
        // TODO: Encapsulate this in a callback
        let result = write_sanity_test();

        let _ignored = remove_file("test_data/write_sanity.wav");

        result.unwrap();
    }

    fn write_sanity_test() -> Result<()> {
        let header = WavHeader {
            sample_format: SampleFormat::Float,
            channels: 10,
            sample_rate: 96000
        };
        let mut open_wav = write_wav_to_file_path("test_data/write_sanity.wav", header)?;

        assert_eq!(SampleFormat::Float, open_wav.sample_format(), "Wrong sample format");
        assert_eq!(10, open_wav.channels(), "Wrong channels");
        assert_eq!(96000, open_wav.sample_rate(), "Wrong sampling rate");
        assert_eq!(4, open_wav.bytes_per_sample(), "Wrong bytes per sample");
        assert_eq!(32, open_wav.bits_per_sample(), "Wrong bits per sample");

        open_wav.flush()?;

        let open_wav = read_wav_from_file_path("test_data/write_sanity.wav")?;

        assert_eq!(SampleFormat::Float, open_wav.sample_format(), "Wrong sample format when reading");
        assert_eq!(10, open_wav.channels(), "Wrong channels when reading");
        assert_eq!(96000, open_wav.sample_rate(), "Wrong sampling rate when reading");
        assert_eq!(4, open_wav.bytes_per_sample(), "Wrong bytes per sample when reading");
        assert_eq!(32, open_wav.bits_per_sample(), "Wrong bits per sample when reading");
        assert_eq!(0, open_wav.len_samples(), "Wrong length when reading");

        Ok(())
    }

    #[test]
    fn write_random() {
        // TODO: Encapsulate this in a callback
        let result = write_random_test();

        let _ignored = remove_file("test_data/write_random.wav");

        result.unwrap();
    }
 
    fn write_random_test() -> Result<()> {
        let header = WavHeader {
            sample_format: SampleFormat::Float,
            channels: 10,
            sample_rate: 96000
        };
        let open_wav = write_wav_to_file_path("test_data/write_random.wav", header)?;

        let mut writer = open_wav.get_random_access_float_writer()?;

        for sample in 0..100 {
            for channel in 0..writer.info().channels() {
                writer.write_sample(sample, channel, (sample as f32 * 10f32) + channel as f32)?;
            }
        }

        writer.flush()?;

        let open_wav = read_wav_from_file_path("test_data/write_random.wav")?;
        assert_eq!(100, open_wav.len_samples(), "Wrong length of samples");

        let mut reader = open_wav.get_random_access_float_reader()?;

        for sample in 0..100 {
            for channel in 0..reader.info().channels() {
                let value = reader.read_sample(sample, channel)?;
                assert_eq!((sample as f32 * 10f32) + channel as f32, value, "Wrong sample read");
            }
        }

        Ok(())
    }
}
