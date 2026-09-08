pub struct PDF;
pub struct Page;

pub enum ReadError {
    FileNotFound,
    InternalError
}

impl PDF {
    pub fn read(path: &str) -> Result<PDF, ReadError> {
        panic!("read is not implemented");
    }

    pub fn from(pages: Vec<Page>) -> PDF {
        panic!("from is not implemented");
    }

    pub fn pages(&self) -> Vec<Page> {
        panic!("pages is not implemented");
    }

    pub fn page(&self, i: usize) -> Page {
        panic!("page is not implemented");
    }

    pub fn concatenate(&self, other: &PDF) -> PDF {
        panic!("concatenate is not implemented")
    }

    pub fn write(&self, path: &str) {
        panic!("write is not implemented");
    }
}
