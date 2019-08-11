use syntax::tree::{PrimitiveType, Span, Type};

pub fn span(line: usize, col: usize, fragment: &str) -> Span {
    Span {
        line,
        col,
        fragment,
    }
}

pub fn code(fragment: &str) -> Span {
    Span {
        line: 1,
        col: 1,
        fragment,
    }
}

pub fn primitive(line: usize, col: usize, name: &str) -> Type {
    Type::Primitive(PrimitiveType {
        name: span(line, col, name),
    })
}
