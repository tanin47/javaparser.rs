use parse::{ParseResult, Tokens};
use tokenize::span::Span;
use tokenize::token::Token;

pub fn skip_comment(input: Tokens) -> Tokens {
    if input.is_empty() {
        return input;
    }

    match &input[0] {
        Token::Comment(_) => (),
        _ => return input,
    };

    let mut count = 0;

    for i in input {
        if let Token::Comment(comment) = i {
            count += 1;
        } else {
            break;
        }
    }

    input.split_at(count).1
}

pub fn symbol<'a>(s: &'a str) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>> {
    move |input: Tokens<'a>| {
        let input = skip_comment(input);
        if input.is_empty() {
            return Err(input);
        }

        if let Token::Symbol(span) = &input[0] {
            if span.fragment == s {
                return Ok((&input[1..], *span));
            }
        }
        Err(input)
    }
}

pub fn word<'a>(s: &'a str) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Span<'a>> {
    move |input: Tokens<'a>| {
        let input = skip_comment(input);
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
    let input = skip_comment(input);
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

pub fn is_not<'a, F, T>(f: F) -> impl Fn(Tokens<'a>) -> ParseResult<'a, Token>
where
    F: Fn(Tokens<'a>) -> ParseResult<'a, T>,
{
    move |input: Tokens<'a>| match f(input) {
        Ok((_, result)) => Err(input),
        Err(_) => Ok((input.split_at(1).1, input[0])),
    }
}

pub fn get_and_followed_by<'a, I, T, F, S>(
    item: I,
    followed: F,
) -> impl Fn(Tokens<'a>) -> ParseResult<'a, T>
where
    I: Fn(Tokens<'a>) -> ParseResult<'a, T>,
    F: Fn(Tokens<'a>) -> ParseResult<'a, S>,
{
    move |original: Tokens<'a>| {
        let (input, result) = item(original)?;

        if let Ok(_) = followed(input) {
            Ok((input, result))
        } else {
            Err(original)
        }
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
