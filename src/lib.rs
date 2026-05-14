use ffmpeg_next::{Dictionary, Rational, format, rescale::TIME_BASE};

pub mod convert;
pub mod error;
pub mod transcode;

pub use convert::Converter;
pub use error::{Result, TranscoderError};
pub use transcode::{AudioTranscoder, VideoTranscoder};

pub fn parse_opts(opts: &str) -> Result<Dictionary<'_>> {
    let mut dict = Dictionary::new();

    for part in opts.split(',').map(str::trim) {
        let (k, v) = part
            .split_once('=')
            .ok_or(TranscoderError::OptionParseError {
                part: part.to_owned(),
            })?;
        dict.set(k, v);
    }

    Ok(dict)
}

pub fn get_duration(ictx: &format::context::Input) -> f64 {
    let raw_dur = Rational::new(ictx.duration() as _, 1);
    let dur = raw_dur * TIME_BASE;

    dur.into()
}
