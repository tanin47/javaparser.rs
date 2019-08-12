use tokenize::Span;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    Symbol(Span<'a>),
    Word(Span<'a>),
    Int(Span<'a>),
    Char(Span<'a>),
    Long(Span<'a>),
    Hex(Span<'a>),
    String(Span<'a>),
    Comment(Span<'a>),
}
