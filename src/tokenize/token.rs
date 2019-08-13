use tokenize::Span;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    Char(Span<'a>),
    Comment(Span<'a>),
    Double(Span<'a>),
    Float(Span<'a>),
    Hex(Span<'a>),
    Int(Span<'a>),
    Long(Span<'a>),
    String(Span<'a>),
    Symbol(Span<'a>),
    Word(Span<'a>),
}

impl<'a> Token<'a> {
    pub fn span(&self) -> Span<'a> {
        let s = match self {
            Token::Symbol(s) => s,
            Token::Word(s) => s,
            Token::Int(s) => s,
            Token::Double(s) => s,
            Token::Float(s) => s,
            Token::Char(s) => s,
            Token::Long(s) => s,
            Token::Hex(s) => s,
            Token::String(s) => s,
            Token::Comment(s) => s,
        };

        *s
    }
}
