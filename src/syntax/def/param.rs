use nom::IResult;

use nom::combinator::opt;
use syntax::def::{annotateds, modifiers};
use syntax::expr::atom::name;
use syntax::statement::block;
use syntax::tree::{Class, Method};
use syntax::tree::{Param, Span};
use syntax::{comment, statement, tag, tpe};

pub fn parse(input: Span) -> IResult<Span, Param> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, tpe) = tpe::parse(input)?;
    let (input, varargs_opt) = opt(tag("..."))(input)?;
    let (input, name) = name::identifier(input)?;

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
    use syntax::tree::{
        Annotated, ArrayType, Block, ClassType, Expr, Int, Keyword, MarkerAnnotated, Method,
        Modifier, Param, ReturnStmt, Statement, Type,
    };
    use test_common::{code, span};

    #[test]
    fn test_class() {
        assert_eq!(
            parse(code(
                r#"
final @Anno Test... t
            "#
                .trim()
            )),
            Ok((
                span(1, 22, ""),
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
            parse(code(
                r#"
Test[] t
            "#
                .trim()
            )),
            Ok((
                span(1, 9, ""),
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
