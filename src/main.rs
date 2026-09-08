use pdfs::{parser::parse, type_checker::type_check, interpreter::interpret};

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() != 2 {
        eprintln!("usage: pdfs [filename]");
        process::exit(1);
    }

    let mut file = match File::open(&arguments[1]) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("couldn't open {}", arguments[1]);
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

    let parsed = parse(&content)?;
    let typed = type_check(&parsed)?;
    interpret(&typed)?;

    Ok(())
}
