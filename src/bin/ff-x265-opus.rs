use std::env;

use anyhow::{Context, Result};
use ff_transcode_helper::{Converter, parse_opts};

fn main() -> Result<()> {
    ffmpeg_next::init().context("failed to init ffmpeg")?;

    let ext = "mp4";
    let v_encoder_name = "libx265";
    let a_encoder_name = "libopus";
    let v_opts = parse_opts("crf=23,preset=slow")?;
    let a_opts = parse_opts("b=256k")?;
    let v_filters = Some("format=yuv420p");
    let a_filters = Some("");

    let converter = Converter::new(
        ext,
        v_encoder_name,
        a_encoder_name,
        v_opts,
        a_opts,
        v_filters,
        a_filters,
    )?;

    let input = env::args().nth(1).expect("missing input");
    converter.convert(&input)?;

    Ok(())
}
