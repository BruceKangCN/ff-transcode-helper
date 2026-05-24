use std::env;

use anyhow::{Context, Result};
use ff_transcode_helper::{Converter, parse_opts};

fn main() -> Result<()> {
    ffmpeg::init().context("failed to init ffmpeg")?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Error);

    let ext = "webm";
    let v_encoder_name = "libsvtav1";
    let a_encoder_name = "libopus";
    let v_opts_spec = [
        "crf=25",
        "pixel_format=yuv420p10le",
        "color_space=bt2020nc",
        "color_primaries=bt2020",
        "color_trc=smpte2084",
        "color_range=tv",
        "preset=4",
    ].join(",");
    let v_opts = parse_opts(&v_opts_spec)?;
    let a_opts = parse_opts("b=256k,sample_rate=48k,sample_fmt=s16")?;
    let v_filter_spec = Some("format=pix_fmts=yuv420p10le:color_ranges=tv");
    let a_filter_spec = Some("asetnsamples=960,aresample=48k,aformat=sample_fmts=s16");

    let converter = Converter::new(
        ext,
        v_encoder_name,
        a_encoder_name,
        v_opts,
        a_opts,
        v_filter_spec,
        a_filter_spec,
    )?;

    for input in env::args().skip(1) {
        converter.convert(&input)?;
    }

    Ok(())
}
