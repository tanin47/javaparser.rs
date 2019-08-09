use nom::bytes::complete::{tag, take, take_till, take_while};
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
use syntax::tree::{AnnotationParam, Expr, Span};
use syntax::tree::{Class, Method};
use syntax::{comment, expr, tpe};

fn modifier(input: Span) -> IResult<Span, Span> {
    alt((tag("abstract"), tag("public")))(input)
}

fn modifiers(input: Span) -> IResult<Span, Vec<Span>> {
    let (input, _) = comment::parse(input)?;
    separated_list(comment::parse, modifier)(input)
}

fn parse_default(input: Span) -> IResult<Span, Option<Expr>> {
    let (input, _) = comment::parse(input)?;

    match tag("default")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, _) = comment::parse(input)?;
            let (input, default) = expr::parse(input)?;
            Ok((input, Some(default)))
        }
        Err(_) => Ok((input, None)),
    }
}

pub fn parse(input: Span) -> IResult<Span, AnnotationParam> {
    let (input, annotateds) = annotateds::parse(input)?;
    let (input, modifiers) = modifiers(input)?;
    let (input, original_tpe) = tpe::parse(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, name) = name::identifier(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(")")(input)?;

    let (input, tpe) = array::parse_tail(input, original_tpe)?;

    let (input, default_opt) = parse_default(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(";")(input)?;

    Ok((
        input,
        AnnotationParam {
            annotateds,
            modifiers,
            tpe,
            name,
            default_opt,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, AnnotationParam, ArrayType, Block, ClassType, Expr, Int, MarkerAnnotated,
        Method, Param, PrimitiveType, ReturnStmt, Statement, Type, TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_full() {
        assert_eq!(
            parse(code(
                r#"
@Anno public abstract int field()[] default 1;
            "#
                .trim()
            )),
            Ok((
                span(1, 47, ""),
                AnnotationParam {
                    annotateds: vec![Annotated::Marker(MarkerAnnotated {
                        name: span(1, 2, "Anno")
                    })],
                    modifiers: vec![span(1, 7, "public"), span(1, 14, "abstract")],
                    name: span(1, 27, "field"),
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(primitive(1, 23, "int")),
                        size_opt: None
                    }),
                    default_opt: Some(Expr::Int(Int {
                        value: span(1, 45, "1")
                    }))
                }
            ))
        );
    }

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
int field();
            "#
                .trim()
            )),
            Ok((
                span(1, 13, ""),
                AnnotationParam {
                    annotateds: vec![],
                    modifiers: vec![],
                    name: span(1, 5, "field"),
                    tpe: primitive(1, 1, "int"),
                    default_opt: None
                }
            ))
        );
    }
}
