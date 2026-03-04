use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;

use anyhow::{Context, Result};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// Wraps a reqwest blocking response body as a symphonia MediaSource.
/// Uses Mutex to satisfy Sync requirement (only accessed from one thread).
struct HttpSource {
    reader: Mutex<Box<dyn Read + Send>>,
    position: u64,
    content_length: Option<u64>,
}

impl Read for HttpSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.reader.get_mut().unwrap().read(buf)?;
        self.position += n as u64;
        Ok(n)
    }
}

impl Seek for HttpSource {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "HTTP streams do not support seeking at the byte level",
        ))
    }
}

impl symphonia::core::io::MediaSource for HttpSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        self.content_length
    }
}

pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: usize,
    duration_secs: Option<f64>,
}

impl AudioDecoder {
    pub fn from_url(url: &str) -> Result<Self> {
        let response = reqwest::blocking::get(url).context("Failed to fetch audio stream")?;
        let content_length = response.content_length();

        let source = HttpSource {
            reader: Mutex::new(Box::new(response)),
            position: 0,
            content_length,
        };

        let mss = MediaSourceStream::new(Box::new(source), Default::default());

        let mut hint = Hint::new();
        // Try to infer format from URL
        if let Some(ext) = url.rsplit('.').next() {
            let ext_clean = ext.split('?').next().unwrap_or(ext);
            hint.with_extension(ext_clean);
        }

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .context("Failed to probe audio format")?;

        let format_reader = probed.format;

        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .context("No audio track found")?;

        let track_id = track.id;
        let codec_params = &track.codec_params;

        let sample_rate = codec_params.sample_rate.unwrap_or(44100);
        let channels = codec_params.channels.map(|c| c.count()).unwrap_or(2);

        let duration_secs = codec_params
            .n_frames
            .map(|frames| frames as f64 / sample_rate as f64);

        let decoder = symphonia::default::get_codecs()
            .make(codec_params, &DecoderOptions::default())
            .context("Failed to create decoder")?;

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration_secs,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn duration_secs(&self) -> Option<f64> {
        self.duration_secs
    }

    /// Decode the next packet, returning interleaved f32 samples.
    pub fn next_packet(&mut self) -> Option<Vec<f32>> {
        loop {
            let packet = match self.format_reader.next_packet() {
                Ok(p) => p,
                Err(_) => return None,
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d,
                Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                Err(_) => return None,
            };

            let spec = *decoded.spec();
            let num_frames = decoded.frames();

            let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec);
            sample_buf.copy_interleaved_ref(decoded);

            return Some(sample_buf.samples().to_vec());
        }
    }

    /// Attempt to seek to a position in seconds.
    pub fn seek(&mut self, position_secs: f64) -> Result<()> {
        let seek_to = SeekTo::Time {
            time: Time::from(position_secs),
            track_id: Some(self.track_id),
        };
        self.format_reader
            .seek(SeekMode::Coarse, seek_to)
            .context("Seek failed")?;
        self.decoder.reset();
        Ok(())
    }
}
