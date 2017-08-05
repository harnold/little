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

fn parse_source_file(source_filename: String) -> Result<(), String> {

    let clang_args_str = "";
    let clang_args: Vec<_> = clang_args_str.split_whitespace().collect();

    let index = juice::index::Index::create(
        juice::index::EXCLUDE_DECLARATIONS_FROM_PCH |
        juice::index::DISPLAY_DIAGNOSTICS);

    println!("Parsing source file...");

    let result = juice::tu::TranslationUnit::parse(
        &index,
        &source_filename,
        clang_args,
        juice::tu::SKIP_FUNCTION_BODIES
    );

    match result {
        Ok(tu) => {
            unsafe {
                let cursor = clang_sys::clang_getTranslationUnitCursor(tu.as_ptr());
                clang_sys::clang_visitChildren(cursor, visit_cursor, std::ptr::null_mut());
            }
            Ok(())
        },
        Err(error_code) => {
            Err(String::from(error_code.description()))
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
