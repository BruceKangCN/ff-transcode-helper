use std::collections::HashMap;
use std::path::PathBuf;

use ffmpeg_next::{Dictionary, Rational, codec, encoder, format, media, rescale::TIME_BASE};
use indicatif::{ProgressBar, ProgressStyle};

use crate::error::{Result, TranscoderError};
use crate::transcode::{AudioTranscoder, Transcoder, VideoTranscoder};
use crate::{format_progress, get_duration, parse_opts};

#[derive(Debug)]
pub struct Converter<'a> {
    /// file extension without the separator (`.`).
    ///
    /// This is also used to determine output container format.
    ext: String,

    /// video encoder name
    v_encoder_name: String,

    /// audio encoder name
    a_encoder_name: String,

    /// video encoder options
    v_opts: Dictionary<'a>,

    /// audio encoder options
    a_opts: Dictionary<'a>,

    /// video filters
    v_filters: Option<String>,

    /// audio filters
    a_filters: Option<String>,
}

struct TaskConfig {
    stream_mapping: Vec<isize>,
    ist_time_bases: Vec<Rational>,
    ost_time_bases: Vec<Rational>,
    transcoders: HashMap<usize, Box<dyn Transcoder>>,
}

impl<'a> Converter<'a> {
    pub fn new(
        ext: &str,
        v_encoder_name: &str,
        a_encoder_name: &str,
        v_opts: Dictionary<'a>,
        a_opts: Dictionary<'a>,
        v_filters: Option<&str>,
        a_filters: Option<&str>,
    ) -> Result<Self> {
        let ext = ext.to_owned();
        let v_encoder_name = v_encoder_name.to_owned();
        let a_encoder_name = a_encoder_name.to_owned();
        let v_filters = v_filters.map(|s| s.to_owned());
        let a_filters = a_filters.map(|s| s.to_owned());

        Ok(Self {
            ext,
            v_encoder_name,
            a_encoder_name,
            v_opts,
            a_opts,
            v_filters,
            a_filters,
        })
    }

    pub fn convert(&self, input: &str) -> Result<()> {
        let input_path = PathBuf::from(input);

        let mut output_path = input_path
            .file_stem()
            .ok_or(TranscoderError::NotAFileError {
                path: input_path.to_owned(),
            })?
            .to_owned();
        output_path.push(format!(".{}", &self.ext));
        let output_path = PathBuf::from(output_path);

        let mut ictx = format::input(&input_path)?;
        let mut octx = format::output(&output_path)?;

        // format::context::input::dump(&ictx, 0, Some(&input));

        let mut config = self.write_header(&ictx, &mut octx)?;

        // format::context::output::dump(&octx, 0, output_path.to_str());

        let pb = ProgressBar::new(ictx.duration() as _);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        let total_dur = get_duration(&ictx);
        pb.set_message(format_progress(0.0, total_dur));

        for (stream, mut packet) in ictx.packets() {
            let ist_index = stream.index();
            let ost_index = config.stream_mapping[ist_index];
            if ost_index < 0 {
                continue;
            }

            let ist_time_base = config.ist_time_bases[ist_index];
            let ost_time_base = config.ost_time_bases[ost_index as usize];

            match config.transcoders.get_mut(&ist_index) {
                Some(transcoder) => {
                    let pos = transcoder.transcode_packet(&mut octx, &mut packet, ost_time_base)?;
                    let ts = pos / f64::from(TIME_BASE);
                    pb.set_position(ts as _);
                    pb.set_message(format_progress(pos, total_dur));
                }
                None => {
                    // Do stream copy on other streams
                    packet.rescale_ts(ist_time_base, ost_time_base);
                    packet.set_position(-1);
                    packet.set_stream(ost_index as _);
                    packet.write_interleaved(&mut octx)?;
                }
            }
        }

        pb.finish();

        // Flush encoders and decoders.
        for (ost_index, transcoder) in config.transcoders.iter_mut() {
            let ost_time_base = config.ost_time_bases[*ost_index];
            transcoder.flush(&mut octx, ost_time_base)?;
        }

        octx.write_trailer()?;

        Ok(())
    }

    pub fn set_v_opts(&mut self, opts: &'a str) -> Result<()> {
        self.v_opts = parse_opts(opts)?;

        Ok(())
    }

    pub fn set_a_opts(&mut self, opts: &'a str) -> Result<()> {
        self.a_opts = parse_opts(opts)?;

        Ok(())
    }

    pub fn update_v_opts(&mut self, opts: &'a str) -> Result<()> {
        for (k, v) in parse_opts(opts)?.into_iter() {
            self.v_opts.set(k, v);
        }

        Ok(())
    }

    pub fn update_a_opts(&mut self, opts: &'a str) -> Result<()> {
        for (k, v) in parse_opts(opts)?.into_iter() {
            self.a_opts.set(k, v);
        }

        Ok(())
    }

    /// Get metadata for the output context, and write file header.
    ///
    /// Also get stream mapping, input/outpu stream time bases, encoder
    /// configurations and retrun them as a `TaskConfig`.
    fn write_header(
        &self,
        ictx: &format::context::Input,
        octx: &mut format::context::Output,
    ) -> Result<TaskConfig> {
        let nb_streams = ictx.nb_streams() as usize;

        let mut stream_mapping = vec![0isize; nb_streams];
        let mut ist_time_bases = vec![Rational(0, 0); nb_streams];
        let mut ost_time_bases = vec![Rational(0, 0); nb_streams];
        let mut transcoders = HashMap::<usize, Box<dyn Transcoder>>::new();

        let mut ost_index = 0;
        for (ist_index, ist) in ictx.streams().enumerate() {
            let ist_medium = ist.parameters().medium();

            if ist_medium != media::Type::Audio
                && ist_medium != media::Type::Video
                && ist_medium != media::Type::Subtitle
            {
                stream_mapping[ist_index] = -1;
                continue;
            }

            stream_mapping[ist_index] = ost_index;
            ist_time_bases[ist_index] = ist.time_base();
            match ist_medium {
                media::Type::Video => {
                    let transcoder = VideoTranscoder::new(
                        &self.v_encoder_name,
                        &ist,
                        octx,
                        ost_index as _,
                        self.v_opts.to_owned(),
                    )?;
                    transcoders.insert(ist_index, Box::new(transcoder));
                }
                // TODO
                // media::Type::Audio => {
                //     let transcoder = AudioTranscoder::new(
                //         &self.a_encoder_name,
                //         &ist,
                //         octx,
                //         ost_index as _,
                //         self.a_opts.to_owned(),
                //     )?;
                //     transcoders.insert(ist_index, Box::new(transcoder));
                // }
                _ => {
                    // Setup for stream copy for non-video and non-audio streams.
                    let mut ost = octx.add_stream(encoder::find(codec::Id::None))?;
                    ost.set_parameters(ist.parameters());
                    // We need to set codec_tag to 0 lest we run into incompatible
                    // codec tag issues when muxing into a different container
                    // format. Unfortunately there's no high level API to do this
                    // (yet).
                    unsafe {
                        (*ost.parameters().as_mut_ptr()).codec_tag = 0;
                    }
                }
            }

            ost_index += 1;
        }

        octx.set_metadata(ictx.metadata().to_owned());
        octx.write_header()?;

        for ost_index in 0..octx.nb_streams() {
            ost_time_bases[ost_index as usize] = octx.stream(ost_index as _).unwrap().time_base();
        }

        Ok(TaskConfig {
            stream_mapping,
            ist_time_bases,
            ost_time_bases,
            transcoders,
        })
    }
}
