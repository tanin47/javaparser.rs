use std::io::Write;
use std::ops::Index;
use std::{io, slice};
use tokenize::combinator::{
    concat, take, take_bit, take_hex_number, take_number, take_one_if_case_insensitive, take_while,
};
use tokenize::span::CharAt;
use tokenize::span::Span;
use tokenize::token::Token;
use JavaFile;

pub mod combinator;
pub mod span;
pub mod token;

pub fn apply<'def>(
    content: &'def str,
    file: *const JavaFile<'def>,
) -> Result<Vec<Token<'def>>, Span<'def>> {
    let mut input = Span {
        line: 1,
        col: 1,
        fragment: content,
        file,
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

fn tokenize<'def>(input: Span<'def>) -> Result<(Span<'def>, Option<Token<'def>>), Span<'def>> {
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
    } else if let Ok((input, token)) = bit(input) {
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
    c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == 12 as char // form feed
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

fn hex_p<'a>(num: Span<'a>, original: Span<'a>) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (must_be_p, input) = take_one_if_case_insensitive("P", original);

    if must_be_p.fragment.is_empty() {
        return Err(original);
    }

    let (maybe_sign, input) = take_one_if_case_insensitive("-+", input);
    let (exponent, input) = take_number(input);

    if exponent.fragment.is_empty() {
        return Err(input);
    }

    let (ending, input) = take_one_if_case_insensitive("FD", input);

    let num = concat(&[num, must_be_p, maybe_sign, exponent, ending]);

    if ending.fragment.char_at(0).to_ascii_uppercase() == 'F' {
        Ok((input, Token::Float(num)))
    } else {
        Ok((input, Token::Double(num)))
    }
}

fn hex_decimal<'a>(num: Span<'a>, original: Span<'a>) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (maybe_p_or_dot, input) = take_one_if_case_insensitive("P.", original);

    if maybe_p_or_dot.fragment.is_empty() {
        return Ok((input, Token::Int(num)));
    }

    if maybe_p_or_dot.fragment.char_at(0) != '.' {
        return hex_p(num, original);
    }

    let must_be_dot = maybe_p_or_dot;
    assert_eq!(must_be_dot.fragment, ".");

    let (decimal, input) = take_hex_number(input);

    if num.fragment.is_empty() && decimal.fragment.is_empty() {
        return Err(original);
    }

    let num = concat(&[num, must_be_dot, decimal]);

    hex_p(num, input)
}

fn hex(original: Span) -> Result<(Span, Token), Span> {
    if original.fragment.len() >= 2
        && original.fragment.char_at(0) == '0'
        && original.fragment.char_at(1).to_ascii_uppercase() == 'X'
    {
    } else {
        return Err(original);
    }

    let (prefix, input) = take(2, original);
    let (hex, input) = take_hex_number(input);

    let (maybe_l, input) = take_one_if_case_insensitive("L", input);
    let num = concat(&[prefix, hex, maybe_l]);

    if maybe_l.fragment.is_empty() {
        hex_decimal(num, input)
    } else {
        if hex.fragment.is_empty() {
            return Err(original);
        }

        Ok((input, Token::Long(num)))
    }
}

fn bit(original: Span) -> Result<(Span, Token), Span> {
    if original.fragment.len() >= 2
        && original.fragment.char_at(0) == '0'
        && original.fragment.char_at(1).to_ascii_uppercase() == 'B'
    {
    } else {
        return Err(original);
    }

    let (prefix, input) = take(2, original);
    let (bits, input) = take_bit(input);

    let (maybe_l, input) = take_one_if_case_insensitive("L", input);
    let num = concat(&[prefix, bits, maybe_l]);

    if maybe_l.fragment.is_empty() {
        Ok((input, Token::Int(num)))
    } else {
        Ok((input, Token::Long(num)))
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
                file: number.file,
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
                file: number.file,
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
    let (maybe_e, input) = take_one_if_case_insensitive("E", original);

    if maybe_e.fragment.is_empty() {
        return float_or_double_end(num, original, include_dot);
    }

    if num.fragment.is_empty() {
        return Err(input);
    }

    let (maybe_operator, input) = take_one_if_case_insensitive("+-", input);
    let (second_number, input) = take_number(input);

    let num = concat(&[num, maybe_e, maybe_operator, second_number]);

    float_or_double_end(num, input, true)
}

fn float_or_double_dot<'a>(
    first_number: Span<'a>,
    original: Span<'a>,
) -> Result<(Span<'a>, Token<'a>), Span<'a>> {
    let (symbol, input) = take(1, original);
    let last_char = if !symbol.fragment.is_empty() {
        symbol.fragment.char_at(0).to_ascii_uppercase()
    } else {
        ' ' // anything that is not a dot.
    };

    if last_char != '.' {
        return float_or_double_e(first_number, original, false);
    }

    let (second_number, input) = take_number(input);

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
        file: first_number.file,
    };

    float_or_double_e(num, input, true)
}

fn int_or_long_or_double_or_float(original: Span) -> Result<(Span, Token), Span> {
    let (number, input) = take_number(original);

    if let Ok(ok) = float_or_double_dot(number, input) {
        return Ok(ok);
    }

    if number.fragment.is_empty() {
        return Err(original);
    }

    let (maybe_l, input) = take_one_if_case_insensitive("L", input);

    let num = concat(&[number, maybe_l]);

    if maybe_l.fragment.is_empty() {
        Ok((input, Token::Int(num)))
    } else {
        Ok((input, Token::Long(num)))
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
    use test_common::{generate_tokens, span};
    use tokenize::span::Span;
    use tokenize::token::Token;
    use JavaFile;

    fn apply(content: &str) -> Result<Vec<Token>, Span> {
        super::apply(content, std::ptr::null())
    }

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
    fn test_bit() {
        assert_eq!(
            apply(
                r#"
0b001 0b1L
"#
                .trim()
            ),
            Ok(vec![
                Token::Int(span(1, 1, "0b001")),
                Token::Long(span(1, 7, "0b1L")),
            ])
        )
    }

    #[test]
    fn test_hex() {
        assert_eq!(
            apply(
                r#"
0x0123456789abcdefABCDEF 0x02L 0x2p+3 0x2p-3d 0xap2f 0x1.0p2d 0x.1p2 0x.ap2f
"#
                .trim()
            ),
            Ok(vec![
                Token::Int(span(1, 1, "0x0123456789abcdefABCDEF")),
                Token::Long(span(1, 26, "0x02L")),
                Token::Double(span(1, 32, "0x2p+3")),
                Token::Double(span(1, 39, "0x2p-3d")),
                Token::Float(span(1, 47, "0xap2f")),
                Token::Double(span(1, 54, "0x1.0p2d")),
                Token::Double(span(1, 63, "0x.1p2")),
                Token::Float(span(1, 70, "0x.ap2f")),
            ])
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
1.0 1. .1 1.0d 1.D .1D 2d 1e-2 1e+3d 5.3e4d e2
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
                Token::Double(span(1, 32, "1e+3d")),
                Token::Double(span(1, 38, "5.3e4d")),
                Token::Identifier(span(1, 45, "e2")),
            ])
        )
    }

    #[test]
    fn test_float() {
        assert_eq!(
            apply(
                r#"
1.0f 1f 1.F .1f 1e2F 1.3e-2F 12e+1f
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
                Token::Float(span(1, 30, "12e+1f")),
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
