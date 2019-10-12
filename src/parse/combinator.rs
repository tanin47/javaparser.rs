use parse::{ParseResult, Tokens};
use std::slice;
use tokenize::span::CharAt;
use tokenize::span::Span;
use tokenize::token::Token;

pub fn any_symbol<'def: 'r, 'r>(
    s: &'def str,
) -> impl Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    move |input: Tokens<'def, 'r>| {
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

pub fn symbol<'def: 'r, 'r>(
    c: char,
) -> impl Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    move |input: Tokens<'def, 'r>| {
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

pub fn symbol2<'def: 'r, 'r>(
    a: char,
    b: char,
) -> impl Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    move |input: Tokens<'def, 'r>| {
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
                            file: first.file,
                        },
                    ));
                }
            }
        }

        Err(input)
    }
}

pub fn symbol3<'def: 'r, 'r>(
    a: char,
    b: char,
    c: char,
) -> impl Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    move |input: Tokens<'def, 'r>| {
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
                                file: first.file,
                            },
                        ));
                    }
                }
            }
        }

        Err(input)
    }
}

pub fn symbol4<'def: 'r, 'r>(
    a: char,
    b: char,
    c: char,
    d: char,
) -> impl Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    move |input: Tokens<'def, 'r>| {
        if input.len() < 4 {
            return Err(input);
        }

        if let Token::Symbol(first) = input[0] {
            if let Token::Symbol(second) = input[1] {
                if let Token::Symbol(third) = input[2] {
                    if let Token::Symbol(fourth) = input[3] {
                        if first.fragment.len() == 1
                            && second.fragment.len() == 1
                            && third.fragment.len() == 1
                            && fourth.fragment.len() == 1
                            && first.fragment.char_at(0) == a
                            && second.fragment.char_at(0) == b
                            && third.fragment.char_at(0) == c
                            && fourth.fragment.char_at(0) == d
                            && first.line == second.line
                            && first.line == third.line
                            && first.line == fourth.line
                            && first.col + 1 == second.col
                            && first.col + 2 == third.col
                            && first.col + 3 == fourth.col
                        {
                            return Ok((
                                &input[4..],
                                Span {
                                    line: first.line,
                                    col: first.col,
                                    fragment: unsafe {
                                        std::str::from_utf8_unchecked(slice::from_raw_parts(
                                            first.fragment.as_ptr(),
                                            first.fragment.len()
                                                + second.fragment.len()
                                                + third.fragment.len()
                                                + fourth.fragment.len(),
                                        ))
                                    },
                                    file: first.file,
                                },
                            ));
                        }
                    }
                }
            }
        }

        Err(input)
    }
}

pub fn keyword<'def: 'r, 'r>(
    s: &'def str,
) -> impl Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    move |input: Tokens<'def, 'r>| {
        if input.is_empty() {
            return Err(input);
        }

        if let Token::Keyword(span) = &input[0] {
            if span.fragment == s {
                return Ok((&input[1..], *span));
            }
        }

        Err(input)
    }
}

pub fn any_keyword<'def: 'r, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    if input.is_empty() {
        return Err(input);
    }

    if let Token::Keyword(span) = &input[0] {
        Ok((&input[1..], *span))
    } else {
        Err(input)
    }
}

pub fn identifier<'def: 'r, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    if input.is_empty() {
        return Err(input);
    }

    if let Token::Identifier(span) = &input[0] {
        Ok((&input[1..], *span))
    } else {
        Err(input)
    }
}

pub fn opt<'def: 'r, 'r, F, T>(
    mut f: F,
) -> impl FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Option<T>>
where
    F: FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, T>,
{
    move |input: Tokens<'def, 'r>| match f(input) {
        Ok((input, result)) => Ok((input, Some(result))),
        Err(_) => Ok((input, None)),
    }
}

pub fn get_and_not_followed_by<'def: 'r, 'r, I, F>(
    item: I,
    followed: F,
) -> impl Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>>
where
    I: Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>>,
    F: Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>>,
{
    move |original: Tokens<'def, 'r>| {
        let (input, result) = item(original)?;

        if input.len() > 0
            && result.line == input[0].span().line
            && result.col + result.fragment.len() == input[0].span().col
        {
            if let Ok((_, followed)) = followed(input) {
                return Err(original);
            }
        }

        Ok((input, result))
    }
}

pub fn separated_list<'def: 'r, 'r, S, I, W, T>(
    sep: S,
    mut item: I,
) -> impl FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Vec<T>>
where
    S: Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, W>,
    I: FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, T>,
{
    move |original: Tokens<'def, 'r>| {
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

pub fn separated_nonempty_list<'def: 'r, 'r, S, I, W, T>(
    sep: S,
    item: I,
) -> impl FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Vec<T>>
where
    S: Fn(Tokens<'def, 'r>) -> ParseResult<'def, 'r, W>,
    I: FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, T>,
{
    let mut f = separated_list(sep, item);
    move |original: Tokens<'def, 'r>| {
        let (input, items) = f(original)?;

        if items.is_empty() {
            Err(original)
        } else {
            Ok((input, items))
        }
    }
}

pub fn many0<'def: 'r, 'r, I, T>(
    mut item: I,
) -> impl FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Vec<T>>
where
    I: FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, T>,
{
    move |original: Tokens<'def, 'r>| {
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

pub fn many1<'def: 'r, 'r, I, T>(
    item: I,
) -> impl FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, Vec<T>>
where
    I: FnMut(Tokens<'def, 'r>) -> ParseResult<'def, 'r, T>,
{
    let mut f = many0(item);
    move |original: Tokens<'def, 'r>| {
        let (input, items) = f(original)?;

        if items.is_empty() {
            Err(original)
        } else {
            Ok((input, items))
        }
    }
}
