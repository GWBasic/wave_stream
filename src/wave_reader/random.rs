use std::io::{ Error, ErrorKind, Read, Result, Seek, SeekFrom };

use crate::OpenWavReader;
use crate::ReadEx;
use crate::SampleFormat;
use crate::RandomAccessWavReader;
use crate::RandomAccessOpenWavReader;

use super::private_parts;

impl<TReader: Read> private_parts::POpenWavReader for OpenWavReader<TReader> {
    fn data_start(&self) -> u32 {
        self.data_start
    }

    fn reader(&mut self) -> &mut (dyn Read) {
        &mut self.reader as &mut (dyn Read)
    }
}

impl<TReader: Read + Seek> private_parts::PRandomAccessOpenWavReader for OpenWavReader<TReader> {
    fn seeker(&mut self) -> &mut (dyn Seek) {
        &mut self.reader as &mut (dyn Seek)
    }
}

impl<TReader: 'static + Read + Seek> RandomAccessOpenWavReader for OpenWavReader<TReader> {
    fn get_random_access_i8_reader(self) -> Result<RandomAccessWavReader<i8>> {
        self.assert_int_8()?;

        Ok(RandomAccessWavReader {
            open_wav: Box::new(self),
            read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i8())
        })
    }

    fn get_random_access_i16_reader(self) -> Result<RandomAccessWavReader<i16>> {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                Ok(RandomAccessWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i16())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 16-bit unsupported"))
        }
    }

    fn get_random_access_i24_reader(self) -> Result<RandomAccessWavReader<i32>> {
        match self.header.sample_format {
            SampleFormat::Int24 => {
                Ok(RandomAccessWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i24())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit unsupported"))
        }
    }

    fn get_random_access_f32_reader(self) -> Result<RandomAccessWavReader<f32>> {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                Ok(RandomAccessWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i16_as_f32())
                })
            },
            SampleFormat::Int24 => {
                Ok(RandomAccessWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i24_as_f32())
                })
            },
            SampleFormat::Float => {
                Ok(RandomAccessWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_f32())
                })
                    },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit unsupported"))
        }
    }
}

impl<T> RandomAccessWavReader<T> {
    pub fn info(&self) -> &Box<dyn RandomAccessOpenWavReader> {
        &self.open_wav
    }

    pub fn read_sample(&mut self, sample: u32, channel: u16) -> Result<T> {
        if sample >= self.open_wav.len_samples() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Sample out of range"));
        }

        if channel >= self.open_wav.channels() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Channel out of range"));
        }

        let sample_in_channels = (sample * self.open_wav.channels() as u32) + channel as u32;
        let sample_in_bytes = sample_in_channels * self.open_wav.bytes_per_sample() as u32;
        let position = self.open_wav.data_start() + sample_in_bytes;

        let seeker = self.open_wav.seeker();
        seeker.seek(SeekFrom::Start(position as u64))?;

        let reader = self.open_wav.reader();
        (*self.read_sample_from_stream)(reader)
    }
}