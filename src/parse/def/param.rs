use parse::combinator::{identifier, opt, symbol};
use parse::def::modifiers;
use parse::tree::Param;
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_varargs(input: Tokens) -> ParseResult<()> {
    let (input, _) = symbol(".")(input)?;
    let (input, _) = symbol(".")(input)?;
    let (input, _) = symbol(".")(input)?;
    Ok((input, ()))
}

pub fn parse(input: Tokens) -> ParseResult<Param> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, tpe) = tpe::parse(input)?;
    let (input, varargs_opt) = opt(parse_varargs)(input)?;
    let (input, name) = identifier(input)?;

    Ok((
        input,
        Param {
            modifiers,
            tpe,
            is_varargs: varargs_opt.is_some(),
            name,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Annotated, ArrayType, ClassType, Keyword, MarkerAnnotated, Modifier, Param, Type,
    };
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_class() {
        assert_eq!(
            parse(&code(
                r#"
final @Anno Test... t
            "#
            )),
            Ok((
                &[] as Tokens,
                Param {
                    modifiers: vec![
                        Modifier::Keyword(Keyword {
                            name: span(1, 1, "final")
                        }),
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 8, "Anno")
                        })),
                    ],
                    tpe: Type::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 13, "Test"),
                        type_args_opt: None
                    }),
                    is_varargs: true,
                    name: span(1, 21, "t"),
                }
            ))
        );
    }

    #[test]
    fn test_array() {
        assert_eq!(
            parse(&code(
                r#"
Test[] t
            "#
            )),
            Ok((
                &[] as Tokens,
                Param {
                    modifiers: vec![],
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(Type::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 1, "Test"),
                            type_args_opt: None
                        })),
                        size_opt: None
                    }),
                    is_varargs: false,
                    name: span(1, 8, "t"),
                }
            ))
        );
    }
}
