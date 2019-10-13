use parse::tree::CompilationUnit;
use tokenize::span::Span;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token<'a> {
    Char(Span<'a>),
    Comment(Span<'a>),
    Double(Span<'a>),
    Float(Span<'a>),
    Int(Span<'a>),
    Long(Span<'a>),
    String(Span<'a>),
    Symbol(Span<'a>),
    Identifier(Span<'a>),
    Keyword(Span<'a>),
}

impl<'a> Token<'a> {
    pub fn span(&self) -> Span<'a> {
        let s = match self {
            Token::Symbol(s) => s,
            Token::Identifier(s) => s,
            Token::Keyword(s) => s,
            Token::Int(s) => s,
            Token::Double(s) => s,
            Token::Float(s) => s,
            Token::Char(s) => s,
            Token::Long(s) => s,
            Token::String(s) => s,
            Token::Comment(s) => s,
        };

        *s
    }
}
