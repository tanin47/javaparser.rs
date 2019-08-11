use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::combinator::map;
use nom::multi::{many0, separated_list};
use syntax::def::{annotateds, param, type_params};
use syntax::expr::atom::name;
use syntax::statement::block;
use syntax::tpe::array;
use syntax::tree::{AnnotationParam, Expr, Modifier, Span, Type};
use syntax::tree::{Class, Method};
use syntax::{comment, expr, tag, tpe, word};

fn parse_default(input: Span) -> IResult<Span, Option<Expr>> {
    let (input, _) = comment::parse(input)?;

    match word("default")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, _) = comment::parse(input)?;
            let (input, default) = expr::parse(input)?;
            Ok((input, Some(default)))
        }
        Err(_) => Ok((input, None)),
    }
}

pub fn parse<'a>(
    input: Span<'a>,
    modifiers: Vec<Modifier<'a>>,
    tpe: Type<'a>,
    name: Span<'a>,
) -> IResult<Span<'a>, AnnotationParam<'a>> {
    let (input, _) = tag("(")(input)?;
    let (input, _) = tag(")")(input)?;

    let (input, tpe) = array::parse_tail(input, tpe)?;
    let (input, default_opt) = parse_default(input)?;

    let (input, _) = tag(";")(input)?;

    Ok((
        input,
        AnnotationParam {
            modifiers,
            tpe,
            name,
            default_opt,
        },
    ))
}

#[cfg(test)]
mod tests {
    use syntax::def::annotation_body;
    use syntax::tree::{
        Annotated, AnnotationBodyItem, AnnotationParam, ArrayType, Block, ClassType, Expr, Int,
        Keyword, MarkerAnnotated, Method, Modifier, Param, PrimitiveType, ReturnStmt, Statement,
        Type, TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_full() {
        assert_eq!(
            annotation_body::parse_item(code(
                r#"
@Anno public abstract int field()[] default 1;
            "#
                .trim()
            )),
            Ok((
                span(1, 47, ""),
                AnnotationBodyItem::Param(AnnotationParam {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "public")
                        }),
                        Modifier::Keyword(Keyword {
                            name: span(1, 14, "abstract")
                        })
                    ],
                    name: span(1, 27, "field"),
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(primitive(1, 23, "int")),
                        size_opt: None
                    }),
                    default_opt: Some(Expr::Int(Int {
                        value: span(1, 45, "1")
                    }))
                })
            ))
        );
    }

    #[test]
    fn test() {
        assert_eq!(
            annotation_body::parse_item(code(
                r#"
int field();
            "#
                .trim()
            )),
            Ok((
                span(1, 13, ""),
                AnnotationBodyItem::Param(AnnotationParam {
                    modifiers: vec![],
                    name: span(1, 5, "field"),
                    tpe: primitive(1, 1, "int"),
                    default_opt: None
                })
            ))
        );
    }
}
