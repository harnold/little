extern crate clang_sys;

use std::env;
use std::ffi;

#[derive(Debug, Default)]
struct Options {
    path: String,
    file: String,
}

fn parse_command_line() -> Result<Options, String> {

    let mut args = env::args();
    let mut path_arg = None;
    let mut file_arg = None;

    args.next();
    while let Some(arg) = args.next() {
        if arg == "-p" || arg == "--path" {
            path_arg = args.next();
        } else if arg == "-f" || arg == "--file" {
            file_arg = args.next();
        } else {
            return Err(format!("Unknown option: {}", arg));
        }
    }

    let mut options = Options::default();

    if let Some(path) = path_arg {
        options.path = path;
    } else {
        return Err(String::from("No path specified (`--path')."));
    }

    if let Some(file) = file_arg {
        options.file = file;
    } else {
        return Err(String::from("No file specified (`--file')."));
    }

    return Ok(options);
}

fn print_options(options: &Options) {
    println!("path: {}", options.path);
    println!("file: {}", options.file);
}

fn cstring_from_string(s: String) -> ffi::CString {
    ffi::CString::new(s).unwrap()
}

fn i32_try_from_usize(x: usize) -> Result<i32, String> {
    if x <= i32::max_value() as usize {
        Ok(x as i32)
    } else {
        Err(String::from("Conversion failed."))
    }
}

mod juice {

    extern crate clang_sys;

    use std::ffi;
    use std::os::raw;

    pub struct String {
        cxstr: clang_sys::CXString,
    }

    impl String {
        pub fn from(s: clang_sys::CXString) -> String {
            String { cxstr: s }
        }

        pub unsafe fn as_ptr(&self) -> *const raw::c_char {
            clang_sys::clang_getCString(self.cxstr)
        }

        pub unsafe fn as_cstr(&self) -> &ffi::CStr {
            ffi::CStr::from_ptr(self.as_ptr())
        }
    }

    impl Drop for String {
        fn drop(&mut self) {
            unsafe {
                clang_sys::clang_disposeString(self.cxstr);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn visit_cursor(
    cursor: clang_sys::CXCursor,
    _: clang_sys::CXCursor,
    _: clang_sys::CXClientData,
) -> clang_sys::CXVisitorResult {

    unsafe {
        let cursor_kind = clang_sys::clang_getCursorKind(cursor);

        if clang_sys::clang_isDeclaration(cursor_kind) != 0 {

            let display_name_jstr =
                juice::String::from(clang_sys::clang_getCursorDisplayName(cursor));
            let display_name_cstr = display_name_jstr.as_cstr();

            if let Ok(display_name) = display_name_cstr.to_str() {
                println!("{}", display_name);
            } else {
                println!("<unknown>");
            }
        }

        clang_sys::CXChildVisit_Continue
    }
}

fn parse_source_file(source_file: String) -> Result<(), String> {

    let source_file_cstr = cstring_from_string(source_file);

    let clang_args_str = "";
    let clang_args: Vec<_> = clang_args_str.split_whitespace().collect();
    let clang_args_cstr = clang_args.iter().map(
        |s| cstring_from_string(s.to_string()),
    );
    let clang_args_ptr_vec: Vec<_> = clang_args_cstr.map(|s| s.as_ptr()).collect();

    unsafe {
        let index = clang_sys::clang_createIndex(1, 1);
        let mut trans_unit: clang_sys::CXTranslationUnit = std::ptr::null_mut();

        println!("Parsing source file...");

        let result = clang_sys::clang_parseTranslationUnit2(
            index,
            source_file_cstr.as_ptr(),
            clang_args_ptr_vec.as_ptr(),
            i32_try_from_usize(clang_args.len()).unwrap(),
            std::ptr::null_mut(),
            0,
            clang_sys::CXTranslationUnit_SkipFunctionBodies,
            &mut trans_unit,
        );

        let cursor = clang_sys::clang_getTranslationUnitCursor(trans_unit);
        clang_sys::clang_visitChildren(cursor, visit_cursor, std::ptr::null_mut());

        clang_sys::clang_disposeTranslationUnit(trans_unit);
        clang_sys::clang_disposeIndex(index);

        match result {
            clang_sys::CXError_Success => Ok(()),
            clang_sys::CXError_Failure => Err(String::from("Failure.")),
            clang_sys::CXError_Crashed => Err(String::from("Crashed.")),
            clang_sys::CXError_InvalidArguments => Err(String::from("Invalid arguments.")),
            clang_sys::CXError_ASTReadError => Err(String::from("AST read error.")),
            _ => Err(String::from("Unknown error.")),
        }
    }
}

fn main() {
    println!("This is little!");

    let options = match parse_command_line() {
        Ok(options) => {
            print_options(&options);
            options
        }
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    match parse_source_file(options.file) {
        Ok(_) => println!("Success."),
        Err(err) => println!("Error: {}", err),
    };
}
