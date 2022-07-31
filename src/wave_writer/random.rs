use std::io::{ Error, ErrorKind, Result, Seek, SeekFrom, Write };

use crate::open_wav::OpenWav;
use super::OpenWavWriter;
use super::RandomAccessWavWriter;
use super::SampleFormat;
use super::WriteEx;

impl OpenWavWriter {
    pub fn get_random_access_i8_writer(self) -> Result<RandomAccessWavWriter<i8>> {
        match self.header.sample_format {
            SampleFormat::Int8 => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8(value))
                })
            },
            SampleFormat::Float => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8_as_f32(value))
                })
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 8-bit int unsupported"))
            }
        }
    }

    pub fn get_random_access_i16_writer(self) -> Result<RandomAccessWavWriter<i16>> {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16(value))
                })
            },
            SampleFormat::Int24 => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16_as_i24(value))
                })
            },
            SampleFormat::Float => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16_as_f32(value))
                })
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 16-bit int unsupported"))
            }
        }
    }

    pub fn get_random_access_i24_writer(self) -> Result<RandomAccessWavWriter<i32>> {
        match self.header.sample_format {
            SampleFormat::Int24 => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i32| writer.write_i24(value))
                })
            },
            SampleFormat::Float => {
                Ok(RandomAccessWavWriter {
                    open_wav: self,
                    write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i32| writer.write_i24_as_f32(value))
                })
            },
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit int unsupported"))
            }
        }
    }

    pub fn get_random_access_f32_writer(self) -> Result<RandomAccessWavWriter<f32>> {
        self.assert_float()?;

        Ok(RandomAccessWavWriter {
            open_wav: self,
            write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: f32| writer.write_f32(value))
        })
    }
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