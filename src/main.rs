use pdfs::{parser::parse, type_checker::type_check, interpreter::interpret};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let parsed = parse(CODE)?;
    let typed = type_check(&parsed)?;
    interpret(&typed)?;

    Ok(())
}

// Simple copy-paste program
const CODE: &str = "
pdf = read('test.pdf');
write('output.pdf', pdf);
";
