use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};

use super::OpenWavWriter;
use super::RandomAccessWavWriter;
use super::SampleFormat;
use super::WriteEx;
use crate::open_wav::OpenWav;
use crate::samples_by_channel::SamplesByChannel;

impl OpenWavWriter {
    pub fn get_random_access_i8_writer(self) -> Result<RandomAccessWavWriter<i8>> {
        match self.header.sample_format {
            SampleFormat::Int8 => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i8| {
                    writer.write_i8(value)
                }),
            }),
            SampleFormat::Int16 => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i8| {
                    writer.write_i8_as_i16(value)
                }),
            }),
            SampleFormat::Int24 => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i8| {
                    writer.write_i8_as_i24(value)
                }),
            }),
            SampleFormat::Float => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i8| {
                    writer.write_i8_as_f32(value)
                }),
            }),
        }
    }

    pub fn get_random_access_i16_writer(self) -> Result<RandomAccessWavWriter<i16>> {
        match self.header.sample_format {
            SampleFormat::Int16 => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i16| {
                    writer.write_i16(value)
                }),
            }),
            SampleFormat::Int24 => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i16| {
                    writer.write_i16_as_i24(value)
                }),
            }),
            SampleFormat::Float => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i16| {
                    writer.write_i16_as_f32(value)
                }),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 16-bit int unsupported",
            )),
        }
    }

    pub fn get_random_access_i24_writer(self) -> Result<RandomAccessWavWriter<i32>> {
        match self.header.sample_format {
            SampleFormat::Int24 => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i32| {
                    writer.write_i24(value)
                }),
            }),
            SampleFormat::Float => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: i32| {
                    writer.write_i24_as_f32(value)
                }),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 24-bit int unsupported",
            )),
        }
    }

    pub fn get_random_access_f32_writer(self) -> Result<RandomAccessWavWriter<f32>> {
        match self.header.sample_format {
            SampleFormat::Float => Ok(RandomAccessWavWriter {
                open_wav: self,
                write_sample_to_stream: Box::new(|mut writer: &mut dyn Write, value: f32| {
                    writer.write_f32(value)
                }),
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Converting to 32-bit float unsupported",
            )),
        }
    }
}

impl<T> RandomAccessWavWriter<T> {
    pub fn info(&self) -> &OpenWavWriter {
        &(self.open_wav)
    }

    pub fn write_samples(
        &mut self,
        sample: usize,
        samples_by_channel: SamplesByChannel<T>,
    ) -> Result<()> {
        if sample >= self.open_wav.header.max_samples {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Wav files can only go up to 4GB.",
            ));
        }

        // Pad the file if needed
        if sample >= self.open_wav.samples_written {
            self.open_wav.writer.seek(SeekFrom::End(0))?;

            let samples_to_pad = (sample + 1) - self.open_wav.samples_written;
            let padding_size = samples_to_pad
                * (self.open_wav.num_channels() * self.open_wav.bytes_per_sample()) as usize;
            let padding = vec![0u8; 1];
            for _ in 0..padding_size {
                self.open_wav.writer.write(&padding)?;
            }
            self.open_wav.samples_written = sample + 1;
        }

        let sample_in_channels = sample * self.open_wav.num_channels() as usize;
        let sample_in_bytes =
            (sample_in_channels as u64) * (self.open_wav.bytes_per_sample() as u64);
        let position = (self.open_wav.data_start as u64) + sample_in_bytes;

        self.open_wav
            .writer
            .seek(SeekFrom::Start(position as u64))?;

        self.open_wav.chunk_size_written = false;

        let channels = self.open_wav.channels().clone();
        if channels.front_left {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel.front_left.expect("Left channel missing"),
            )?;
        }
        if channels.front_right {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .front_right
                    .expect("Right channel missing"),
            )?;
        }
        if channels.front_center {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .front_center
                    .expect("Center channel missing"),
            )?;
        }
        if channels.low_frequency {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .low_frequency
                    .expect("Low frequency channel missing"),
            )?;
        }
        if channels.back_left {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .back_left
                    .expect("Back left channel missing"),
            )?;
        }
        if channels.back_right {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .back_right
                    .expect("Back right channel missing"),
            )?;
        }
        if channels.front_left_of_center {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .front_left_of_center
                    .expect("Front left of center channel missing"),
            )?;
        }
        if channels.front_right_of_center {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .front_right_of_center
                    .expect("Front right of center channel missing"),
            )?;
        }
        if channels.back_center {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .back_center
                    .expect("Back center channel missing"),
            )?;
        }
        if channels.side_left {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .side_left
                    .expect("Side left channel missing"),
            )?;
        }
        if channels.side_right {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .side_right
                    .expect("Side right channel missing"),
            )?;
        }
        if channels.top_center {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .top_center
                    .expect("Top center channel missing"),
            )?;
        }
        if channels.top_front_left {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .top_front_left
                    .expect("Top front left channel missing"),
            )?;
        }
        if channels.top_front_center {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .top_front_center
                    .expect("Top front center channel missing"),
            )?;
        }
        if channels.top_front_right {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .top_front_right
                    .expect("Top front right channel missing"),
            )?;
        }
        if channels.top_back_left {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .top_back_left
                    .expect("Top back left channel missing"),
            )?;
        }
        if channels.top_back_center {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .top_back_center
                    .expect("Top back center channel missing"),
            )?;
        }
        if channels.top_back_right {
            (*self.write_sample_to_stream)(
                &mut self.open_wav.writer,
                samples_by_channel
                    .top_back_right
                    .expect("Top back right channel missing"),
            )?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.open_wav.flush()
    }
}

unsafe impl<T> Send for RandomAccessWavWriter<T> {}
