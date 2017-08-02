#[macro_use]
extern crate bitflags;
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

pub mod juice;

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

    let index = juice::index::Index::create(
        juice::index::EXCLUDE_DECLARATIONS_FROM_PCH |
        juice::index::DISPLAY_DIAGNOSTICS);

    let mut trans_unit = juice::tu::TranslationUnit::new();

    unsafe {
        println!("Parsing source file...");

        let result = clang_sys::clang_parseTranslationUnit2(
            index.as_cx_index(),
            source_file_cstr.as_ptr(),
            clang_args_ptr_vec.as_ptr(),
            i32_try_from_usize(clang_args.len()).unwrap(),
            std::ptr::null_mut(),
            0,
            clang_sys::CXTranslationUnit_SkipFunctionBodies,
            trans_unit.as_mut_cx_translation_unit()
        );

        let cursor = clang_sys::clang_getTranslationUnitCursor(trans_unit.as_cx_translation_unit());
        clang_sys::clang_visitChildren(cursor, visit_cursor, std::ptr::null_mut());

        let error_code = juice::ErrorCode::from(result);

        match error_code {
            juice::ErrorCode::Success => Ok(()),
            _ => Err(String::from(error_code.description())),
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
