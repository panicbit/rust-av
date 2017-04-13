use ffi::AVCodecID;
use codec::MediaType;

error_chain! {
    errors {
        EncoderNotFound(name: String) {
            description("Could not find suitable encoder")
            display("Could not find encoder for {}", name)
        }

        DecoderNotFound(name: String) {
            description("Could not find suitable decoder")
            display("Could not find decoder for {}", name)
        }

        OpenEncoder(kind: &'static str) {
            description("Could not open encoder")
            display("Could not open {} encoder", kind)
        }

        OpenDecoder(kind: &'static str) {
            description("Could not open decoder")
            display("Could not open {} decoder", kind)
        }

        CopyCodecParameters

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
