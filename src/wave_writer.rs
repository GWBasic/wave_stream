use std::io::{ Error, ErrorKind, Result, Seek, SeekFrom, Write };

use crate::WriteEx;
use crate::SampleFormat;
use crate::WavHeader;

pub trait WriteSeek : Write + Seek {}

impl<TWriteSeek: Write + Seek> WriteSeek for TWriteSeek {}

pub struct OpenWavWriter {
    writer: Box<dyn WriteSeek>,
    header: WavHeader,
    data_start: u32,
    chunk_size_written: bool,
    samples_written: u32
}

impl OpenWavWriter {
    pub fn new<TWriter: 'static + WriteSeek>(mut writer: TWriter, header: WavHeader) -> Result<OpenWavWriter> {
        writer.write_str("data")?;
        writer.write_u32(0)?;

        let data_start = writer.stream_position()? as u32;

        Ok(OpenWavWriter {
            writer: Box::new(writer),
            header,
            data_start,
            chunk_size_written: false,
            samples_written: 0
        })
    }

    fn assert_float(&self) -> Result<()> {
        if self.header.sample_format == SampleFormat::Float {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to float unsupported"))
        }
    }

    pub fn get_random_access_int_8_writer(self) -> Result<RandomAccessWavWriter<i8>> {
        match self.header.sample_format {
            SampleFormat::Int8 => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8(value))
                })
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 8-bit int unsupported"))
            }
        }
    }

    pub fn get_random_access_int_16_writer(self) -> Result<RandomAccessWavWriter<i16>> {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16(value))
                })
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 16-bit int unsupported"))
            }
        }
    }

    pub fn get_random_access_int_24_writer(self) -> Result<RandomAccessWavWriter<i32>> {
        match self.header.sample_format {
            SampleFormat::Int24 => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i32| writer.write_i24(value))
                })
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit int unsupported"))
            }
        }
    }

    pub fn get_random_access_float_writer(self) -> Result<RandomAccessWavWriter<f32>> {
        self.assert_float()?;

        Ok(RandomAccessWavWriter {
            open_wav: self,
            write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: f32| writer.write_f32(value))
        })
    }

    pub fn write_all_int_8<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<Vec<i8>>>
    {
        match self.header.sample_format {
            SampleFormat::Int8 => {
                self.write_all(
                    samples_itr,
                    Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8(value)))
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 8-bit int unsupported"))
            }
        }
    }

    pub fn write_all_int_16<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<Vec<i16>>>
    {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                self.write_all(
                    samples_itr,
                    Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16(value)))
                    },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 16-bit int unsupported"))
            }
        }
    }

    pub fn write_all_int_24<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<Vec<i32>>>
    {
        match self.header.sample_format {
            SampleFormat::Int24 => {
                self.write_all(
                    samples_itr,
                    Box::new(|mut writer: &mut dyn Write, value: i32| writer.write_i24(value)))
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit int unsupported"))
            }
        }
    }

    pub fn write_all_float<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<Vec<f32>>>
    {
        match self.header.sample_format {
            SampleFormat::Float => {
                self.write_all(
                    samples_itr,
                    Box::new(|mut writer: &mut dyn Write, value: f32| writer.write_f32(value)))
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to float int unsupported"))
            }
        }
    }

    pub fn write_all<T, TIterator>(
        mut self,
        samples_itr: TIterator,
        write_sample_to_stream: Box<dyn Fn(&mut dyn Write, T) -> Result<()>>) -> Result<()>
    where
        TIterator: Iterator<Item = Result<Vec<T>>>
    {
        let channels = self.channels() as usize;

        let position = self.data_start as u64;

        self.writer.seek(SeekFrom::Start(position as u64))?;
        
        self.chunk_size_written = false;

        for samples_result in samples_itr {
            let samples = samples_result?;

            if samples.len() != channels {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                     format!("Wrong number of channels in sample {}. Expected {}, got {}", self.samples_written, channels, samples.len())));
            }

            for value in samples {
                write_sample_to_stream(&mut self.writer, value)?;
            }


            self.samples_written += 1;
        }

        self.flush()?;
        Ok(())
    }

    pub fn sample_format(&self) -> SampleFormat {
        self.header.sample_format
    }

    pub fn channels(&self) -> u16 {
        self.header.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.header.sample_rate
    }

    pub fn bits_per_sample(&self) -> u16 {
        self.bytes_per_sample() * 8
    }

    pub fn bytes_per_sample(&self) -> u16 {
        match self.header.sample_format {
            SampleFormat::Float => 4,
            SampleFormat::Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        // data chunk
        let chunk_size = self.samples_written * (self.channels() * self.bytes_per_sample()) as u32;
        self.writer.seek(SeekFrom::Start(self.data_start as u64 - 4u64))?;
        self.writer.write_u32(chunk_size)?;

        // RIFF header
        self.writer.seek(SeekFrom::Start(4))?;
        self.writer.write_u32(chunk_size + 32 - 8)?;

        self.chunk_size_written = true;

        self.writer.flush()?;

        Ok(())
    }
}

impl Drop for OpenWavWriter {
    fn drop(&mut self) {
        if !self.chunk_size_written {
            self.flush().unwrap();
        }
    }
}

pub struct RandomAccessWavWriter<T> {
    open_wav: OpenWavWriter,
    write_sample_to_stream: Box<dyn Fn(&mut dyn Write, T) -> Result<()>>
}

impl<T> RandomAccessWavWriter<T> {
    pub fn info(&self) -> &OpenWavWriter {
        &(self.open_wav)
    }

    pub fn write_sample(&mut self, sample: u32, channel: u16, value: T) -> Result<()> {
        if channel >= self.open_wav.channels() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Channel out of range"));
        }

        // Pad the file if needed
        if sample >= self.open_wav.samples_written {
            self.open_wav.writer.seek(SeekFrom::End(0))?;

            let padding_size = (self.open_wav.samples_written - sample + 1) * (self.open_wav.channels() * self.open_wav.bytes_per_sample()) as u32;
            let padding = vec![0u8; 1];
            for _ in 0..padding_size {
                self.open_wav.writer.write(&padding)?;
            }
            self.open_wav.samples_written = sample + 1;
        }

        let sample_in_channels = (sample * self.open_wav.channels() as u32) + channel as u32;
        let sample_in_bytes = (sample_in_channels as u64) * (self.open_wav.bytes_per_sample() as u64);
        let position = (self.open_wav.data_start as u64) + sample_in_bytes;

        self.open_wav.writer.seek(SeekFrom::Start(position as u64))?;
        
        self.open_wav.chunk_size_written = false;
        (*self.write_sample_to_stream)(&mut self.open_wav.writer, value)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.open_wav.flush()
    }
}
