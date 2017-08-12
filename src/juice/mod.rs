pub mod index;
pub mod tu;
mod util;

use clang_sys;
use std::ffi;
use std::os::raw;

pub struct String {
    ptr: clang_sys::CXString
}

impl String {
    pub unsafe fn as_ptr(&self) -> *const raw::c_char {
        clang_sys::clang_getCString(self.ptr)
    }

    pub unsafe fn as_cstr(&self) -> &ffi::CStr {
        ffi::CStr::from_ptr(self.as_ptr())
    }
}

impl From<clang_sys::CXString> for String {
    fn from(s: clang_sys::CXString) -> String {
        String { ptr: s }
    }
}

impl Drop for String {
    fn drop(&mut self) {
        unsafe {
            clang_sys::clang_disposeString(self.ptr);
        }
    }
}

#[derive(Debug)]
pub enum ErrorCode {
    Success,
    Failure,
    Crashed,
    InvalidArguments,
    ASTReadError,
    UnknownError,
}

impl ErrorCode {
    pub fn description(&self) -> &str {
        match *self {
            ErrorCode::Success => "Success.",
            ErrorCode::Failure => "Generic error code.",
            ErrorCode::Crashed => "libclang crashed.",
            ErrorCode::InvalidArguments => "Invalid arguments.",
            ErrorCode::ASTReadError => "AST deserialization error.",
            ErrorCode::UnknownError => "Unknown error.",
        }
    }
}

impl From<clang_sys::CXErrorCode> for ErrorCode {
    fn from(e: clang_sys::CXErrorCode) -> ErrorCode {
        match e {
            clang_sys::CXError_Success => ErrorCode::Success,
            clang_sys::CXError_Failure => ErrorCode::Failure,
            clang_sys::CXError_Crashed => ErrorCode::Crashed,
            clang_sys::CXError_InvalidArguments => ErrorCode::InvalidArguments,
            clang_sys::CXError_ASTReadError => ErrorCode::ASTReadError,
            _ => ErrorCode::UnknownError,
        }
    }
}
