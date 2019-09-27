use parse::combinator::{identifier, keyword, many0, opt, separated_nonempty_list, symbol};
use parse::tree::{Import, ImportPrefix};
use parse::{ParseResult, Tokens};
use std::cell::RefCell;
use tokenize::span::Span;
use tokenize::token::Token;

fn parse_wildcard(input: Tokens) -> ParseResult<Span> {
    let (input, _) = symbol('.')(input)?;
    let (input, wildcard) = symbol('*')(input)?;

    Ok((input, wildcard))
}

fn import(input: Tokens) -> ParseResult<Import> {
    let (input, _) = keyword("import")(input)?;

    let (input, static_opt) = opt(keyword("static"))(input)?;

    let (input, components) = separated_nonempty_list(symbol('.'), identifier)(input)?;
    let (input, wildcard_opt) = opt(parse_wildcard)(input)?;

    let (input, _) = symbol(';')(input)?;

    let mut prefix_opt: Option<ImportPrefix> = None;

    for component in &components[0..(components.len() - 1)] {
        prefix_opt = Some(ImportPrefix {
            prefix_opt: prefix_opt.map(Box::new),
            name: component.clone(),
            def_opt: RefCell::new(None),
        })
    }

    Ok((
        input,
        Import {
            prefix_opt: prefix_opt.map(Box::new),
            is_static: static_opt.is_some(),
            is_wildcard: wildcard_opt.is_some(),
            name: components.last().unwrap().clone(),
            def_opt: RefCell::new(None),
        },
    ))
}

pub fn parse(input: Tokens) -> ParseResult<Vec<Import>> {
    many0(import)(input)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Import, ImportPrefix};
    use std::cell::RefCell;
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
                        prefix_opt: Some(Box::new(ImportPrefix {
                            prefix_opt: None,
                            name: span(1, 8, "test"),
                            def_opt: RefCell::new(None)
                        })),
                        is_static: false,
                        is_wildcard: true,
                        name: span(1, 13, "a"),
                        def_opt: RefCell::new(None)
                    },
                    Import {
                        prefix_opt: Some(Box::new(ImportPrefix {
                            prefix_opt: None,
                            name: span(2, 15, "c"),
                            def_opt: RefCell::new(None)
                        })),
                        is_static: true,
                        is_wildcard: false,
                        name: span(2, 17, "b"),
                        def_opt: RefCell::new(None)
                    }
                ]
            ))
        )
    }
}
