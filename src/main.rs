use pdfs::pdf::{PDF, ReadError};

fn main() {
    match PDF::read("test.pdf") {
        Ok(file) => file.write("output.pdf"),
        Err(ReadError::FileNotFound) => println!("couldn't find file"),
        Err(ReadError::InternalError) => println!("internal error")
    }
}
