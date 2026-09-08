use pdfs::pdf::{PDF, Page, ReadError};

fn main() {
    let file = match PDF::read("test.pdf") {
        Ok(file) => file,
        Err(ReadError::FileNotFound) => panic!("couldn't find file"),
        Err(ReadError::InternalError) => panic!("internal error")
    };

    let pages: Vec<Page> = file.pages().into_iter().step_by(2).collect();
    PDF::from(pages).write("output.pdf");
}
