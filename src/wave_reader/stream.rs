use std::io::{ Error, ErrorKind, Read, Result };
use std::iter::IntoIterator;

use crate::OpenWavReader;
use crate::ReadEx;
use crate::SampleFormat;
use crate::StreamOpenWavReader;
use crate::StreamWavReader;
use crate::StreamWavReaderIterator;

impl<TReader: 'static + Read> StreamOpenWavReader for OpenWavReader<TReader> {
    fn get_stream_i8_reader(self) -> Result<StreamWavReader<i8>> {
        self.assert_int_8()?;

        Ok(StreamWavReader {
            open_wav: Box::new(self),
            read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i8())
        })
    }
    
    fn get_stream_i16_reader(self) -> Result<StreamWavReader<i16>> {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                Ok(StreamWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i16())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 16-bit unsupported"))
        }
    }

    fn get_stream_i24_reader(self) -> Result<StreamWavReader<i32>> {
        match self.header.sample_format {
            SampleFormat::Int24 => {
                Ok(StreamWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i24())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit unsupported"))
        }
    }

    fn get_stream_f32_reader(self) -> Result<StreamWavReader<f32>> {
        match self.header.sample_format {
            SampleFormat::Int8 => {
                Ok(StreamWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i8_as_f32())
                })
            },
            SampleFormat::Int16 => {
                Ok(StreamWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i16_as_f32())
                })
            },
            SampleFormat::Int24 => {
                Ok(StreamWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i24_as_f32())
                })
            },
            SampleFormat::Float => {
                Ok(StreamWavReader {
                    open_wav: Box::new(self),
                    read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_f32())
                })
            },
        }
    }
}

impl<T> StreamWavReader<T> {
    pub fn info(&self) -> &Box<dyn StreamOpenWavReader> {
        &self.open_wav
    }
}

impl<T> IntoIterator for StreamWavReader<T> {
    type Item = Result<Vec<T>>;
    type IntoIter = StreamWavReaderIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        StreamWavReaderIterator {
            open_wav: self.open_wav,
            read_sample_from_stream: self.read_sample_from_stream,
            current_sample: 0
        }
    }
}

impl<T> Iterator for StreamWavReaderIterator<T> {
    type Item = Result<Vec<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.open_wav.len_samples() {
            None
        } else {
            let num_channels: usize = self.open_wav.channels().into();
            let mut samples = Vec::new();

            for _channel in 0..num_channels {
                let read_result = (*self.read_sample_from_stream)(&mut self.open_wav.reader());
                let sample = match read_result {
                    Ok(sample) => sample,
                    Err(err) => {
                        self.current_sample = u32::MAX;
                        return Some(Err(err))
                    }
                };

                samples.push(sample);
            }

            self.current_sample += 1;

            Some(Ok(samples))
        }
    }
}