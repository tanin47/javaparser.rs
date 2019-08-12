use parse::combinator::{identifier, many0, opt, separated_nonempty_list, symbol, word};
use parse::tree::Import;
use parse::{ParseResult, Tokens};
use tokenize::span::Span;
use tokenize::token::Token;

fn parse_wildcard(input: Tokens) -> ParseResult<Span> {
    let (input, _) = symbol(".")(input)?;
    let (input, wildcard) = symbol("*")(input)?;

    Ok((input, wildcard))
}

fn import(input: Tokens) -> ParseResult<Import> {
    let (input, _) = word("import")(input)?;

    let (input, static_opt) = opt(word("static"))(input)?;

    let (input, components) = separated_nonempty_list(symbol("."), identifier)(input)?;
    let (input, wildcard_opt) = opt(parse_wildcard)(input)?;

    let (input, _) = symbol(";")(input)?;

    Ok((
        input,
        Import {
            is_static: static_opt.is_some(),
            components,
            is_wildcard: wildcard_opt.is_some(),
        },
    ))
}

pub fn parse(input: Tokens) -> ParseResult<Vec<Import>> {
    many0(import)(input)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::Import;
    use test_common::{code, span};
    use tokenize;
    use tokenize::token::Token;

    #[test]
    fn test_wildcard() {
        assert_eq!(
            parse(&code(
                r#"
import test.a.*; 
import static c.b; 
            "#
            )),
            Ok((
                &[] as &[Token],
                vec![
                    Import {
                        is_static: false,
                        components: vec![span(1, 8, "test"), span(1, 13, "a")],
                        is_wildcard: true
                    },
                    Import {
                        is_static: true,
                        components: vec![span(2, 15, "c"), span(2, 17, "b")],
                        is_wildcard: false
                    }
                ]
            ))
        )
    }
}
