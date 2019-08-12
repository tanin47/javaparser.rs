use tokenize::token::Token;

pub mod combinator;
pub mod compilation_unit;
pub mod def;
pub mod expr;
pub mod tpe;
pub mod tree;

pub type Tokens<'a> = &'a [Token<'a>];

pub type ParseResult<'a, T> = Result<(Tokens<'a>, T), Tokens<'a>>;
