use parse::tree::{CompilationUnit, PrimitiveType, Type};
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

pub fn parse(tokens: Tokens) -> CompilationUnit {
    match ::parse::apply(tokens) {
        Ok(compilation_unit) => compilation_unit,
        Err(_) => panic!("Error parsing: {:#?}", tokens[0]),
    }
}
pub fn primitive(line: usize, col: usize, name: &str) -> Type {
    Type::Primitive(PrimitiveType {
        name: span(line, col, name),
    })
}
