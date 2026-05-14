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

struct TimeDelta {
    pub hour: u64,
    pub minute: u64,
    pub second: f64,
}

impl From<f64> for TimeDelta {
    fn from(value: f64) -> Self {
        let mut rest = value;
        let h = (rest / 3600.0).floor();
        rest -= h * 3600.0;
        let m = (rest / 60.0).floor();
        rest -= m * 60.0;
        let second = rest;

        Self {
            hour: h as _,
            minute: m as _,
            second,
        }
    }
}

/// Format progress message in `%H:%M:%S.3f` format.
///
/// If `total` is less than 1 hour (3600 seconds), hour part will be omitted. If
/// `total` is less than 1 minute (60 seconds), minute part will also be omitted.
pub fn format_progress(current: f64, total: f64) -> String {
    let c = TimeDelta::from(current);
    let t = TimeDelta::from(total);

    let (c, t) = if total < 60.0 {
        let c = format!("{:5.3}", c.second);
        let t = format!("{:5.3}", t.second);

        (c, t)
    } else if total < 3600.0 {
        let c = format!("{}:{:05.3}", c.minute, c.second);
        let t = format!("{}:{:05.3}", t.minute, t.second);

        (c, t)
    } else {
        let c = format!("{}:{02}:{:05.3}", c.hour, c.minute, c.second);
        let t = format!("{}:{02}:{:05.3}", t.hour, t.minute, t.second);

        (c, t)
    };

    format!("{} / {}", c, t)
}
