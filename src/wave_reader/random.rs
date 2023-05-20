use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

use crate::samples_by_channel::SamplesByChannel;
use crate::OpenWavReader;
use crate::RandomAccessOpenWavReader;
use crate::RandomAccessWavReader;
use crate::ReadEx;
use crate::SampleFormat;

use super::private_parts;

impl<TReader: Read> private_parts::POpenWavReader for OpenWavReader<TReader> {
    fn data_start(&self) -> usize {
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
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i8()),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 8-bit unsupported",
            )),
        }
    }

    fn get_random_access_i16_reader(self) -> Result<RandomAccessWavReader<i16>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i8_as_i16()
                }),
            }),
            SampleFormat::Int16 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i16()),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 16-bit unsupported",
            )),
        }
    }

    fn get_random_access_i24_reader(self) -> Result<RandomAccessWavReader<i32>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i8_as_i24()
                }),
            }),
            SampleFormat::Int16 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i16_as_i24()
                }),
            }),
            SampleFormat::Int24 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_i24()),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 24-bit unsupported",
            )),
        }
    }

    fn get_random_access_f32_reader(self) -> Result<RandomAccessWavReader<f32>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i8_as_f32()
                }),
            }),
            SampleFormat::Int16 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i16_as_f32()
                }),
            }),
            SampleFormat::Int24 => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| {
                    reader.read_i24_as_f32()
                }),
            }),
            SampleFormat::Float => Ok(RandomAccessWavReader {
                open_wav: Box::new(self),
                read_sample_from_stream: Box::new(|mut reader: &mut dyn Read| reader.read_f32()),
            }),
        }
    }
}

impl<T> RandomAccessWavReader<T> {
    pub fn info(&self) -> &Box<dyn RandomAccessOpenWavReader> {
        &self.open_wav
    }

    pub fn read_sample(&mut self, sample: usize) -> Result<SamplesByChannel<T>> {
        if sample >= self.open_wav.len_samples() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Sample out of range"));
        }

        let sample_in_channels = sample * self.open_wav.num_channels() as usize;
        let sample_in_bytes = sample_in_channels * self.open_wav.bytes_per_sample() as usize;
        let position = self.open_wav.data_start() + sample_in_bytes;

        let seeker = self.open_wav.seeker();
        seeker.seek(SeekFrom::Start(position as u64))?;

        // Channels are cloned, because otherwise it holds an immutable borrow of self
        let channels = self.open_wav.channels().clone();

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

unsafe impl<T> Send for RandomAccessWavReader<T> {}
