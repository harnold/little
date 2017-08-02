extern crate clang_sys;

use std::ffi;
use std::os::raw;

pub struct String {
    cx_str: clang_sys::CXString
}

impl String {
    pub fn from(s: clang_sys::CXString) -> String {
        String { cx_str: s }
    }

    pub unsafe fn as_ptr(&self) -> *const raw::c_char {
        clang_sys::clang_getCString(self.cx_str)
    }

    pub unsafe fn as_cstr(&self) -> &ffi::CStr {
        ffi::CStr::from_ptr(self.as_ptr())
    }
}

impl Drop for String {
    fn drop(&mut self) {
        unsafe {
            clang_sys::clang_disposeString(self.cx_str);
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
    pub fn from(e: clang_sys::CXErrorCode) -> ErrorCode {
        match e {
            clang_sys::CXError_Success => ErrorCode::Success,
            clang_sys::CXError_Failure => ErrorCode::Failure,
            clang_sys::CXError_Crashed => ErrorCode::Crashed,
            clang_sys::CXError_InvalidArguments => ErrorCode::InvalidArguments,
            clang_sys::CXError_ASTReadError => ErrorCode::ASTReadError,
            _ => ErrorCode::UnknownError,
        }
    }

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

pub mod index {

    extern crate clang_sys;

    bitflags! {
        pub struct Options: u32 {
            const EXCLUDE_DECLARATIONS_FROM_PCH = 0b00000001;
            const DISPLAY_DIAGNOSTICS           = 0b00000010;
        }
    }

    pub struct Index {
        cx_index: clang_sys::CXIndex
    }

    impl Index {
        pub fn create(o: Options) -> Index {
            let exclude_decls_from_pch =
                if o.contains(EXCLUDE_DECLARATIONS_FROM_PCH) { 1 } else { 0 };
            let display_diagnostics =
                if o.contains(DISPLAY_DIAGNOSTICS) { 1 } else { 0 };

            unsafe {
                Index {
                    cx_index: clang_sys::clang_createIndex(
                        exclude_decls_from_pch,
                        display_diagnostics)
                }
            }
        }

        pub fn as_cx_index(&self) -> clang_sys::CXIndex {
            self.cx_index
        }
    }

    impl Drop for Index {
        fn drop(&mut self) {
            unsafe {
                clang_sys::clang_disposeIndex(self.cx_index);
            }
        }
    }
}

pub mod tu {

    extern crate clang_sys;

    use std;

    pub struct TranslationUnit {
        cx_translation_unit: clang_sys::CXTranslationUnit
    }

    impl TranslationUnit {
        pub fn new() -> TranslationUnit {
            TranslationUnit { cx_translation_unit: std::ptr::null_mut() }
        }

        pub fn as_cx_translation_unit(&self) -> clang_sys::CXTranslationUnit {
            self.cx_translation_unit
        }

        pub fn as_mut_cx_translation_unit(&mut self) -> &mut clang_sys::CXTranslationUnit {
            &mut self.cx_translation_unit
        }
    }

    impl Drop for TranslationUnit {
        fn drop(&mut self) {
            unsafe {
                clang_sys::clang_disposeTranslationUnit(self.cx_translation_unit);
            }
        }
    }
}
