use parse::tree::CompilationUnit;
use std::borrow::Borrow;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::null;
use tokenize;
use tokenize::token::Token;

pub mod combinator;
pub mod compilation_unit;
pub mod def;
pub mod expr;
pub mod statement;
pub mod tpe;
pub mod tree;

pub type Tokens<'a> = &'a [Token<'a>];

pub type ParseResult<'a, T> = Result<(Tokens<'a>, T), Tokens<'a>>;

pub struct JavaFile<'def> {
    pub unit: CompilationUnit<'def>,
    pub content: String,
    pub path: String,
}
unsafe impl<'a> Sync for JavaFile<'a> {}

pub fn apply_tokens<'def>(input: Tokens<'def>) -> Result<CompilationUnit<'def>, Tokens<'def>> {
    let result = compilation_unit::parse(input);

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
) -> Result<Pin<Box<JavaFile<'def>>>, &'def str> {
    let mut file = Pin::new(Box::new(JavaFile {
        unit: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
        content: input.to_owned(),
        path: path.to_owned(),
    }));
    let tokens = match tokenize::apply(unsafe { &*(file.content.as_ref() as *const str) }, &*file) {
        Ok(tokens) => tokens,
        Err(span) => return Err(span.fragment),
    };
    let unit = match apply_tokens(unsafe { &*(&tokens as *const Vec<Token<'def>>) }) {
        Ok(unit) => unit,
        Err(tokens) => return Err(tokens.first().unwrap().span().fragment),
    };

    file.unit = unit;

    Ok(file)
}
