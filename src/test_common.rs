use parse::tpe::primitive::build_type_type;
use parse::tree::{CompilationUnit, PrimitiveType, Type};
use parse::{JavaFile, Tokens};
use tokenize;
use tokenize::span::Span;
use tokenize::token::Token;

pub fn span(line: usize, col: usize, fragment: &str) -> Span {
    span2(line, col, fragment, std::ptr::null())
}

pub fn span2<'def>(
    line: usize,
    col: usize,
    fragment: &'def str,
    file: *const JavaFile<'def>,
) -> Span<'def> {
    Span {
        line,
        col,
        fragment,
        file,
    }
}

pub fn random_func() {}

pub fn generate_tokens(fragment: &str) -> Vec<Token> {
    tokenize::apply(fragment, std::ptr::null()).ok().unwrap()
}

pub fn primitive(line: usize, col: usize, name: &str) -> Type {
    Type::Primitive(PrimitiveType {
        name: span(line, col, name),
        tpe: build_type_type(name).unwrap(),
    })
}

#[macro_export]
macro_rules! collect_files {
    ($sources:expr) => {{
        let mut files = vec![];

        for (index, source) in $sources.iter().enumerate() {
            files.push(
                ::parse::apply(source.trim(), &format!("file{}.java", index))
                    .ok()
                    .unwrap(),
            );
        }

        files
    }};
}

#[macro_export]
macro_rules! semantics_files {
    (vec $x:expr) => {{
        let files = collect_files!($x);

        let mut units = vec![];
        for file in &files {
            units.push(&file.unit);
        }

        let root = ::analyze::resolve::apply(&units);

        for file in &files {
            ::semantics::apply(&file.unit, &root);
        }

        (files, root)
    }};
    ($($x:expr),*) => {{
        let files = collect_files!(vec![$($x),*]);

        let mut units = vec![];
        for file in &files {
            units.push(&file.unit);
        }

        let root = ::analyze::resolve::apply(&units);

        for file in &files {
            ::semantics::apply(&file.unit, &root);
        }

        (files, root)
    }};
}
