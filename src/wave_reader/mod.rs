use std::io::{Read, Result};

use crate::open_wav::OpenWav;
use crate::wave_header::Channels;
use crate::ReadEx;
use crate::SampleFormat;
use crate::WavHeader;

/// Represents an open wav file
pub struct OpenWavReader<TReader: Read> {
    reader: TReader,
    header: WavHeader,
    data_length: usize,
    data_start: usize,
}

impl<TReader: Read> OpenWav for OpenWavReader<TReader> {
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
        self.bytes_per_sample() * 8
    }

    fn bytes_per_sample(&self) -> u16 {
        match self.header.sample_format {
            SampleFormat::Float => 4,
            SampleFormat::Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1,
        }
    }

    fn len_samples(&self) -> usize {
        self.data_length
            / (self.bytes_per_sample() as usize)
            / (self.header.channels.count() as usize)
    }
}

impl<TReader: 'static + Read> OpenWavReader<TReader> {
    /// Creates a new OpenWavReader
    ///
    /// # Arguments
    ///
    /// * 'reader' - A Read struct. It is strongly recommended that this struct implement some form of buffering, such as via a BufReader
    /// * 'header' - The header that represents the sample rate and bit depth of the wav
    /// * 'position' - The current position of the reader
    pub fn new(
        mut reader: TReader,
        header: WavHeader,
        position: usize,
    ) -> Result<OpenWavReader<TReader>> {
        let mut data_start = position;
        'find_data_chunk: loop {
            let chunk_name = reader.read_str(4)?;
            data_start += 8;

            if chunk_name.eq("data") {
                break 'find_data_chunk;
            }

            let chunk_size = reader.read_u32()? as usize;
            data_start += chunk_size;
            reader.skip(chunk_size as usize)?;
        }

        let data_length = reader.read_u32()? as usize;

        Ok(OpenWavReader {
            reader,
            header,
            data_length,
            data_start,
        })
    }
}

type ReadSampleFromStream<T> = fn(&mut dyn Read) -> Result<T>;

mod private_parts {
    use std::io::{Read, Seek};

    pub trait POpenWavReader: super::OpenWav {
        fn data_start(&self) -> usize;
        fn reader(&mut self) -> &mut (dyn Read);
    }

    pub trait PRandomAccessOpenWavReader: POpenWavReader {
        fn seeker(&mut self) -> &mut (dyn Seek);
    }
}

/// An open streaming wav reader. Samples must be read in a sequential manner
pub trait StreamOpenWavReader: private_parts::POpenWavReader {
    /// Reads the wav as 8-bit samples. (Note that downsampling to 8-bit is not supported)
    fn get_stream_i8_reader(self) -> Result<StreamWavReader<i8>>;
    /// Reads the wav as 16-bit samples. (Note that downsampling to 16-bit is not supported)
    fn get_stream_i16_reader(self) -> Result<StreamWavReader<i16>>;
    /// Reads the wav as 24-bit samples. (Note that downsampling to 24-bit is not supported)
    fn get_stream_i24_reader(self) -> Result<StreamWavReader<i32>>;
    /// Reads the wav as floating point samples. All sample formats can be read as floats
    fn get_stream_f32_reader(self) -> Result<StreamWavReader<f32>>;
}

/// An open random-access wav reader. Samples may be read in a random-access manner
pub trait RandomAccessOpenWavReader: private_parts::PRandomAccessOpenWavReader {
    /// Reads the wav as 8-bit samples. (Note that downsampling to 8-bit is not supported)
    fn get_random_access_i8_reader(self) -> Result<RandomAccessWavReader<i8>>;
    /// Reads the wav as 16-bit samples. (Note that downsampling to 16-bit is not supported)
    fn get_random_access_i16_reader(self) -> Result<RandomAccessWavReader<i16>>;
    /// Reads the wav as 24-bit samples. (Note that downsampling to 24-bit is not supported)
    fn get_random_access_i24_reader(self) -> Result<RandomAccessWavReader<i32>>;
    /// Reads the wav as floating point samples. All sample formats can be read as floats
    fn get_random_access_f32_reader(self) -> Result<RandomAccessWavReader<f32>>;
}

/// An open random-access wav reader. Samples may be read in a random-access manner
pub struct RandomAccessWavReader<T> {
    open_wav: Box<dyn RandomAccessOpenWavReader>,
    read_sample_from_stream: Box<ReadSampleFromStream<T>>,
}

// An open streaming wav reader. Samples must be read in a sequential manner
pub struct StreamWavReader<T> {
    open_wav: Box<dyn StreamOpenWavReader>,
    read_sample_from_stream: Box<ReadSampleFromStream<T>>,
}

// An open streaming wav reader. Samples must be read in a sequential manner
pub struct StreamWavReaderIterator<T> {
    open_wav: Box<dyn StreamOpenWavReader>,
    read_sample_from_stream: Box<ReadSampleFromStream<T>>,
    current_sample: usize,
}

mod random;
mod stream;
