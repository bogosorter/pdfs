use pdfs::{parser::parse, type_checker::type_check, interpreter::interpret};

use std::fs::File;
use std::io::Read;
use std::process;
use std::env;

fn main() {
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() != 2 {
        eprintln!("usage: pdfs [filename]");
        process::exit(1);
    }

    let path = &arguments[1];

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("couldn't open {}", path);
            process::exit(1)
        }
    };

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => {},
        Err(_) => {
            eprintln!("couldn't read {}", arguments[1]);
            process::exit(1)
        }
    }

    let parsed = match parse(&content) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{}", error.to_string(path, &content));
            process::exit(1);
        }
    };

    let typed = match type_check(&parsed) {
        Ok(typed) => typed,
        Err(error) => {
            eprintln!("{}", error.to_string(path, &content));
            process::exit(1);
        }
    };

    match interpret(&typed) {
        Ok(_) => {},
        Err(error) => {
            eprintln!("{}", error.to_string(path, &content));
            process::exit(1);
        }
    }
}
