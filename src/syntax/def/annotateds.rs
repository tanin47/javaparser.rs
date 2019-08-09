use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::combinator::{map, peek};
use nom::error::ErrorKind;
use nom::multi::{many0, separated_list, separated_listc};
use nom::sequence::preceded;
use syntax::def::{param, type_params};
use syntax::expr::atom::name;
use syntax::statement::block;
use syntax::tree::{
    Annotated, AnnotatedParam, MarkerAnnotated, NormalAnnotated, SingleAnnotated, Span,
};
use syntax::tree::{Class, Method};
use syntax::{comment, expr, tag, tpe};

fn identifier(original: Span) -> IResult<Span, Span> {
    let (input, _) = comment::parse(original)?;
    let (input, name) = name::identifier(input)?;

    if name.fragment == "interface" {
        Err(nom::Err::Error((original, ErrorKind::Tag)))
    } else {
        Ok((input, name))
    }
}

fn parse_param(input: Span) -> IResult<Span, AnnotatedParam> {
    let (input, _) = comment::parse(input)?;
    let (input, name) = name::identifier(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("=")(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, expr) = expr::parse(input)?;

    Ok((input, AnnotatedParam { name, expr }))
}

fn parse_normal(input: Span) -> IResult<Span, Annotated> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("@")(input)?;

    let (input, name) = identifier(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("(")(input)?;

    let (input, params) = separated_list(tag(","), parse_param)(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, Annotated::Normal(NormalAnnotated { name, params })))
}

fn parse_marker(input: Span) -> IResult<Span, Annotated> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("@")(input)?;

    let (input, name) = identifier(input)?;

    Ok((input, Annotated::Marker(MarkerAnnotated { name })))
}

fn parse_single(input: Span) -> IResult<Span, Annotated> {
    let (input, _) = tag("@")(input)?;

    let (input, name) = identifier(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, expr) = expr::parse(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, Annotated::Single(SingleAnnotated { name, expr })))
}

pub fn parse_annotated(input: Span) -> IResult<Span, Annotated> {
    alt((parse_normal, parse_single, parse_marker))(input)
}

pub fn parse(input: Span) -> IResult<Span, Vec<Annotated>> {
    many0(parse_annotated)(input)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, AnnotatedParam, Block, ClassType, Expr, Int, MarkerAnnotated, Method,
        NormalAnnotated, Param, PrimitiveType, ReturnStmt, SingleAnnotated, Statement, Type,
        TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
@Anno
@Anno()
@Anno(1)
@Anno(number=1)
            "#
                .trim()
            )),
            Ok((
                span(4, 16, ""),
                vec![
                    Annotated::Marker(MarkerAnnotated {
                        name: span(1, 2, "Anno")
                    }),
                    Annotated::Normal(NormalAnnotated {
                        name: span(2, 2, "Anno"),
                        params: vec![]
                    }),
                    Annotated::Single(SingleAnnotated {
                        name: span(3, 2, "Anno"),
                        expr: Expr::Int(Int {
                            value: span(3, 7, "1")
                        })
                    }),
                    Annotated::Normal(NormalAnnotated {
                        name: span(4, 2, "Anno"),
                        params: vec![AnnotatedParam {
                            name: span(4, 7, "number"),
                            expr: Expr::Int(Int {
                                value: span(4, 14, "1")
                            })
                        }]
                    }),
                ]
            ))
        );
    }
}
