use std::io::{ Error, ErrorKind, Read, Result };

use crate::open_wav::OpenWav;
use crate::ReadEx;
use crate::SampleFormat;
use crate::WavHeader;

pub struct OpenWavReader<TReader: Read> {
    reader: TReader,
    header: WavHeader,
    data_length: u32,
    data_start: u32,
}

impl<TReader : Read> OpenWav for OpenWavReader<TReader> {
    fn sample_format(&self) -> SampleFormat {
        self.header.sample_format
    }

    fn channels(&self) -> u16 {
        self.header.channels
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
            SampleFormat:: Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1
        }
    }

    fn len_samples(&self) -> u32 {
        self.data_length / (self.bytes_per_sample()) as u32 / self.header.channels as u32
    }
}

impl<TReader: 'static + Read> OpenWavReader<TReader> {
    pub fn new(mut reader: TReader, header: WavHeader, position: u32) -> Result<OpenWavReader<TReader>> {
        let mut data_start = position;
        'find_data_chunk: loop {
            let chunk_name = reader.read_str(4)?;
            data_start += 8;
            
            if chunk_name.eq("data") {
                break 'find_data_chunk;
            }

            let chunk_size = reader.read_u32()?;
            data_start += chunk_size;
            reader.skip(chunk_size as usize)?;
        }

        let data_length = reader.read_u32()?;

        Ok(OpenWavReader {
            reader,
            header,
            data_length,
            data_start
        })
    }

    fn assert_int_8(&self) -> Result<()> {
        if self.header.sample_format == SampleFormat::Int8 {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to 8-bit unsupported"))
        }
    }

    fn assert_float(&self) -> Result<()> {
        if self.header.sample_format == SampleFormat::Float {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to float unsupported"))
        }
    }
}

type ReadSampleFromStream<T> = fn(&mut dyn Read) -> Result<T>;

mod private_parts {
    use std::io::{ Read, Seek };

    pub trait POpenWavReader: super::OpenWav {
        fn data_start(&self) -> u32;
        fn reader(&mut self) -> &mut (dyn Read);
    }

    pub trait PRandomAccessOpenWavReader: POpenWavReader {
        fn seeker(&mut self) -> &mut (dyn Seek);
    }
}

pub trait StreamOpenWavReader: private_parts::POpenWavReader {
    fn get_stream_int_8_reader(self) -> Result<StreamWavReader<i8>>;
    fn get_stream_int_16_reader(self) -> Result<StreamWavReader<i16>>;
    fn get_stream_int_24_reader(self) -> Result<StreamWavReader<i32>>;
    fn get_stream_float_reader(self) -> Result<StreamWavReader<f32>>;
}

pub trait RandomAccessOpenWavReader: private_parts::PRandomAccessOpenWavReader {
    fn get_random_access_int_8_reader(self) -> Result<RandomAccessWavReader<i8>>;
    fn get_random_access_int_16_reader(self) -> Result<RandomAccessWavReader<i16>>;
    fn get_random_access_int_24_reader(self) -> Result<RandomAccessWavReader<i32>>;
    fn get_random_access_float_reader(self) -> Result<RandomAccessWavReader<f32>>;
}

pub struct RandomAccessWavReader<T> {
    open_wav: Box<dyn RandomAccessOpenWavReader>,
    read_sample_from_stream: Box<ReadSampleFromStream<T>>
}

pub struct StreamWavReader<T> {
    open_wav: Box<dyn StreamOpenWavReader>,
    read_sample_from_stream: Box<ReadSampleFromStream<T>>
}

pub struct StreamWavReaderIterator<T> {
    open_wav: Box<dyn StreamOpenWavReader>,
    read_sample_from_stream: Box<ReadSampleFromStream<T>>,
    current_sample: u32
}

mod random;
mod stream;
