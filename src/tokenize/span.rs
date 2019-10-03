use parse::tree::CompilationUnit;
use parse::JavaFile;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Span<'a> {
    pub line: usize,
    pub col: usize,
    pub fragment: &'a str,
    pub file: *const JavaFile<'a>,
}

pub trait CharAt {
    fn char_at(&self, i: usize) -> char;
}

impl CharAt for str {
    fn char_at(&self, i: usize) -> char {
        raw_char_at(i, self)
    }
}

fn raw_char_at(index: usize, s: &str) -> char {
    unsafe { *s.as_ptr().add(index) as char }
}
