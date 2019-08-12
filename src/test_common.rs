use parse::tree::{PrimitiveType, Type};
use parse::Tokens;
use tokenize;
use tokenize::span::Span;
use tokenize::token::Token;

pub fn span(line: usize, col: usize, fragment: &str) -> Span {
    Span {
        line,
        col,
        fragment,
    }
}

pub fn code(fragment: &str) -> Vec<Token> {
    tokenize::apply(fragment.trim()).ok().unwrap()
}

pub fn primitive(line: usize, col: usize, name: &str) -> Type {
    Type::Primitive(PrimitiveType {
        name: span(line, col, name),
    })
}
