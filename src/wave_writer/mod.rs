use std::io::{Result, Seek, SeekFrom, Write};

use crate::open_wav::OpenWav;
use crate::wave_header::Channels;
use crate::SampleFormat;
use crate::SampleFormatSize;
use crate::WavHeader;
use crate::WriteEx;

pub trait WriteSeek: Write + Seek {}

impl<TWriteSeek: Write + Seek> WriteSeek for TWriteSeek {}

/// An open wav writer
pub struct OpenWavWriter {
    writer: Box<dyn WriteSeek>,
    header: WavHeader,
    data_start: usize,
    chunk_size_written: bool,
    samples_written: usize,
    max_samples: usize,
}

/// An open random access wav writer
pub struct RandomAccessWavWriter<T> {
    open_wav: OpenWavWriter,
    write_sample_to_stream: Box<dyn Fn(&mut dyn Write, T) -> Result<()>>,
}

impl OpenWavWriter {
    /// Constructs a new wav writer
    ///
    /// * 'writer' - The (Write + Seek) struct to write the wav into. It is strongly recommended that this struct implement some form of buffering, such as via a BufWriter
    /// * 'header' - The header that represents the desired sampling rate and bit depth
    pub fn new<TWriter: 'static + WriteSeek>(
        writer: TWriter,
        header: WavHeader,
    ) -> Result<OpenWavWriter> {
        return OpenWavWriter::new_max_samples(writer, header, header.max_samples());
    }

    /// Intended to support testing max_samples
    pub(crate) fn new_max_samples<TWriter: 'static + WriteSeek>(
        mut writer: TWriter,
        header: WavHeader,
        max_samples: usize,
    ) -> Result<OpenWavWriter> {
        writer.write_str("data")?;
        writer.write_u32(0)?;

        let data_start = writer.stream_position()? as usize;

        Ok(OpenWavWriter {
            writer: Box::new(writer),
            header,
            data_start,
            chunk_size_written: false,
            samples_written: 0,
            max_samples,
        })
    }

    /// Flushes all buffered data to the stream
    pub fn flush(&mut self) -> Result<()> {
        // data chunk
        let chunk_size =
            self.samples_written * (self.num_channels() * self.bytes_per_sample()) as usize;
        self.writer
            .seek(SeekFrom::Start(self.data_start as u64 - 4u64))?;
        self.writer.write_u32(chunk_size as u32)?;

        // RIFF header
        self.writer.seek(SeekFrom::Start(4))?;
        self.writer.write_u32((chunk_size + 32 - 8) as u32)?;

        self.chunk_size_written = true;

        self.writer.flush()?;

        Ok(())
    }

    /// The maximum number of samples that can be written without exceeding the 4GB limit
    pub fn max_samples(&self) -> usize {
        self.max_samples
    }
}

impl OpenWav for OpenWavWriter {
    fn sample_format(&self) -> SampleFormat {
        self.header.sample_format
    }

    fn num_channels(&self) -> u16 {
        self.header.channels.count()
    }

    fn channels(&self) -> &Channels {
        &self.header.channels
    }

    fn sample_rate(&self) -> u32 {
        self.header.sample_rate
    }

    fn bits_per_sample(&self) -> u16 {
        self.header.sample_format.bits_per_sample()
    }

    fn bytes_per_sample(&self) -> u16 {
        self.header.sample_format.bytes_per_sample()
    }

    fn len_samples(&self) -> usize {
        self.samples_written
    }
}

impl Drop for OpenWavWriter {
    fn drop(&mut self) {
        if !self.chunk_size_written {
            self.flush().unwrap();
        }
    }
}

mod random;
mod stream;
