use std::io::{Error, ErrorKind, Read, Result};
use std::iter::IntoIterator;

use crate::samples_by_channel::SamplesByChannel;
use crate::OpenWavReader;
use crate::ReadEx;
use crate::SampleFormat;
use crate::StreamOpenWavReader;
use crate::StreamWavReader;
use crate::StreamWavReaderIterator;

impl<TReader: 'static + Read> StreamOpenWavReader for OpenWavReader<TReader> {
    fn get_stream_i8_reader(self) -> Result<StreamWavReader<i8>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i8()),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 8-bit unsupported",
            )),
        }
    }

    fn get_stream_i16_reader(self) -> Result<StreamWavReader<i16>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i8_as_i16()
                }),
            }),
            SampleFormat::Int16 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i16()),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 16-bit unsupported",
            )),
        }
    }

    fn get_stream_i24_reader(self) -> Result<StreamWavReader<i32>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i8_as_i24()
                }),
            }),
            SampleFormat::Int16 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i16_as_i24()
                }),
            }),
            SampleFormat::Int24 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i24()),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 24-bit unsupported",
            )),
        }
    }

    fn get_stream_f32_reader(self) -> Result<StreamWavReader<f32>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i8_as_f32()
                }),
            }),
            SampleFormat::Int16 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i16_as_f32()
                }),
            }),
            SampleFormat::Int24 => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i24_as_f32()
                }),
            }),
            SampleFormat::Float => Ok(StreamWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_f32()),
            }),
        }
    }
}

impl<T> StreamWavReader<T> {
    pub fn info(&self) -> &Box<dyn StreamOpenWavReader> {
        &self.open_wav
    }
}

impl<T> IntoIterator for StreamWavReader<T> {
    type Item = Result<SamplesByChannel<T>>;
    type IntoIter = StreamWavReaderIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        StreamWavReaderIterator {
            open_wav: self.open_wav,
            read_sample_from_stream: self.read_sample_from_stream,
            current_sample: 0,
        }
    }
}

impl<T> StreamWavReaderIterator<T> {
    fn read_samples(&mut self) -> Result<SamplesByChannel<T>> {
        // Channels are cloned, because otherwise it holds an immutable borrow of self
        let channels = self.open_wav.channels().clone();

        self.current_sample += 1;

        Ok(SamplesByChannel {
            front_left: if channels.front_left {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            front_right: if channels.front_right {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            front_center: if channels.front_center {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            low_frequency: if channels.low_frequency {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            back_left: if channels.back_left {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            back_right: if channels.back_right {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            front_left_of_center: if channels.front_left_of_center {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            front_right_of_center: if channels.front_right_of_center {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            back_center: if channels.back_center {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            side_left: if channels.side_left {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            side_right: if channels.side_right {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            top_center: if channels.top_center {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            top_front_left: if channels.top_front_left {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            top_front_center: if channels.top_front_center {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            top_front_right: if channels.top_front_right {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            top_back_left: if channels.top_back_left {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            top_back_center: if channels.top_back_center {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
            top_back_right: if channels.top_back_right {
                Some((*self.read_sample_from_stream)(
                    &mut self.open_wav.reader(),
                )?)
            } else {
                None
            },
        })
    }
}

impl<T> Iterator for StreamWavReaderIterator<T> {
    type Item = Result<SamplesByChannel<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.open_wav.len_samples() {
            None
        } else {
            Some(self.read_samples())
        }
    }
}
