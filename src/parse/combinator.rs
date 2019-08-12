use parse::{ParseResult, Tokens};
use std::slice;
use tokenize::span::CharAt;
use tokenize::span::Span;
use tokenize::token::Token;

pub fn any_symbol<'a>(s: &'a str) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>> {
    move |input: Tokens<'a>| {
        if input.is_empty() {
            return Err(input);
        }

        if let Token::Symbol(span) = &input[0] {
            if span.fragment.len() == 1 && s.contains(span.fragment) {
                return Ok((&input[1..], *span));
            }
        }
        Err(input)
    }
}

pub fn symbol<'a>(c: char) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>> {
    move |input: Tokens<'a>| {
        if input.is_empty() {
            return Err(input);
        }

        if let Token::Symbol(span) = input[0] {
            if span.fragment.len() == 1 && span.fragment.char_at(0) == c {
                return Ok((&input[1..], span));
            }
        }
        Err(input)
    }
}

pub fn symbol2<'a>(a: char, b: char) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>> {
    move |input: Tokens<'a>| {
        if input.len() < 2 {
            return Err(input);
        }

        if let Token::Symbol(first) = &input[0] {
            if let Token::Symbol(second) = &input[1] {
                if first.fragment.len() == 1
                    && second.fragment.len() == 1
                    && first.fragment.char_at(0) == a
                    && second.fragment.char_at(0) == b
                    && first.line == second.line
                    && first.col + 1 == second.col
                {
                    return Ok((
                        &input[2..],
                        Span {
                            line: first.line,
                            col: first.col,
                            fragment: unsafe {
                                std::str::from_utf8_unchecked(slice::from_raw_parts(
                                    first.fragment.as_ptr(),
                                    first.fragment.len() + second.fragment.len(),
                                ))
                            },
                        },
                    ));
                }
            }
        }

        Err(input)
    }
}

pub fn symbol3<'a>(a: char, b: char, c: char) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>> {
    move |input: Tokens<'a>| {
        if input.len() < 3 {
            return Err(input);
        }

        if let Token::Symbol(first) = input[0] {
            if let Token::Symbol(second) = input[1] {
                if let Token::Symbol(third) = input[2] {
                    if first.fragment.len() == 1
                        && second.fragment.len() == 1
                        && third.fragment.len() == 1
                        && first.fragment.char_at(0) == a
                        && second.fragment.char_at(0) == b
                        && third.fragment.char_at(0) == c
                        && first.line == second.line
                        && first.line == third.line
                        && first.col + 1 == second.col
                        && first.col + 2 == third.col
                    {
                        return Ok((
                            &input[3..],
                            Span {
                                line: first.line,
                                col: first.col,
                                fragment: unsafe {
                                    std::str::from_utf8_unchecked(slice::from_raw_parts(
                                        first.fragment.as_ptr(),
                                        first.fragment.len()
                                            + second.fragment.len()
                                            + third.fragment.len(),
                                    ))
                                },
                            },
                        ));
                    }
                }
            }
        }

        Err(input)
    }
}

pub fn word<'a>(s: &'a str) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>> {
    move |input: Tokens<'a>| {
        if input.is_empty() {
            return Err(input);
        }

        if let Token::Word(span) = &input[0] {
            if span.fragment == s {
                return Ok((&input[1..], *span));;
            }
        }

        Err(input)
    }
}

pub fn identifier(input: Tokens) -> ParseResult<Span> {
    if input.is_empty() {
        return Err(input);
    }

    if let Token::Word(span) = &input[0] {
        Ok((&input[1..], *span))
    } else {
        Err(input)
    }
}

pub fn opt<'a, F, T>(f: F) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Option<T>>
where
    F: Fn(Tokens<'a>) -> ParseResult<'a, T>,
{
    move |input: Tokens<'a>| match f(input) {
        Ok((input, result)) => Ok((input, Some(result))),
        Err(_) => Ok((input, None)),
    }
}

pub fn is_not<'a, F, T>(f: F) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span>
where
    F: Fn(Tokens<'a>) -> ParseResult<'a, T>,
{
    move |input: Tokens<'a>| match f(input) {
        Ok((_, result)) => Err(input),
        Err(_) => Ok((input.split_at(1).1, input[0].span())),
    }
}

pub fn get_and_followed_by<'a, I, F>(
    item: I,
    followed: F,
) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>>
where
    I: Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>>,
    F: Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>>,
{
    move |original: Tokens<'a>| {
        let (input, result) = item(original)?;

        if let Ok((_, followed)) = followed(input) {
            return Ok((input, result));
        }

        Err(original)
    }
}

pub fn separated_list<'a, S, I, W, T>(sep: S, item: I) -> impl Fn(Tokens<'a>) -> ParseResult<Vec<T>>
where
    S: Fn(Tokens<'a>) -> ParseResult<'a, W>,
    I: Fn(Tokens<'a>) -> ParseResult<'a, T>,
{
    move |original: Tokens<'a>| {
        let mut input = original;
        let mut before_sep = original;
        let mut items = vec![];
        loop {
            before_sep = match item(input) {
                Ok((input, i)) => {
                    items.push(i);
                    input
                }
                Err(_) => break,
            };

            input = match sep(before_sep) {
                Ok((input, _)) => input,
                Err(_) => break,
            }
        }

        input = before_sep;
        Ok((input, items))
    }
}

pub fn separated_nonempty_list<'a, S, I, W, T>(
    sep: S,
    item: I,
) -> impl Fn(Tokens<'a>) -> ParseResult<Vec<T>>
where
    S: Fn(Tokens<'a>) -> ParseResult<'a, W>,
    I: Fn(Tokens<'a>) -> ParseResult<'a, T>,
{
    let f = separated_list(sep, item);
    move |original: Tokens<'a>| {
        let (input, items) = f(original)?;

        if items.is_empty() {
            Err(original)
        } else {
            Ok((input, items))
        }
    }
}

pub fn many0<'a, I, T>(item: I) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Vec<T>>
where
    I: Fn(Tokens<'a>) -> ParseResult<'a, T>,
{
    move |original: Tokens<'a>| {
        let mut input = original;
        let mut items = vec![];
        loop {
            input = match item(input) {
                Ok((input, i)) => {
                    items.push(i);
                    input
                }
                Err(_) => break,
            };
        }

        Ok((input, items))
    }
}

pub fn many1<'a, I, T>(item: I) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Vec<T>>
where
    I: Fn(Tokens<'a>) -> ParseResult<'a, T>,
{
    let f = many0(item);
    move |original: Tokens<'a>| {
        let (input, items) = f(original)?;

        if items.is_empty() {
            Err(original)
        } else {
            Ok((input, items))
        }
    }
}
