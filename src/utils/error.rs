use std::ops::Range;
use annotate_snippets::{Level, Renderer, Snippet, AnnotationKind};


pub struct Error {
    message: String,
    range: Range<usize>
}

impl Error {
    pub fn new(message: String, range: Range<usize>) -> Error {
        Error {
            message,
            range
        }
    }

    pub fn to_string(&self, path: &str, source: &str) -> String {
        let report = &[Level::ERROR
            .primary_title(self.message.clone())
            .element(
                Snippet::source(source)
                    .line_start(1)
                    .path(path)
                    .annotation(AnnotationKind::Primary.span(self.range.clone())),
            )];

        Renderer::styled().render(report)
    }
}
