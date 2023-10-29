use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};

use super::OpenWavWriter;
use super::SampleFormat;
use super::WriteEx;
use crate::samples_by_channel::SamplesByChannel;

impl OpenWavWriter {
    pub fn write_all_i8<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<SamplesByChannel<i8>>>,
    {
        match self.header.sample_format {
            SampleFormat::Int8 => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8(value)),
            ),
            SampleFormat::Int16 => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8_as_i16(value)),
            ),
            SampleFormat::Int24 => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8_as_i24(value)),
            ),
            SampleFormat::Float => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i8| writer.write_i8_as_f32(value)),
            ),
        }
    }

    pub fn write_all_i16<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<SamplesByChannel<i16>>>,
    {
        match self.header.sample_format {
            SampleFormat::Int16 => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16(value)),
            ),
            SampleFormat::Int24 => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16_as_i24(value)),
            ),
            SampleFormat::Float => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i16| writer.write_i16_as_f32(value)),
            ),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 16-bit int unsupported",
            )),
        }
    }

    pub fn write_all_i24<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<SamplesByChannel<i32>>>,
    {
        match self.header.sample_format {
            SampleFormat::Int24 => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i32| writer.write_i24(value)),
            ),
            SampleFormat::Float => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: i32| writer.write_i24_as_f32(value)),
            ),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 24-bit int unsupported",
            )),
        }
    }

    pub fn write_all_f32<TIterator>(self, samples_itr: TIterator) -> Result<()>
    where
        TIterator: Iterator<Item = Result<SamplesByChannel<f32>>>,
    {
        match self.header.sample_format {
            SampleFormat::Float => self.write_all(
                samples_itr,
                Box::new(|mut writer: &mut dyn Write, value: f32| writer.write_f32(value)),
            ),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to float int unsupported",
            )),
        }
    }

    pub fn write_all<T, TIterator>(
        mut self,
        samples_itr: TIterator,
        write_sample_to_stream: Box<dyn Fn(&mut dyn Write, T) -> Result<()>>,
    ) -> Result<()>
    where
        TIterator: Iterator<Item = Result<SamplesByChannel<T>>>,
    {
        let position = self.data_start as u64;

        self.writer.seek(SeekFrom::Start(position as u64))?;

        self.chunk_size_written = false;

        let channels = self.header.channels.clone();

        for samples_result in samples_itr {
            if self.samples_written >= self.max_samples {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Wav files can only go up to 4GB.",
                ));
            }

            let samples_by_channel = samples_result?;

            if channels.front_left {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel.front_left.expect("Left channel missing"),
                )?;
            }
            if channels.front_right {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .front_right
                        .expect("Right channel missing"),
                )?;
            }
            if channels.front_center {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .front_center
                        .expect("Center channel missing"),
                )?;
            }
            if channels.low_frequency {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .low_frequency
                        .expect("Low frequency channel missing"),
                )?;
            }
            if channels.back_left {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .back_left
                        .expect("Back left channel missing"),
                )?;
            }
            if channels.back_right {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .back_right
                        .expect("Back right channel missing"),
                )?;
            }
            if channels.front_left_of_center {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .front_left_of_center
                        .expect("Front left of center channel missing"),
                )?;
            }
            if channels.front_right_of_center {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .front_right_of_center
                        .expect("Front right of center channel missing"),
                )?;
            }
            if channels.back_center {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .back_center
                        .expect("Back center channel missing"),
                )?;
            }
            if channels.side_left {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .side_left
                        .expect("Side left channel missing"),
                )?;
            }
            if channels.side_right {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .side_right
                        .expect("Side right channel missing"),
                )?;
            }
            if channels.top_center {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .top_center
                        .expect("Top center channel missing"),
                )?;
            }
            if channels.top_front_left {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .top_front_left
                        .expect("Top front left channel missing"),
                )?;
            }
            if channels.top_front_center {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .top_front_center
                        .expect("Top front center channel missing"),
                )?;
            }
            if channels.top_front_right {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .top_front_right
                        .expect("Top front right channel missing"),
                )?;
            }
            if channels.top_back_left {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .top_back_left
                        .expect("Top back left channel missing"),
                )?;
            }
            if channels.top_back_center {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .top_back_center
                        .expect("Top back center channel missing"),
                )?;
            }
            if channels.top_back_right {
                write_sample_to_stream(
                    &mut self.writer,
                    samples_by_channel
                        .top_back_right
                        .expect("Top back right channel missing"),
                )?;
            }

            self.samples_written += 1;
        }

        self.flush()?;
        Ok(())
    }
}
