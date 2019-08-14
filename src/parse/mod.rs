use parse::tree::CompilationUnit;
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

pub fn apply(input: Tokens) -> Result<CompilationUnit, Tokens> {
    let result = compilation_unit::parse(input);

    match result {
        Ok((input, unit)) => {
            if input.is_empty() {
                Ok(unit)
            } else {
                Err(input)
            }
        }
        Err(e) => Err(e),
    }
}
