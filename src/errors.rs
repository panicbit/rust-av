use ffi::AVCodecID;
use codec::MediaType;

error_chain! {
    errors {
        EncoderNotFound(codec_id: AVCodecID) {
            description("Could not find suitable encoder")
            // TODO: maybe use avcodec_get_name(codec_id)
            display("Could not find encoder for {:?}", codec_id)
        }

        DecoderNotFound(codec_id: AVCodecID) {
            description("Could not find suitable decoder")
            // TODO: maybe use avcodec_get_name(codec_id)
            display("Could not find decoder for {:?}", codec_id)
        }

        OpenEncoder(kind: &'static str) {
            description("Could not open encoder")
            display("Could not open {} encoder", kind)
        }

        EncodingUnsupported(codec_id: AVCodecID) {
            description("Codec does not support encoding")
            display("{:?} codec does not support encoding", codec_id)
        }

        MediaTypeMismatch(encoder_type: MediaType, codec_id: AVCodecID) {
            description("Encoder and codec media types mismatch")
            display("Cannot encode/decode {:?} using {:?} encoder/decoder", codec_id, encoder_type)
        }
    }
}
