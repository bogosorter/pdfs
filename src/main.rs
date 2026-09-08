use lopdf::{Document, Object};

fn main() {
    let mut file = Document::load("test.pdf").expect("couldn't open file");

    let mut ids: Vec<_> = file.get_pages().into_values().collect();
    ids.reverse();

    let pages_id = file.catalog().unwrap().get(b"Pages").unwrap().as_reference().unwrap();
    let pages_dict = file.get_object_mut(pages_id).unwrap().as_dict_mut().unwrap();
    pages_dict.set("Kids", ids.into_iter().map(Object::Reference).collect::<Vec<_>>());

    file.save("output.pdf").unwrap();
}
