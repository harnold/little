#[macro_use]
extern crate bitflags;
extern crate clang_sys;
extern crate libc;

use std::any::Any;
use std::env;

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

    Ok(options)
}

fn print_options(options: &Options) {
    println!("path: {}", options.path);
    println!("file: {}", options.file);
}

pub mod juice;

struct EmptyData {}

fn print_declaration(
    cursor: juice::cursor::Cursor,
    _: juice::cursor::Cursor,
    _: &mut Any
) -> juice::cursor::ChildVisitResult {

    if cursor.kind().is_declaration() {
        let display_name = cursor.display_name();
        println!("{}", display_name.to_str());
    }

    juice::cursor::ChildVisitResult::Continue
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
            let cursor = tu.get_cursor();
            let mut data = EmptyData {};
            cursor.visit_children(print_declaration, &mut data);
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
