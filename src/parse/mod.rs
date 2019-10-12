use parse::id_gen::IdGen;
use parse::tree::CompilationUnit;
use std::borrow::Borrow;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::null;
use tokenize::span::Span;
use tokenize::token::Token;
use {tokenize, JavaFile};

pub mod combinator;
pub mod compilation_unit;
pub mod def;
pub mod expr;
pub mod id_gen;
pub mod statement;
pub mod tpe;
pub mod tree;

pub type Tokens<'def, 'r> = &'r [Token<'def>];
pub type ParseResult<'def, 'r, T> = Result<(Tokens<'def, 'r>, T), Tokens<'def, 'r>>;

pub fn apply_tokens<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> Result<CompilationUnit<'def>, Tokens<'def, 'r>> {
    let result = compilation_unit::parse(input, id_gen);

    match result {
        Ok((err_input, unit)) => {
            if err_input.is_empty() {
                Ok(unit)
            } else {
                Err(err_input)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn apply<'def, 'input, 'path>(
    input: &'input str,
    path: &'path str,
) -> Result<Pin<Box<JavaFile<'def>>>, Span<'def>> {
    let mut file = Pin::new(Box::new(JavaFile {
        unit: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
        content: input.to_owned(),
        path: path.to_owned(),
    }));
    let tokens = match tokenize::apply(unsafe { &*(file.content.as_ref() as *const str) }, &*file) {
        Ok(tokens) => tokens,
        Err(span) => return Err(span),
    };
    let mut id_gen = IdGen {
        uuid: 1,
        path: path.to_string(),
        runner: 0,
    };
    let unit = match apply_tokens(
        unsafe { &*(&tokens as *const Vec<Token<'def>>) },
        &mut id_gen,
    ) {
        Ok(unit) => unit,
        Err(tokens) => return Err(tokens.first().unwrap().span()),
    };

    file.unit = unit;

    Ok(file)
}
