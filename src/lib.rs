use std::fs::File;
use std::io::{ BufReader, BufWriter, ErrorKind, Read, Result, Seek, Write };
use std::path::Path;

pub mod open_wav;
pub mod reader;
pub mod wave_header;
pub mod wave_reader;
pub mod wave_writer;
pub mod writer;

mod assertions;
mod constants;
mod upconvert;

use reader::ReadEx;
use wave_header::*;
use wave_reader::*;
use wave_writer::*;
use writer::WriteEx;

pub fn read_wav_from_file_path(file_path: &Path) -> Result<OpenWavReader<BufReader<File>>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    read_wav(reader)
}

pub fn read_wav<TReader: 'static + Read>(mut reader: TReader) -> Result<OpenWavReader<TReader>> {
    // Verify that this is a RIFF file
    reader.assert_str("RIFF", ErrorKind::InvalidInput, "Not a WAVE file (Missing RIFF Header)")?;
    let _file_length = reader.read_u32()?;
    reader.assert_str("WAVE", ErrorKind::Unsupported, "Not a WAVE file (Missing WAVE header)")?;

    // file position is 12

    let mut subchunk_size = 0u32;
    let header = WavHeader::from_reader(&mut reader, &mut subchunk_size)?;

    // subchunk size doesn't include 4-letter prefix and 4-byte length

    OpenWavReader::new(reader, header, 20 + subchunk_size)
}

pub fn write_wav_to_file_path(file_path: &Path, header: WavHeader) -> Result<OpenWavWriter> {
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    write_wav(writer, header)
}

pub fn write_wav<TWriter: 'static + Write + Seek>(mut writer: TWriter, header: WavHeader) -> Result<OpenWavWriter> {
    // Write RIFF header and format
    writer.write(b"RIFF    WAVE")?;

    WavHeader::to_writer(&mut writer, &header)?;

    OpenWavWriter::new(writer, header)
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::io::Take;

    use tempfile::tempdir;

    use super::*;
    use crate::upconvert::{ INT_16_DIVIDE_FOR_FLOAT, INT_24_DIVIDE_FOR_FLOAT, INT_8_ADD_FOR_FLOAT_ABS, INT_8_DIVIDE_FOR_FLOAT };
    use crate::open_wav::OpenWav;

    #[test]
    fn open_sanity() {
        let open_wav = read_wav_from_file_path(Path::new("test_data/short_float.wav")).unwrap();
        assert_eq!(SampleFormat::Float, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(32, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = read_wav_from_file_path(Path::new("test_data/short_24.wav")).unwrap();
        assert_eq!(SampleFormat::Int24, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(24, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());

        let open_wav = read_wav_from_file_path(Path::new("test_data/short_16.wav")).unwrap();
        assert_eq!(SampleFormat::Int16, open_wav.sample_format());
        assert_eq!(1, open_wav.channels());
        assert_eq!(16, open_wav.bits_per_sample());
        assert_eq!(48000, open_wav.sample_rate());
        assert_eq!(1267, open_wav.len_samples());
    }

    #[test]
    fn read_float_sanity() {
        let open_wav = read_wav_from_file_path(Path::new("test_data/short_float.wav")).unwrap();
        let wave_reader_float = open_wav.get_random_access_f32_reader().unwrap();
        assert_eq!(SampleFormat::Float, wave_reader_float.info().sample_format());
        assert_eq!(1, wave_reader_float.info().channels());
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
            i8::from_le_bytes([0x7A])).unwrap();
    }

    #[test]
    fn read_random_i16() {
        read_random(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
            i16::from_le_bytes([0x61, 0xFD]),
            i16::from_le_bytes([0xF9, 0xFD]),
            i16::from_le_bytes([0x9C, 0xFE])).unwrap();
    }

    #[test]
    fn read_random_i24() {
        read_random(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            i32::from_le_bytes([0x2E, 0x61, 0xFD, 0x00]) >> 8,
            i32::from_le_bytes([0xE7, 0xF8, 0xFD, 0x00]) >> 8,
            i32::from_le_bytes([0x94, 0x9C, 0xFE, 0x00]) >> 8).unwrap();
    }

    #[test]
    fn read_random_i8_as_f32() {
        read_random(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            0.9843137,
            1.0,
            0.9607843).unwrap();
    }

    #[test]
    fn read_random_i16_as_f32() {
        read_random(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            -0.020462334,
            -0.015823603,
            -0.010849178).unwrap();
    }

    #[test]
    fn read_random_i24_as_f32() {
        read_random(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            0.00773263,
            0.0077506304,
            0.0077701807).unwrap();
    }

    #[test]
    fn read_random_f32() {
        read_random(
            Path::new("test_data/short_float.wav"),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            f32::from_le_bytes([0x6D, 0xB4, 0xA7, 0xBC]),
            f32::from_le_bytes([0x02, 0xC6, 0x81, 0xBC]),
            f32::from_le_bytes([0xA0, 0xB5, 0x31, 0xBC])).unwrap();
    }

    fn read_random<T: Debug + PartialEq>(
        path: &Path,
        get_random_access_reader: Box<dyn FnOnce(OpenWavReader<BufReader<File>>) -> Result<RandomAccessWavReader<T>>>,
        expected_sample_0: T,
        expected_sample_1: T,
        expected_sample_end: T)
        -> Result<()> {

        let open_wav = read_wav_from_file_path(path)?;
        let mut wave_reader = get_random_access_reader(open_wav)?;

        let actual_sample = wave_reader.read_sample(0, 0)?;
        assert_eq!(expected_sample_0, actual_sample, "Wrong sample read at sample 0, channel 0");

        let actual_sample = wave_reader.read_sample(1, 0)?;
        assert_eq!(expected_sample_1, actual_sample, "Wrong sample read at sample 1, channel 0");

        let actual_sample = wave_reader.read_sample(wave_reader.info().len_samples() - 1, 0)?;
        assert_eq!(expected_sample_end, actual_sample, "Wrong sample read at sample 1266, channel 0");

        Ok(())
    }

    #[test]
    fn read_stream_f32_sanity() {

        let file = File::open(Path::new("test_data/short_float.wav")).unwrap();
        let reader = BufReader::new(file)
            .take(u64::MAX); // calling "take" forces reader to be just a Read, instead of a Read + Seek
    
        let open_wav = read_wav(reader).unwrap();
        let wave_reader_float = open_wav.get_stream_f32_reader().unwrap();
        assert_eq!(SampleFormat::Float, wave_reader_float.info().sample_format());
        assert_eq!(1, wave_reader_float.info().channels());
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
            i8::from_le_bytes([0x7A])).unwrap();
    }

    #[test]
    fn read_stream_i8_as_f32() {
        read_stream(
            Path::new("test_data/short_8.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            0.9843137,
            1.0,
            0.9607843).unwrap();
    }

    #[test]
    fn read_stream_i16() {
        read_stream(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            i16::from_le_bytes([0x61, 0xFD]),
            i16::from_le_bytes([0xF9, 0xFD]),
            i16::from_le_bytes([0x9C, 0xFE])).unwrap();
    }

    #[test]
    fn read_stream_i16_as_f32() {
        read_stream(
            Path::new("test_data/short_16.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            -0.020462334,
            -0.015823603,
            -0.010849178).unwrap();
    }

    #[test]
    fn read_stream_i24() {
        read_stream(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            i32::from_le_bytes([0x2E, 0x61, 0xFD, 0x00]) >> 8,
            i32::from_le_bytes([0xE7, 0xF8, 0xFD, 0x00]) >> 8,
            i32::from_le_bytes([0x94, 0x9C, 0xFE, 0x00]) >> 8).unwrap();
    }

    #[test]
    fn read_stream_i24_as_f32() {
        read_stream(
            Path::new("test_data/short_24.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            0.00773263,
            0.0077506304,
            0.0077701807).unwrap();
    }

    #[test]
    fn read_stream_f32() {
        read_stream(
            Path::new("test_data/short_float.wav"),
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            f32::from_le_bytes([0x6D, 0xB4, 0xA7, 0xBC]),
            f32::from_le_bytes([0x02, 0xC6, 0x81, 0xBC]),
            f32::from_le_bytes([0xA0, 0xB5, 0x31, 0xBC])).unwrap();
    }

    fn read_stream<T: Debug + PartialEq + Default + Clone>(
        path: &Path,
        get_stream_reader: Box<dyn FnOnce(OpenWavReader<Take<BufReader<File>>>) -> Result<StreamWavReader<T>>>,
        expected_sample_0: T,
        expected_sample_1: T,
        expected_sample_end: T)
        -> Result<()> {
            let mut current_sample: usize = 0;

            let file = File::open(path)?;
            let reader = BufReader::new(file)
                .take(u64::MAX); // calling "take" forces reader to be just a Read, instead of a Read + Seek
        
            let open_wav = read_wav(reader)?;
    
            let iterator = get_stream_reader(open_wav)?.into_iter();
            for samples_result in iterator {
                let samples = samples_result?;
    
                assert_eq!(1, samples.len(), "Wrong number of samples");
    
                if current_sample == 0 {
                    assert_eq!(expected_sample_0, samples[0], "Wrong sample read at sample 0, channel 0");
            
                } else if current_sample == 1 {
                    assert_eq!(expected_sample_1, samples[0], "Wrong sample read at sample 1, channel 0");
                } else if current_sample == 1266 {
                    assert_eq!(expected_sample_end, samples[0], "Wrong sample read at sample 1266, channel 0");
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
                channels: 10,
                sample_rate: 96000
            };
            let mut open_wav = write_wav_to_file_path(path, header)?;

            assert_eq!(SampleFormat::Float, open_wav.sample_format(), "Wrong sample format");
            assert_eq!(10, open_wav.channels(), "Wrong channels");
            assert_eq!(96000, open_wav.sample_rate(), "Wrong sampling rate");
            assert_eq!(4, open_wav.bytes_per_sample(), "Wrong bytes per sample");
            assert_eq!(32, open_wav.bits_per_sample(), "Wrong bits per sample");

            open_wav.flush()?;

            let open_wav = read_wav_from_file_path(path)?;

            assert_eq!(SampleFormat::Float, open_wav.sample_format(), "Wrong sample format when reading");
            assert_eq!(10, open_wav.channels(), "Wrong channels when reading");
            assert_eq!(96000, open_wav.sample_rate(), "Wrong sampling rate when reading");
            assert_eq!(4, open_wav.bytes_per_sample(), "Wrong bytes per sample when reading");
            assert_eq!(32, open_wav.bits_per_sample(), "Wrong bits per sample when reading");
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
            Box::new(|sample_value| sample_value as i8));
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
            Box::new(|sample_value| sample_value as i8));
    }

    #[test]
    fn write_random_i16() {
        write_random(
            SampleFormat::Int16,
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()),
            Box::new(|sample_value| sample_value),
            Box::new(|open_wav| open_wav.get_random_access_i16_writer()),
            Box::new(|sample_value| sample_value as i16));
    }

    #[test]
    fn write_random_i16_as_f32() {
        write_random(
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            Box::new(|sample_value| (sample_value * INT_16_DIVIDE_FOR_FLOAT) as i16),
            Box::new(|open_wav| open_wav.get_random_access_i16_writer()),
            Box::new(|sample_value| sample_value as i16));
    }

    #[test]
    fn write_random_i24() {
        write_random(
            SampleFormat::Int24,
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()),
            Box::new(|sample_value| sample_value),
            Box::new(|open_wav| open_wav.get_random_access_i24_writer()),
            Box::new(|sample_value| sample_value as i32));
    }

    #[test]
    fn write_random_i24_as_f32() {
        write_random(
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            Box::new(|sample_value| (sample_value * INT_24_DIVIDE_FOR_FLOAT - 0.5) as i32),
            Box::new(|open_wav| open_wav.get_random_access_i24_writer()),
            Box::new(|sample_value| sample_value as i32));
    }

    #[test]
    fn write_random_f32() {
        write_random(
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()),
            Box::new(|sample_value| sample_value),
            Box::new(|open_wav| open_wav.get_random_access_f32_writer()),
            Box::new(|sample_value| sample_value as f32));
    }

    fn write_random<T: Debug + PartialEq + 'static, TFile: Debug + PartialEq + 'static>(
        sample_format: SampleFormat,
        get_random_access_reader: Box<dyn FnOnce(OpenWavReader<BufReader<File>>) -> Result<RandomAccessWavReader<TFile>>>,
        convert_sample_to_read: Box<dyn Fn(TFile) -> T>,
        get_random_access_writer: Box<dyn FnOnce(OpenWavWriter) -> Result<RandomAccessWavWriter<T>>>,
        convert_sample_to_write: Box<dyn Fn(i32) -> T>) {
            
        test_with_file(Box::new(move |path| {
            let header = WavHeader {
                sample_format,
                channels: 10,
                sample_rate: 96000
            };
            let open_wav = write_wav_to_file_path(path, header)?;
            let mut writer = get_random_access_writer(open_wav)?;

            for sample in 0..100u32 {
                for channel in 0..writer.info().channels() {
                    let sample_value = (sample as i32) * 10 + (channel as i32);
                    writer.write_sample(sample, channel, convert_sample_to_write(sample_value))?;
                }
            }

            writer.flush()?;

            let open_wav = read_wav_from_file_path(path)?;
            assert_eq!(100, open_wav.len_samples(), "Wrong length of samples");

            let mut reader = get_random_access_reader(open_wav)?;

            for sample in 0..100 {
                for channel in 0..reader.info().channels() {
                    let value = reader.read_sample(sample, channel)?;
                    let sample_value = (sample as i32) * 10 + (channel as i32);
                    assert_eq!(
                        convert_sample_to_write(sample_value),
                        convert_sample_to_read(value),
                        "Wrong sample read at {sample}, channel {channel}");
                }
            }

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
            Box::new(|open_wav| open_wav.get_random_access_i8_reader()))
    }

    #[test]
    fn write_stream_i8_as_f32() {
        write_stream(
            Path::new("test_data/short_8.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_i8_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i8(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()))
    }

    #[test]
    fn write_stream_i16() {
        write_stream(
            Path::new("test_data/short_16.wav"),
            SampleFormat::Int16,
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i16(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i16_reader()))
    }

    #[test]
    fn write_stream_i16_as_f32() {
        write_stream(
            Path::new("test_data/short_16.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_i16_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i16(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()))
    }

    #[test]
    fn write_stream_i24() {
        write_stream(
            Path::new("test_data/short_24.wav"),
            SampleFormat::Int24,
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i24(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_i24_reader()))
    }

    #[test]
    fn write_stream_i24_as_f32() {
        write_stream(
            Path::new("test_data/short_24.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_i24_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_i24(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()))
    }

    #[test]
    fn write_stream_f32() {
        write_stream(
            Path::new("test_data/short_float.wav"),
            SampleFormat::Float,
            Box::new(|open_wav| open_wav.get_stream_f32_reader()),
            Box::new(|open_wav, read_samples_iter| open_wav.write_all_f32(read_samples_iter)),
            Box::new(|open_wav| open_wav.get_random_access_f32_reader()))
    }

    // TODO: Is TFile needed

    fn write_stream<T: Debug + PartialEq + Default + Clone + 'static, TFile: Debug + PartialEq + Default + Clone + 'static>(
        read_path: &Path,
        sample_format: SampleFormat,
        get_stream_reader: Box<dyn FnOnce(OpenWavReader<BufReader<File>>) -> Result<StreamWavReader<T>>>,
        write_all: Box<dyn FnOnce(OpenWavWriter, StreamWavReaderIterator<T>) -> Result<()>>,
        get_random_access_reader: Box<dyn Fn(OpenWavReader<BufReader<File>>) -> Result<RandomAccessWavReader<TFile>>>) {
        
        let read_path_buf = read_path.to_path_buf();
        
        test_with_file(Box::new(move |path| {
            let read_path = read_path_buf.as_path();
            let source_wav = read_wav_from_file_path(read_path)?;

            let header = WavHeader {
                sample_format,
                channels: source_wav.channels(),
                sample_rate: source_wav.sample_rate()
            };
            let open_wav = write_wav_to_file_path(path, header)?;

            let read_samples_iter = get_stream_reader(source_wav)?.into_iter();
            write_all(open_wav, read_samples_iter)?;

            let expected_wav = read_wav_from_file_path(read_path)?;
            let actual_wav = read_wav_from_file_path(path)?;

            assert_eq!(expected_wav.channels(), actual_wav.channels());
            assert_eq!(expected_wav.len_samples(), actual_wav.len_samples());

            let len_samples = expected_wav.len_samples();
            let channels = expected_wav.channels();

            let mut expected_wav_reader = get_random_access_reader(expected_wav)?;
            let mut actual_wav_reader = get_random_access_reader(actual_wav)?;

            for sample_ctr in 0..len_samples {
                for channel_ctr in 0..channels {
                    let expected_sample = expected_wav_reader.read_sample(sample_ctr, channel_ctr)?;
                    let actual_sample = actual_wav_reader.read_sample(sample_ctr, channel_ctr)?;

                    assert_eq!(
                        expected_sample,
                        actual_sample,
                        "Wrong value for sample");
                }
            }

            Ok(())
        }));
    }
}
