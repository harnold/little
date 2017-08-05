extern crate clang_sys;

use std::string;
use std::ffi;
use std::os::raw;

fn cstring_from_str(s: &str) -> ffi::CString {
    ffi::CString::new(s).unwrap()
}

fn i32_try_from_usize(x: usize) -> Result<i32, string::String> {
    if x <= i32::max_value() as usize {
        Ok(x as i32)
    } else {
        Err(string::String::from("Conversion failed."))
    }
}

fn i32_from_usize(x: usize) -> i32 {
    i32_try_from_usize(x).unwrap()
}

pub struct String {
    ptr: clang_sys::CXString,
}

impl String {
    pub fn from(s: clang_sys::CXString) -> String {
        String { ptr: s }
    }

    pub unsafe fn as_ptr(&self) -> *const raw::c_char {
        clang_sys::clang_getCString(self.ptr)
    }

    pub unsafe fn as_cstr(&self) -> &ffi::CStr {
        ffi::CStr::from_ptr(self.as_ptr())
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
        cx_index: clang_sys::CXIndex,
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

    use juice;
    use juice::index;

    use std;

    bitflags! {
        pub struct Flags: u32 {
            const NONE                                          = 0x00;
            const DETAILED_PREPROCESSING_RECORD                 = 0x01;
            const INCOMPLETE                                    = 0x02;
            const PRECOMPILED_PREAMBLE                          = 0x04;
            const CACHE_COMPLETION_RESULTS                      = 0x08;
            const FOR_SERIALIZATION                             = 0x10;
            const CXX_CHAINED_PCH                               = 0x20;
            const SKIP_FUNCTION_BODIES                          = 0x40;
            const INCLUDE_BRIEF_COMMENTS_IN_CODE_COMPLETIONS    = 0x80;
            // #[cfg(feature="gte_clang_3_8")]
            // const CREATE_PREAMBLE_ON_FIRST_PARSE                = 0x100;
            // #[cfg(feature="gte_clang_3_9")]
            // const KEEP_GOING                                    = 0x200;
        }
    }

    pub struct TranslationUnit {
        ptr: clang_sys::CXTranslationUnit,
    }

    impl TranslationUnit {
        pub fn from_ptr(ptr: clang_sys::CXTranslationUnit) -> TranslationUnit {
            TranslationUnit { ptr: ptr }
        }

        pub fn parse<'a, I>(
            index: &index::Index,
            source_filename: &str,
            command_line_args: I,
            // unsaved_file: ...
            flags: Flags,
        ) -> Result<TranslationUnit, juice::ErrorCode>
        where
            I: IntoIterator<Item = &'a str>,
        {

            let source_filename_cstr = juice::cstring_from_str(source_filename);
            let command_line_args_cstr = command_line_args.into_iter().map(|s| {
                juice::cstring_from_str(s)
            });
            let command_line_args_ptr_vec: Vec<_> =
                command_line_args_cstr.map(|s| s.as_ptr()).collect();

            let mut tu_ptr: clang_sys::CXTranslationUnit = std::ptr::null_mut();

            unsafe {
                let result = clang_sys::clang_parseTranslationUnit2(
                    index.as_cx_index(),
                    source_filename_cstr.as_ptr(),
                    command_line_args_ptr_vec.as_ptr(),
                    juice::i32_from_usize(command_line_args_ptr_vec.len()),
                    std::ptr::null_mut(),
                    0,
                    clang_sys::CXTranslationUnit_Flags::from_bits_truncate(flags.bits),
                    &mut tu_ptr,
                );

                let error_code = juice::ErrorCode::from(result);

                match error_code {
                    juice::ErrorCode::Success => Ok(TranslationUnit::from_ptr(tu_ptr)),
                    _ => Err(error_code),
                }
            }
        }

        pub fn as_ptr(&self) -> clang_sys::CXTranslationUnit {
            self.ptr
        }

        pub fn as_mut_ptr(&mut self) -> &mut clang_sys::CXTranslationUnit {
            &mut self.ptr
        }
    }

    impl Drop for TranslationUnit {
        fn drop(&mut self) {
            unsafe {
                clang_sys::clang_disposeTranslationUnit(self.ptr);
            }
        }
    }
}
