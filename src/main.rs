extern crate juice;

use std::env;

#[derive(Debug, Default)]
struct Options {
    path: String,
    file: String,
    cflags: String,
}

fn parse_command_line() -> Result<Options, String> {
    let mut args = env::args();
    let mut path_arg = None;
    let mut file_arg = None;
    let mut cflags_arg = None;

    args.next();
    while let Some(arg) = args.next() {
        if arg == "-p" || arg == "--path" {
            path_arg = args.next()
        } else if arg == "-f" || arg == "--file" {
            file_arg = args.next()
        } else if arg == "-c" || arg == "--cflags" {
            cflags_arg = args.next()
        } else {
            return Err(format!("Unknown option: {}", arg));
        }
    }

    let mut options = Options::default();
    options.path = path_arg.ok_or(String::from("No path specified (`--path')."))?;
    options.file = file_arg.ok_or(String::from("No file specified (`--file')."))?;
    options.cflags = cflags_arg.ok_or(String::from("No compiler flags specified (`--cflags')."))?;
    Ok(options)
}

fn print_options(options: &Options) {
    println!("{:?}", options)
}

use juice::index;
use juice::tu;

fn run(options: &Options) -> Result<(), String> {
    let clang_args: Vec<_> = options.cflags.split_whitespace().collect();
    let index = index::Index::create(index::Options::empty());

    println!("Parsing source file...");

    let result =
        juice::tu::TranslationUnit::parse(&index, &options.file, clang_args, tu::Flags::empty());

    let tu = match result {
        Ok(tu) => tu,
        Err(error_code) => return Err(String::from(error_code.description())),
    };

    let diagnostics = tu.get_diagnostics();

    for diag in diagnostics {
        println!(">> {}", diag.spelling().to_str())
    }

    Ok(())
}

fn main() {
    println!("This is little!");

    let options = match parse_command_line() {
        Ok(options) => options,
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    print_options(&options);

    match run(&options) {
        Ok(_) => println!("Success."),
        Err(err) => println!("Error: {}", err),
    };
}
