use std::cmp;
use std::fmt::{self, Display, Debug};
use std::mem::{self, MaybeUninit};
use std::os::raw::{c_void, c_uint, c_int};
use std::ptr;

use fdk_aac_sys::enc;

pub use enc::InfoStruct;

pub struct EncoderError(enc::Error);

impl EncoderError {
    fn message(&self) -> &'static str {
        match self.0 {
            enc::Error::OK => "Ok",
            enc::Error::INVALID_HANDLE => "Handle passed to function call was invalid.",
            enc::Error::MEMORY_ERROR => "Memory allocation failed.",
            enc::Error::UNSUPPORTED_PARAMETER => "Parameter not available.",
            enc::Error::INVALID_CONFIG => "Configuration not provided.",
            enc::Error::INIT_ERROR => "General initialization error.",
            enc::Error::INIT_AAC_ERROR => "AAC library initialization error.",
            enc::Error::INIT_SBR_ERROR => "SBR library initialization error.",
            enc::Error::INIT_TP_ERROR => "Transport library initialization error.",
            enc::Error::INIT_META_ERROR => "Meta data library initialization error.",
            enc::Error::INIT_MPS_ERROR => "MPS library initialization error.",
            enc::Error::ENCODE_ERROR => "The encoding process was interrupted by an unexpected error.",
            enc::Error::ENCODE_EOF => "End of file reached.",
        }
    }
}

impl Debug for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EncoderError {{ code: {:?}, message: {:?} }}", self.0 as c_int, self.message())
    }
}

impl Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

fn check(e: enc::Error) -> Result<(), EncoderError> {
    if e == enc::Error::OK {
        Ok(())
    } else {
        Err(EncoderError(e))
    }
}

struct EncoderHandle {
    ptr: enc::HANDLE_AACENCODER,
}

impl EncoderHandle {
    pub fn alloc(max_modules: usize, max_channels: usize) -> Result<Self, EncoderError> {
        let mut ptr: enc::HANDLE_AACENCODER = ptr::null_mut();
        check(unsafe {
            enc::aacEncOpen(&mut ptr as *mut _, max_modules as c_uint, max_channels as c_uint)
        })?;
        Ok(EncoderHandle { ptr })
    }
}

impl Drop for EncoderHandle {
    fn drop(&mut self) {
        unsafe { enc::aacEncClose(&mut self.ptr as *mut _); }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BitRate {
    Cbr(u32),
    VbrVeryLow,
    VbrLow,
    VbrMedium,
    VbrHigh,
    VbrVeryHigh,
}

pub struct EncoderParams {
    pub bit_rate: BitRate,
    pub sample_rate: u32,
    pub transport: Transport,
}

pub struct Encoder {
    handle: EncoderHandle,
}

#[derive(Debug)]
pub enum Transport {
    Adts,
}

#[derive(Debug)]
pub struct EncodeInfo {
    pub input_consumed: usize,
    pub output_size: usize,
}

impl Encoder {
    pub fn new(params: EncoderParams) -> Result<Self, EncoderError> {
        let handle = EncoderHandle::alloc(0, 2 /* hardcode stereo */)?;

        unsafe {
            // hardcode MPEG-4 AAC Low Complexity for now:
            check(enc::aacEncoder_SetParam(handle.ptr, enc::Param::AOT, 2))?;

            let bitrate_mode = match params.bit_rate {
                BitRate::Cbr(bitrate) => {
                    check(enc::aacEncoder_SetParam(handle.ptr, enc::Param::BITRATE, bitrate))?;
                    0
                }
                BitRate::VbrVeryLow => 1,
                BitRate::VbrLow => 2,
                BitRate::VbrMedium => 3,
                BitRate::VbrHigh => 4,
                BitRate::VbrVeryHigh => 5,
            };

            check(enc::aacEncoder_SetParam(handle.ptr, enc::Param::BITRATEMODE, bitrate_mode))?;

            check(enc::aacEncoder_SetParam(handle.ptr, enc::Param::SAMPLERATE, params.sample_rate))?;

            match params.transport {
                Transport::Adts =>{
                    check(enc::aacEncoder_SetParam(handle.ptr, enc::Param::TRANSMUX, 2))?;
                }
            }

            // hardcode SBR off for now
            check(enc::aacEncoder_SetParam(handle.ptr, enc::Param::SBR_MODE, 0))?;

            // hardcode stereo
            check(enc::aacEncoder_SetParam(handle.ptr, enc::Param::CHANNELMODE, 2))?;

            // call encode once with all null params according to docs
            check(enc::aacEncEncode(handle.ptr, ptr::null(), ptr::null(), ptr::null(), ptr::null()))?;
        }

        Ok(Encoder { handle })
    }

    pub fn info(&self) -> Result<InfoStruct, EncoderError> {
        let mut info = MaybeUninit::uninit();
        check(unsafe { enc::aacEncInfo(self.handle.ptr, info.as_mut_ptr()) })?;
        Ok(unsafe { info.assume_init() })
    }

    pub fn encode(&self, input: &[i16], output: &mut [u8]) -> Result<EncodeInfo, EncoderError> {
        let input_len = cmp::min(i32::max_value() as usize, input.len()) as i32;

        let mut input_buf = input.as_ptr() as *mut i16;
        let mut input_buf_ident: c_int = enc::IN_AUDIO_DATA;
        let mut input_buf_size: c_int = input_len as c_int;
        let mut input_buf_el_size: c_int = mem::size_of::<i16>() as c_int;
        let input_desc = enc::BufDesc {
            num_bufs: 1,
            bufs: &mut input_buf as *mut _ as *mut *mut c_void,
            buffer_identifiers: &mut input_buf_ident as *mut c_int,
            buf_sizes: &mut input_buf_size as *mut c_int,
            buf_el_sizes: &mut input_buf_el_size as *mut c_int,
        };

        let mut output_buf = output.as_mut_ptr();
        let mut output_buf_ident: c_int = enc::OUT_BITSTREAM_DATA;
        let mut output_buf_size: c_int = output.len() as c_int;
        let mut output_buf_el_size: c_int = mem::size_of::<i16>() as c_int;
        let output_desc = enc::BufDesc {
            num_bufs: 1,
            bufs: &mut output_buf as *mut _ as *mut *mut c_void,
            buffer_identifiers: &mut output_buf_ident as *mut _,
            buf_sizes: &mut output_buf_size as *mut _,
            buf_el_sizes: &mut output_buf_el_size as *mut _,
        };

        let in_args = enc::InArgs {
            num_in_samples: input_len,
            num_anc_bytes: 0,
        };

        let mut out_args = unsafe { mem::zeroed() };

        check(unsafe { enc::aacEncEncode(self.handle.ptr, &input_desc, &output_desc, &in_args, &mut out_args) })?;

        Ok(EncodeInfo {
            output_size: out_args.num_out_bytes as usize,
            input_consumed: out_args.num_in_samples as usize,
        })
    }
}

impl Debug for Encoder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Encoder {{ handle: {:?} }}", self.handle.ptr)
    }
}
