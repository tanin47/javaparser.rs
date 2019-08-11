use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::combinator::{map, opt, peek};
use nom::error::ErrorKind;
use nom::multi::{many0, separated_list, separated_listc, separated_nonempty_list};
use nom::sequence::{preceded, tuple};
use syntax::def::{param, type_params};
use syntax::expr::atom::name;
use syntax::statement::block;
use syntax::tree::{
    Annotated, AnnotatedParam, MarkerAnnotated, NormalAnnotated, SingleAnnotated, Span,
};
use syntax::tree::{Class, Method};
use syntax::{comment, expr, tag, tag_and_followed_by, tpe};

fn identifier(original: Span) -> IResult<Span, Span> {
    let (input, name) = name::identifier(original)?;

    if name.fragment == "interface" {
        Err(nom::Err::Error((original, ErrorKind::Tag)))
    } else {
        Ok((input, name))
    }
}

fn parse_param(input: Span) -> IResult<Span, AnnotatedParam> {
    let (input, name) = name::identifier(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, expr) = expr::parse(input)?;

    Ok((input, AnnotatedParam { name, expr }))
}

pub fn parse_annotated(input: Span) -> IResult<Span, Annotated> {
    let (input, _) = tag("@")(input)?;
    let (input, name) = identifier(input)?;

    if let Ok((input, _)) = tag("(")(input) {
        if let Ok((input, _)) = tag(")")(input) {
            Ok((
                input,
                Annotated::Normal(NormalAnnotated {
                    name,
                    params: vec![],
                }),
            ))
        } else if let Ok((input, params)) = separated_nonempty_list(tag(","), parse_param)(input) {
            let (input, _) = tag(")")(input)?;
            Ok((input, Annotated::Normal(NormalAnnotated { name, params })))
        } else {
            let (input, expr) = expr::parse(input)?;
            let (input, _) = tag(")")(input)?;
            Ok((input, Annotated::Single(SingleAnnotated { name, expr })))
        }
    } else {
        let (input, _) = opt(tuple((tag("("), tag(")"))))(input)?;
        Ok((input, Annotated::Marker(MarkerAnnotated { name })))
    }
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
