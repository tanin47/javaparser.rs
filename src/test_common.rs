use parse::tpe::primitive::build_type_type;
use parse::tree::{CompilationUnit, PrimitiveType, Type};
use parse::{apply_tokens, Tokens};
use tokenize::span::Span;
use tokenize::token::Token;
use {tokenize, JavaFile};

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

pub fn generate_tokens(fragment: &str) -> Vec<Token> {
    tokenize::apply(fragment.trim(), std::ptr::null())
        .ok()
        .unwrap()
}

pub fn primitive(line: usize, col: usize, name: &str) -> Type {
    Type::Primitive(PrimitiveType {
        name: span(line, col, name),
        tpe: build_type_type(name).unwrap(),
    })
}

#[macro_export]
macro_rules! parse_files {
    (vec $sources:expr) => {{
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
    ($($source:expr),*) => {{
        parse_files!(vec vec![$($source),*])
    }};
}

#[macro_export]
macro_rules! assign_type_files {
    (vec $x:expr) => {{
        let files = parse_files!(vec $x);

        let mut units = vec![];
        for file in &files {
            units.push(&file.unit);
        }

        let mut root = ::analyze::resolve::merge(&units);
        ::analyze::resolve::assign_type::apply(&mut root);

        (files, root)
    }};
    ($($x:expr),*) => {{
        assign_type_files!(vec vec![$($x),*])
    }};
}

#[macro_export]
macro_rules! assign_parameterized_type_files {
    (vec $x:expr) => {{
        let (files, mut root) = assign_type_files!(vec $x);
        ::analyze::resolve::assign_parameterized_type::apply(&mut root);

        (files, root)
    }};
    ($($x:expr),*) => {{
        assign_parameterized_type_files!(vec vec![$($x),*])
    }};
}

#[macro_export]
macro_rules! semantics_files {
    (vec $x:expr) => {{
        let (files, root) = assign_parameterized_type_files!(vec $x);

        for file in &files {
            ::semantics::apply(&file.unit, &root);
        }

        (files, root)
    }};
    ($($x:expr),*) => {{
        semantics_files!(vec vec![$($x),*])
    }};
}

pub fn apply_analyze_build(source: &str) -> CompilationUnit {
    let tokens = generate_tokens(source);
    apply_tokens(&tokens).ok().unwrap()
}
