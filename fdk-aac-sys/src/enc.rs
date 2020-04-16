use std::os::raw::{c_void, c_int, c_uchar, c_uint};

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Error {
    OK = 0x0000,
    INVALID_HANDLE = 0x0020,
    MEMORY_ERROR = 0x0021,
    UNSUPPORTED_PARAMETER = 0x0022,
    INVALID_CONFIG = 0x0023,
    INIT_ERROR = 0x0040,
    INIT_AAC_ERROR = 0x0041,
    INIT_SBR_ERROR = 0x0042,
    INIT_TP_ERROR = 0x0043,
    INIT_META_ERROR = 0x0044,
    INIT_MPS_ERROR = 0x0045,
    ENCODE_ERROR = 0x0060,
    ENCODE_EOF = 0x0080,
}

pub const IN_AUDIO_DATA: c_int = 0;
pub const IN_ANCILLRY_DATA: c_int = 1;
pub const IN_METADATA_SETUP: c_int = 2;
pub const OUT_BITSTREAM_DATA: c_int = 3;
pub const OUT_AU_SIZES: c_int = 4;

#[allow(non_camel_case_types)]
pub type HANDLE_AACENCODER = *mut c_void;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Param {
    AOT = 0x0100,
    BITRATE = 0x0101,
    BITRATEMODE = 0x0102,
    SAMPLERATE = 0x0103,
    SBR_MODE = 0x0104,
    GRANULE_LENGTH = 0x0105,
    CHANNELMODE = 0x0106,
    CHANNELORDER = 0x0107,
    SBR_RATIO = 0x0108,
    AFTERBURNER = 0x0200,
    BANDWIDTH = 0x0203,
    PEAK_BITRATE = 0x0207,
    TRANSMUX = 0x0300,
    HEADER_PERIOD = 0x0301,
    SIGNALING_MODE = 0x0302,
    TPSUBFRAMES = 0x0303,
    AUDIOMUXVER = 0x0304,
    PROTECTION = 0x0306,
    ANCILLARY_BITRATE = 0x0500,
    METADATA_MODE = 0x0600,
    CONTROL_STATE = 0xFF00,
    NONE = 0xFFFF,
}


/// Provides some info about the encoder configuration.
#[repr(C)]
pub struct InfoStruct {
    /// Maximum number of encoder bitstream bytes within one
    /// frame. Size depends on maximum number of supported
    /// channels in encoder instance. For superframing (as
    /// used for example in DAB+), size has to be a multiple
    /// accordingly.
    pub max_out_buf_bytes: c_uint,

    /// Maximum number of ancillary data bytes which can be
    /// inserted into bitstream within one frame.
    pub max_anc_bytes: c_uint,

    /// Internal input buffer fill level in samples per
    /// channel. This parameter will automatically be cleared
    /// if samplingrate or channel(Mode/Order) changes.
    pub in_buf_fill_level: c_uint,

    /// Number of input channels expected in encoding
    /// process.
    pub input_channels: c_uint,

    /// Amount of input audio samples consumed each frame per
    /// channel, depending on audio object type configuration.
    pub frame_length: c_uint,

    /// Codec delay in PCM samples/channel. Depends on framelength
    /// and AOT. Does not include framing delay for filling up encoder
    /// PCM input buffer.
    pub n_delay: c_uint,

    /// Codec delay in PCM samples/channel, w/o delay caused by
    /// the decoder SBR module. This delay is needed to correctly
    /// write edit lists for gapless playback. The decoder may not
    /// know how much delay is introdcued by SBR, since it may not
    /// know if SBR is active at all (implicit signaling),
    /// therefore the deocder must take into account any delay
    /// caused by the SBR module.
    pub n_delay_core: c_uint,

    /// Configuration buffer in binary format as an
    /// AudioSpecificConfig or StreamMuxConfig according to the
    /// selected transport type.
    pub conf_buf: [c_uchar; 64],

    /// Number of valid bytes in confBuf.
    pub conf_size: c_uint,
}

/// Describes the input and output buffers for an aacEncEncode() call.
#[repr(C)]
pub struct BufDesc {
    /// Number of buffers.
    pub num_bufs: c_int,

    /// Pointer to vector containing buffer addresses.,
    pub bufs: *mut *mut c_void,

    /// Identifier of each buffer element. See
    /// ::AACENC_BufferIdentifier.
    pub buffer_identifiers: *mut c_int,

    /// Size of each buffer in 8-bit bytes.
    pub buf_sizes: *mut c_int,

    /// Size of each buffer element in bytes.
    pub buf_el_sizes: *mut c_int,
}

/// Defines the input arguments for an aacEncEncode() call.
#[repr(C)]
pub struct InArgs {
    /// Number of valid input audio samples (multiple of input
    /// channels).
    pub num_in_samples: c_int,

    /// Number of ancillary data bytes to be encoded.
    pub num_anc_bytes: c_int,
}

/// Defines the output arguments for an aacEncEncode() call.
#[repr(C)]
pub struct OutArgs {
    /// Number of valid bitstream bytes generated during
    /// aacEncEncode().
    pub num_out_bytes: c_int,

    /// Number of input audio samples consumed by the encoder.
    pub num_in_samples: c_int,

    /// Number of ancillary data bytes consumed by the encoder.
    pub num_anc_bytes: c_int,

    /// State of the bit reservoir in bits.
    pub bit_res_state: c_int,
}

extern "C" {
    pub fn aacEncOpen(
        encoder: *mut HANDLE_AACENCODER,
        enc_modules: c_uint,
        max_channels: c_uint,
    ) -> Error;

    pub fn aacEncClose(encoder: *mut HANDLE_AACENCODER) -> Error;

    pub fn aacEncEncode(
        encoder: HANDLE_AACENCODER,
        in_buf_desc: *const BufDesc,
        out_buf_desc: *const BufDesc,
        in_args: *const InArgs,
        out_args: *const OutArgs,
    ) -> Error;

    pub fn aacEncInfo(
        encoder: HANDLE_AACENCODER,
        info: *mut InfoStruct,
    ) -> Error;

    pub fn aacEncoder_SetParam(
        encoder: HANDLE_AACENCODER,
        param: Param,
        value: c_uint,
    ) -> Error;

    pub fn aacEncoder_GetParam(
        encoder: HANDLE_AACENCODER,
        param: Param,
    ) -> c_uint;
}
