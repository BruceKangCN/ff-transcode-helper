use ffmpeg::{Dictionary, Packet, Rational, codec, decoder, encoder, format, frame, picture};

use crate::error::{Result, TranscoderError};

pub trait Transcoder {
    fn new(
        encoder_name: &str,
        ist: &format::stream::Stream,
        octx: &mut format::context::Output,
        ost_index: usize,
        opts: Dictionary,
    ) -> Result<Self>
    where
        Self: Sized;

    fn transcode_packet(
        &mut self,
        octx: &mut format::context::Output,
        packet: &mut Packet,
        ost_time_base: Rational,
    ) -> Result<f64>;

    fn flush(&mut self, octx: &mut format::context::Output, ost_time_base: Rational) -> Result<()>;
}

pub struct VideoTranscoder {
    ost_index: usize,
    decoder: decoder::Video,
    input_time_base: Rational,
    encoder: encoder::Video,
}

impl Transcoder for VideoTranscoder {
    fn new(
        encoder_name: &str,
        ist: &format::stream::Stream,
        octx: &mut format::context::Output,
        ost_index: usize,
        opts: Dictionary,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        // put this before `octx.add_stream` to pass borrow checker
        let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);

        let decoder = codec::context::Context::from_parameters(ist.parameters())?
            .decoder()
            .video()?;

        let codec =
            encoder::find_by_name(encoder_name).ok_or(TranscoderError::InvalidEncoderError {
                name: encoder_name.to_owned(),
            })?;
        let mut ost = octx.add_stream(codec)?;

        let mut encoder = codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()?;
        let input_time_base = ist.time_base();
        ost.set_parameters(&encoder);
        encoder.set_height(decoder.height());
        encoder.set_width(decoder.width());
        encoder.set_aspect_ratio(decoder.aspect_ratio());
        encoder.set_format(decoder.format());
        encoder.set_frame_rate(decoder.frame_rate());
        encoder.set_time_base(input_time_base);

        if global_header {
            encoder.set_flags(codec::Flags::GLOBAL_HEADER);
        }

        let encoder = encoder.open_with(opts)?;
        ost.set_parameters(&encoder);

        Ok(Self {
            ost_index,
            decoder,
            input_time_base,
            encoder,
        })
    }

    fn transcode_packet(
        &mut self,
        octx: &mut format::context::Output,
        packet: &mut Packet,
        ost_time_base: Rational,
    ) -> Result<f64> {
        self.send_packet_to_decoder(packet)?;
        let pos = self.receive_and_process_decoded_frames(octx, ost_time_base)?;

        Ok(pos)
    }

    fn flush(&mut self, octx: &mut format::context::Output, ost_time_base: Rational) -> Result<()> {
        self.send_eof_to_decoder()?;
        self.receive_and_process_decoded_frames(octx, ost_time_base)?;
        self.send_eof_to_encoder()?;
        self.receive_and_process_encoded_packets(octx, ost_time_base)?;

        Ok(())
    }
}

impl VideoTranscoder {
    fn send_packet_to_decoder(&mut self, packet: &Packet) -> Result<()> {
        Ok(self.decoder.send_packet(packet)?)
    }

    fn send_eof_to_decoder(&mut self) -> Result<()> {
        Ok(self.decoder.send_eof()?)
    }

    fn receive_and_process_decoded_frames(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) -> Result<f64> {
        let mut frame = frame::Video::empty();
        let mut pos = 0.0;
        while self.decoder.receive_frame(&mut frame).is_ok() {
            let timestamp = frame.timestamp();

            if let Some(ts) = timestamp {
                let current_ts = Rational::new(ts as _, 1);
                pos = f64::from(current_ts * self.input_time_base);
            }

            frame.set_pts(timestamp);
            frame.set_kind(picture::Type::None);

            // TODO: implement filtering

            self.send_frame_to_encoder(&frame)?;
            self.receive_and_process_encoded_packets(octx, ost_time_base)?;
        }

        Ok(pos)
    }

    fn send_frame_to_encoder(&mut self, frame: &frame::Video) -> Result<()> {
        Ok(self.encoder.send_frame(frame)?)
    }

    fn send_eof_to_encoder(&mut self) -> Result<()> {
        Ok(self.encoder.send_eof()?)
    }

    fn receive_and_process_encoded_packets(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) -> Result<()> {
        let mut encoded = Packet::empty();

        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(self.ost_index);
            encoded.rescale_ts(self.input_time_base, ost_time_base);
            encoded.write_interleaved(octx)?;
        }

        Ok(())
    }
}

pub struct AudioTranscoder {
    // TODO
}

impl Transcoder for AudioTranscoder {
    fn new(
        encoder_name: &str,
        ist: &format::stream::Stream,
        octx: &mut format::context::Output,
        ost_index: usize,
        opts: Dictionary,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn transcode_packet(
        &mut self,
        octx: &mut format::context::Output,
        packet: &mut Packet,
        ost_time_base: Rational,
    ) -> Result<f64> {
        todo!()
    }

    fn flush(&mut self, octx: &mut format::context::Output, ost_time_base: Rational) -> Result<()> {
        todo!()
    }
}

impl AudioTranscoder {
    // TODO
}
