use std::ops::Index;
use tokenize::combinator::{take, take_while};
use tokenize::span::CharAt;
use tokenize::span::Span;
use tokenize::token::Token;

pub mod combinator;
pub mod span;
pub mod token;

pub fn apply(content: &str) -> Result<Vec<Token>, Span> {
    let mut input = Span {
        line: 1,
        col: 1,
        fragment: content,
    };
    let mut tokens = vec![];

    while input.fragment.len() > 0 {
        let (next_input, token_opt) = tokenize(input)?;
        input = next_input;

        match token_opt {
            Some(Token::Comment(_)) => (),
            Some(token) => tokens.push(token),
            None => (),
        };
    }

    Ok(tokens)
}

fn tokenize(input: Span) -> Result<(Span, Option<Token>), Span> {
    let input = skip_space(input);

    if input.fragment.is_empty() {
        return Ok((input, None));
    }

    if let Ok((input, token)) = oneline_comment(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = multiline_comment(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = string(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = literal_char(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = hex(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = int_or_long(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = word(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = symbol(input) {
        Ok((input, Some(token)))
    } else {
        Err(input)
    }
}

fn is_whitespace(index: usize, s: &str) -> bool {
    let c = s.char_at(index);
    c == ' ' || c == '\t' || c == '\r' || c == '\n'
}

fn skip_space(input: Span) -> Span {
    let (_, input) = take_while(is_whitespace, input);
    input
}

fn string(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.char_at(0) == '"' {
    } else {
        return Err(input);
    }

    let (s, after) = take_while(
        |index, s| !(index >= 2 && s.char_at(index - 2) != '\\' && s.char_at(index - 1) == '"'),
        input,
    );

    Ok((after, Token::String(s)))
}

fn literal_char(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.char_at(0) == '\'' {
    } else {
        return Err(input);
    }

    let (literal_char, after) = take_while(
        |index, s| !(index >= 2 && s.char_at(index - 2) != '\\' && s.char_at(index - 1) == '\''),
        input,
    );

    Ok((after, Token::Char(literal_char)))
}

fn hex(original: Span) -> Result<(Span, Token), Span> {
    if original.fragment.len() >= 2
        && original.fragment.char_at(0) == '0'
        && original.fragment.char_at(1) == 'x'
    {
    } else {
        return Err(original);
    }

    let (prefix, input) = take(2, original);

    let (hex, input) = take_while(
        |index, s| {
            let c = s.char_at(index);
            c >= '0' && c <= '9' || c >= 'a' && c <= 'f' || c >= 'A' && c <= 'F'
        },
        input,
    );

    Ok((
        input,
        Token::Hex(Span {
            line: prefix.line,
            col: prefix.col,
            fragment: &original.fragment[0..(prefix.fragment.len() + hex.fragment.len())],
        }),
    ))
}

fn is_identifier(index: usize, s: &str) -> bool {
    let c = s.char_at(index);
    c >= '0' && c <= '9' || c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
}

fn word(original: Span) -> Result<(Span, Token), Span> {
    let (word, input) = take_while(is_identifier, original);

    if word.fragment.is_empty() {
        return Err(original);
    }

    Ok((input, Token::Word(word)))
}

fn symbol(input: Span) -> Result<(Span, Token), Span> {
    // No need to check anything before word(..) and is_whitespace(..) guarantees that this is a symbol.

    let (symbol, input) = take(1, input);

    Ok((input, Token::Symbol(symbol)))
}

fn int_or_long(original: Span) -> Result<(Span, Token), Span> {
    let (number, input) = take_while(
        |index, s| {
            let c = s.char_at(index);
            c >= '0' && c <= '9'
        },
        original,
    );

    if number.fragment.is_empty() {
        return Err(original);
    }

    let last_char = input.fragment.char_at(0);
    if last_char == 'l' || last_char == 'L' {
        let (_, input) = take(1, input);
        Ok((
            input,
            Token::Long(Span {
                line: number.line,
                col: number.col,
                fragment: &original.fragment[0..(number.fragment.len() + 1)],
            }),
        ))
    } else {
        Ok((input, Token::Int(number)))
    }
}

fn oneline_comment(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.len() >= 2
        && input.fragment.char_at(0) == '/'
        && input.fragment.char_at(1) == '/'
    {
    } else {
        return Err(input);
    }

    let (comment, after) = take_while(|index, s| s.char_at(index) != '\n', input);

    Ok((after, Token::Comment(comment)))
}

fn multiline_comment(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.len() >= 2
        && input.fragment.char_at(0) == '/'
        && input.fragment.char_at(1) == '*'
    {
    } else {
        return Err(input);
    }

    let (comment, after) = take_while(
        |index, s| !(index >= 2 && s.char_at(index - 2) == '*' && s.char_at(index - 1) == '/'),
        input,
    );

    Ok((after, Token::Comment(comment)))
}

#[cfg(test)]
mod tests {
    use super::apply;
    use test_common::span;
    use tokenize::token::Token;
    use tokenize::Span;

    #[test]
    fn test_oneline_comment() {
        assert_eq!(apply("// test"), Ok(vec![]))
    }

    #[test]
    fn test_multiline_comment() {
        assert_eq!(
            apply(
                r#"
/* test
 *
 */
"#
                .trim()
            ),
            Ok(vec![])
        )
    }

    #[test]
    fn test_string() {
        assert_eq!(
            apply(
                r#"
"ab\"c"
"#
                .trim()
            ),
            Ok(vec![Token::String(span(1, 1, "\"ab\\\"c\""))])
        )
    }

    #[test]
    fn test_char() {
        assert_eq!(
            apply(
                r#"
'a'
"#
                .trim()
            ),
            Ok(vec![Token::Char(span(1, 1, "'a'"))])
        )
    }

    #[test]
    fn test_escaped_char() {
        assert_eq!(
            apply(
                r#"
'\''
"#
                .trim()
            ),
            Ok(vec![Token::Char(span(1, 1, "'\\''"))])
        )
    }

    #[test]
    fn test_hex() {
        assert_eq!(
            apply(
                r#"
0x0123456789abcdefABCDEF
"#
                .trim()
            ),
            Ok(vec![Token::Hex(span(1, 1, "0x0123456789abcdefABCDEF"))])
        )
    }

    #[test]
    fn test_int() {
        assert_eq!(
            apply(
                r#"
1234567890
"#
                .trim()
            ),
            Ok(vec![Token::Int(span(1, 1, "1234567890"))])
        )
    }

    #[test]
    fn test_long() {
        assert_eq!(
            apply(
                r#"
1234567890L
"#
                .trim()
            ),
            Ok(vec![Token::Long(span(1, 1, "1234567890L"))])
        )
    }

    #[test]
    fn test_word() {
        assert_eq!(
            apply(
                r#"
a_b1B
"#
                .trim()
            ),
            Ok(vec![Token::Word(span(1, 1, "a_b1B"))])
        )
    }

    #[test]
    fn test_symbol() {
        assert_eq!(
            apply(
                r#"
>>
"#
                .trim()
            ),
            Ok(vec![
                Token::Symbol(span(1, 1, ">")),
                Token::Symbol(span(1, 2, ">"))
            ])
        )
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            apply(
                r#"
void method() {
    // test
    a++;
}
"#
                .trim()
            ),
            Ok(vec![
                Token::Word(span(1, 1, "void")),
                Token::Word(span(1, 6, "method")),
                Token::Symbol(span(1, 12, "(")),
                Token::Symbol(span(1, 13, ")")),
                Token::Symbol(span(1, 15, "{")),
                Token::Word(span(3, 5, "a")),
                Token::Symbol(span(3, 6, "+")),
                Token::Symbol(span(3, 7, "+")),
                Token::Symbol(span(3, 8, ";")),
                Token::Symbol(span(4, 1, "}")),
            ])
        )
    }
}
