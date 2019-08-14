use std::io::Write;
use std::ops::Index;
use std::{io, slice};
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
    } else if let Ok((input, token)) = int_or_long_or_double_or_float(input) {
        Ok((input, Some(token)))
    } else if let Ok((input, token)) = keyword_or_identifier(input) {
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

    let mut escaped = false;
    let mut size = 1;

    for index in 1..input.fragment.len() {
        let c = input.fragment.char_at(index);

        size += 1;

        if escaped {
            escaped = false;
            continue;
        }

        if c == '"' {
            break;
        }

        if c == '\\' {
            escaped = true;
        }
    }

    let (string, after) = take(size, input);

    Ok((after, Token::String(string)))
}

fn literal_char(input: Span) -> Result<(Span, Token), Span> {
    if input.fragment.char_at(0) == '\'' {
    } else {
        return Err(input);
    }

    let (literal_char, after) = take_while(
        |index, s| {
            let end_cond =
                index >= 2 && s.char_at(index - 2) != '\\' && s.char_at(index - 1) == '\'';
            let end_cond_2 = index >= 3
                && s.char_at(index - 3) == '\\'
                && s.char_at(index - 2) == '\\'
                && s.char_at(index - 1) == '\'';

            !(end_cond || end_cond_2)
        },
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

    let (maybe_l, input_after_l) = take(1, input);

    if maybe_l.fragment.char_at(0) == 'L' {
        Ok((
            input_after_l,
            Token::Hex(Span {
                line: prefix.line,
                col: prefix.col,
                fragment: &original.fragment[0..(prefix.fragment.len() + hex.fragment.len() + 1)],
            }),
        ))
    } else {
        Ok((
            input,
            Token::Hex(Span {
                line: prefix.line,
                col: prefix.col,
                fragment: &original.fragment[0..(prefix.fragment.len() + hex.fragment.len())],
            }),
        ))
    }
}

fn is_identifier(index: usize, s: &str) -> bool {
    let c = s.char_at(index);
    c >= '0' && c <= '9' || c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_' || c == '$'
}

fn is_keyword(s: &str) -> bool {
    match s {
        "abstract" | "assert" | "boolean" | "break" | "byte" | "case" | "catch" | "char"
        | "class" | "const" | "continue" | "default" | "do" | "double" | "else" | "enum"
        | "extends" | "final" | "finally" | "float" | "for" | "goto" | "if" | "implements"
        | "import" | "instanceof" | "int" | "interface" | "long" | "native" | "new" | "package"
        | "private" | "protected" | "public" | "return" | "short" | "static" | "strictfp"
        | "super" | "switch" | "synchronized" | "this" | "throw" | "throws" | "transient"
        | "try" | "void" | "volatile" | "while" | "true" | "false" | "null" => true,
        _ => false,
    }
}

fn keyword_or_identifier(original: Span) -> Result<(Span, Token), Span> {
    let (ident, input) = take_while(is_identifier, original);

    if ident.fragment.is_empty() {
        return Err(original);
    }

    if is_keyword(ident.fragment) {
        Ok((input, Token::Keyword(ident)))
    } else {
        Ok((input, Token::Identifier(ident)))
    }
}

fn symbol(input: Span) -> Result<(Span, Token), Span> {
    // No need to check anything before word(..) and is_whitespace(..) guarantees that this is a symbol.

    let (symbol, input) = take(1, input);

    Ok((input, Token::Symbol(symbol)))
}

fn number(original: Span) -> Result<(Span, Span), Span> {
    let (number, input) = take_while(
        |index, s| {
            let c = s.char_at(index);
            c >= '0' && c <= '9'
        },
        original,
    );

    Ok((number, input))
}

fn float_or_double_end<'a>(
    number: Span<'a>,
    original: Span<'a>,
    include_dot_or_e: bool,
) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    if number.fragment.is_empty() {
        return Err(original);
    }

    let (symbol, input) = take(1, original);

    let last_char = symbol.fragment.char_at(0).to_ascii_uppercase();
    if last_char == 'D' {
        Ok((
            input,
            Token::Double(Span {
                line: number.line,
                col: number.col,
                fragment: unsafe {
                    std::str::from_utf8_unchecked(slice::from_raw_parts(
                        number.fragment.as_ptr(),
                        number.fragment.len() + symbol.fragment.len(),
                    ))
                },
            }),
        ))
    } else if last_char == 'F' {
        Ok((
            input,
            Token::Float(Span {
                line: number.line,
                col: number.col,
                fragment: unsafe {
                    std::str::from_utf8_unchecked(slice::from_raw_parts(
                        number.fragment.as_ptr(),
                        number.fragment.len() + symbol.fragment.len(),
                    ))
                },
            }),
        ))
    } else if include_dot_or_e {
        Ok((original, Token::Double(number)))
    } else {
        Err(original)
    }
}

fn float_or_double_e<'a>(
    num: Span<'a>,
    original: Span<'a>,
    include_dot: bool,
) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (symbol, input) = take(1, original);
    let last_char = symbol.fragment.char_at(0).to_ascii_uppercase();

    if last_char != 'E' {
        return float_or_double_end(num, original, include_dot);
    }

    if num.fragment.is_empty() {
        return Err(input);
    }

    let (maybe_minus, input) = if input.fragment.char_at(0) == '-' {
        take(1, input)
    } else {
        (
            Span {
                line: input.line,
                col: input.col,
                fragment: "",
            },
            input,
        )
    };
    let (second_number, input) = number(input)?;
    let num = Span {
        line: num.line,
        col: num.col,
        fragment: unsafe {
            std::str::from_utf8_unchecked(slice::from_raw_parts(
                num.fragment.as_ptr(),
                num.fragment.len()
                    + symbol.fragment.len()
                    + maybe_minus.fragment.len()
                    + second_number.fragment.len(),
            ))
        },
    };

    float_or_double_end(num, input, true)
}

fn float_or_double_dot<'a>(
    first_number: Span<'a>,
    original: Span<'a>,
) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (symbol, input) = take(1, original);
    let last_char = symbol.fragment.char_at(0).to_ascii_uppercase();

    if last_char != '.' {
        return float_or_double_e(first_number, original, false);
    }

    let (second_number, input) = number(input)?;

    if first_number.fragment.is_empty() && second_number.fragment.is_empty() {
        return Err(original);
    }

    let num = Span {
        line: first_number.line,
        col: first_number.col,
        fragment: unsafe {
            std::str::from_utf8_unchecked(slice::from_raw_parts(
                first_number.fragment.as_ptr(),
                first_number.fragment.len() + symbol.fragment.len() + second_number.fragment.len(),
            ))
        },
    };

    float_or_double_e(num, input, true)
}

fn int_or_long_or_double_or_float(original: Span) -> Result<(Span, Token), Span> {
    let (number, input) = number(original)?;

    if let Ok(ok) = float_or_double_dot(number, input) {
        return Ok(ok);
    }

    if number.fragment.is_empty() {
        return Err(original);
    }

    let last_char = input.fragment.char_at(0).to_ascii_uppercase();
    if last_char == 'L' {
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
    fn test_unicode() {
        assert_eq!(
            apply(
                r#"
"打包选项"
"#
                .trim()
            ),
            Ok(vec![Token::String(span(1, 1, "\"打包选项\""))])
        )
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(
            apply(
                r#"
"" +
"#
                .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\"")),
                Token::Symbol(span(1, 4, "+"))
            ])
        )
    }

    #[test]
    fn test_string() {
        assert_eq!(
            apply(
                r#"
"ab\"c" +
"#
                .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"ab\\\"c\"")),
                Token::Symbol(span(1, 9, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_string() {
        assert_eq!(
            apply(
                r#"
"\"" +
"#
                .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\\\"\"")),
                Token::Symbol(span(1, 6, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_string_2() {
        assert_eq!(
            apply(
                r#"
"\\" +
"#
                .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\\\\\"")),
                Token::Symbol(span(1, 6, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_string_3() {
        assert_eq!(
            apply(
                r#"
"\\\"" +
"#
                .trim()
            ),
            Ok(vec![
                Token::String(span(1, 1, "\"\\\\\\\"\"")),
                Token::Symbol(span(1, 8, "+"))
            ])
        )
    }

    #[test]
    fn test_empty_char() {
        assert_eq!(
            apply(
                r#"
'' +
"#
                .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "''")),
                Token::Symbol(span(1, 4, "+"))
            ])
        )
    }

    #[test]
    fn test_char() {
        assert_eq!(
            apply(
                r#"
'a' +
"#
                .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "'a'")),
                Token::Symbol(span(1, 5, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_char() {
        assert_eq!(
            apply(
                r#"
'\'' +
"#
                .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "'\\''")),
                Token::Symbol(span(1, 6, "+"))
            ])
        )
    }

    #[test]
    fn test_escaped_backslash_char() {
        assert_eq!(
            apply(
                r#"
'\\' +
"#
                .trim()
            ),
            Ok(vec![
                Token::Char(span(1, 1, "'\\\\'")),
                Token::Symbol(span(1, 6, "+"))
            ])
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
    fn test_hex_with_l() {
        assert_eq!(
            apply(
                r#"
0x0123456789abcdefABCDEFL
"#
                .trim()
            ),
            Ok(vec![Token::Hex(span(1, 1, "0x0123456789abcdefABCDEFL"))])
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
    fn test_double() {
        assert_eq!(
            apply(
                r#"
1.0 1. .1 1.0d 1.D .1D 2d 1e-2 1e3d 5.3e4d e2
"#
                .trim()
            ),
            Ok(vec![
                Token::Double(span(1, 1, "1.0")),
                Token::Double(span(1, 5, "1.")),
                Token::Double(span(1, 8, ".1")),
                Token::Double(span(1, 11, "1.0d")),
                Token::Double(span(1, 16, "1.D")),
                Token::Double(span(1, 20, ".1D")),
                Token::Double(span(1, 24, "2d")),
                Token::Double(span(1, 27, "1e-2")),
                Token::Double(span(1, 32, "1e3d")),
                Token::Double(span(1, 37, "5.3e4d")),
                Token::Identifier(span(1, 44, "e2")),
            ])
        )
    }

    #[test]
    fn test_float() {
        assert_eq!(
            apply(
                r#"
1.0f 1f 1.F .1f 1e2F 1.3e-2F 
"#
                .trim()
            ),
            Ok(vec![
                Token::Float(span(1, 1, "1.0f")),
                Token::Float(span(1, 6, "1f")),
                Token::Float(span(1, 9, "1.F")),
                Token::Float(span(1, 13, ".1f")),
                Token::Float(span(1, 17, "1e2F")),
                Token::Float(span(1, 22, "1.3e-2F")),
            ])
        )
    }

    #[test]
    fn test_word() {
        assert_eq!(
            apply(
                r#"
a_b$1B
"#
                .trim()
            ),
            Ok(vec![Token::Identifier(span(1, 1, "a_b$1B"))])
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
                Token::Keyword(span(1, 1, "void")),
                Token::Identifier(span(1, 6, "method")),
                Token::Symbol(span(1, 12, "(")),
                Token::Symbol(span(1, 13, ")")),
                Token::Symbol(span(1, 15, "{")),
                Token::Identifier(span(3, 5, "a")),
                Token::Symbol(span(3, 6, "+")),
                Token::Symbol(span(3, 7, "+")),
                Token::Symbol(span(3, 8, ";")),
                Token::Symbol(span(4, 1, "}")),
            ])
        )
    }
}
