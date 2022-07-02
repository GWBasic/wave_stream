use std::io::{ Error, ErrorKind, Result, Seek, SeekFrom, Write };

use super::OpenWavWriter;
use super::RandomAccessWavWriter;
use super::SampleFormat;
use super::WriteEx;

impl OpenWavWriter {
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
}