// WARNING
// To thee, stranger, I confess: this module I have blatantly vibe-coded.
// Once I taketh this project in earnest, I will revisit it.


use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::collections::HashMap;

#[derive(Clone)]
pub struct PDF {
    doc: Document,
}

#[derive(Clone)]
pub struct Page {
    doc: Document,
    page_id: ObjectId,
}

pub enum ReadError {
    FileNotFound,
    InternalError,
}

pub struct OutOfBounds;

impl PDF {
    pub fn read(path: &str) -> Result<PDF, ReadError> {
        match Document::load(path) {
            Ok(doc) => Ok(PDF { doc }),
            Err(lopdf::Error::IO(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(ReadError::FileNotFound)
            }
            Err(_) => Err(ReadError::InternalError),
        }
    }

    pub fn from(pages: Vec<Page>) -> PDF {
        let mut doc = Document::with_version("1.5");
        let page_ids: Vec<ObjectId> = pages
            .into_iter()
            .map(|page| {
                let mut cache = HashMap::new();
                copy_object_deep(&page.doc, &mut doc, page.page_id, &mut cache)
            })
            .collect();
        finalize_document(&mut doc, page_ids);
        PDF { doc }
    }

    pub fn pages(&self) -> Vec<Page> {
        self.doc
            .get_pages()
            .into_values()
            .map(|id| self.extract_page(id))
            .collect()
    }

    pub fn page(&self, i: i32) -> Result<Page, OutOfBounds> {
        let index = self.validate_index(i)?;
        let ids: Vec<ObjectId> = self.doc.get_pages().into_values().collect();
        Ok(self.extract_page(ids[index]))
    }

    pub fn slice(&self, start: Option<i32>, end: Option<i32>, step: Option<i32>) -> Result<PDF, OutOfBounds> {
        let ids: Vec<ObjectId> = self.doc.get_pages().into_values().collect();
        let pages = ids.len() as i32;

        let step = if let Some(i) = step { i } else { 1 };
        let start = if let Some(i) = start { self.positive_index(i) as i32 } else { if step.is_positive() { 0 } else { pages - 1 } };
        let end = if let Some(i) = end { self.positive_index(i) as i32 } else { if step.is_positive() { pages } else { -1 } };

        // When the step is in the wrong direction, an empty PDF is returned
        if step.is_positive() != (end - start).is_positive() {
            return Ok(PDF::from(Vec::new()));
        }

        // I admit, this logic is ugly...
        let ids: Vec<ObjectId> = self.doc.get_pages().into_values().collect();
        let pages: Result<Vec<Page>, OutOfBounds> = if step.is_positive() {
            (start..end).step_by(step as usize).map(|i| -> Result<Page, OutOfBounds> {
                let i = self.validate_index(i)?;
                Ok(self.extract_page(ids[i as usize]))
            }).collect::<Result<Vec<Page>, OutOfBounds>>()
        } else {
            (end + 1..start + 1).rev().step_by(step.abs() as usize).map(|i| -> Result<Page, OutOfBounds> {
                let i = self.validate_index(i)?;
                Ok(self.extract_page(ids[i as usize]))
            }).collect::<Result<Vec<Page>, OutOfBounds>>()
        };

        Ok(PDF::from(pages?))
    }

    pub fn positive_index(&self, i: i32) -> usize {
        if i >= 0 {
            i as usize
        } else {
            let pages = self.doc.get_pages().len();
            (i + pages as i32) as usize
        }
    }

    pub fn validate_index(&self, mut i: i32) -> Result<usize, OutOfBounds> {
        let pages = self.doc.get_pages().len();

        if i < 0 {
            i += pages as i32;

            if i < 0 {
                return Err(OutOfBounds);
            }
        }

        let i = i as usize;
        if i >= pages {
            return Err(OutOfBounds);
        }

        Ok(i)
    }

    fn extract_page(&self, id: ObjectId) -> Page {
        let mut mini = Document::with_version(self.doc.version.clone());
        let mut cache = HashMap::new();
        let new_id = copy_object_deep(&self.doc, &mut mini, id, &mut cache);
        finalize_document(&mut mini, vec![new_id]);
        Page { doc: mini, page_id: new_id }
    }

    pub fn concatenate(&self, other: &PDF) -> PDF {
        let mut pages = self.pages();
        pages.extend(other.pages());
        PDF::from(pages)
    }

    pub fn write(&self, path: &str) {
        let mut doc = self.doc.clone();
        doc.save(path).expect("failed to write PDF");
    }
}

/// Recursively copies `id` and everything it references from `src` into `dst`,
/// returning the new object's ID in `dst`. Memoized to dedupe shared resources
/// and to terminate on cycles (e.g. /Parent pointers).
fn copy_object_deep(
    src: &Document,
    dst: &mut Document,
    id: ObjectId,
    cache: &mut HashMap<ObjectId, ObjectId>,
) -> ObjectId {
    if let Some(&existing) = cache.get(&id) {
        return existing;
    }
    let new_id = dst.new_object_id();
    cache.insert(id, new_id); // reserve before recursing, to break cycles

    let obj = src
        .get_object(id)
        .expect("dangling reference in source document")
        .clone();
    let copied = copy_value(src, dst, obj, cache);
    dst.objects.insert(new_id, copied);
    new_id
}

fn copy_value(
    src: &Document,
    dst: &mut Document,
    obj: Object,
    cache: &mut HashMap<ObjectId, ObjectId>,
) -> Object {
    match obj {
        Object::Reference(rid) => Object::Reference(copy_object_deep(src, dst, rid, cache)),
        Object::Array(items) => Object::Array(
            items.into_iter().map(|o| copy_value(src, dst, o, cache)).collect(),
        ),
        Object::Dictionary(d) => {
            let mut new_d = Dictionary::new();
            for (k, v) in d.iter() {
                new_d.set(k.clone(), copy_value(src, dst, v.clone(), cache));
            }
            Object::Dictionary(new_d)
        }
        Object::Stream(s) => {
            let mut new_dict = Dictionary::new();
            for (k, v) in s.dict.iter() {
                new_dict.set(k.clone(), copy_value(src, dst, v.clone(), cache));
            }
            Object::Stream(Stream::new(new_dict, s.content))
        }
        other => other, // numbers, strings, names, bools, null: copied as-is
    }
}

/// Wraps `page_ids` in a fresh /Pages + /Catalog and installs it as the trailer root.
fn finalize_document(doc: &mut Document, page_ids: Vec<ObjectId>) {
    let pages_id = doc.new_object_id();

    for pid in &page_ids {
        if let Ok(dict) = doc.get_object_mut(*pid).and_then(|o| o.as_dict_mut()) {
            dict.set("Parent", Object::Reference(pages_id));
        }
    }

    let kids: Vec<Object> = page_ids.iter().map(|id| Object::Reference(*id)).collect();
    let count = kids.len() as i32;
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => count,
        }),
    );

    let catalog_id = doc.new_object_id();
    doc.objects.insert(
        catalog_id,
        Object::Dictionary(dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(pages_id),
        }),
    );

    doc.trailer.set("Root", Object::Reference(catalog_id));
}

impl Page {
    pub fn blank() -> Page {
        let mut doc = Document::with_version("1.5");
        let page_id = doc.new_object_id();
        doc.objects.insert(page_id, Object::Dictionary(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()], // US Letter
            "Resources" => Dictionary::new(),
        }));
        Page { doc, page_id }
    }
}
