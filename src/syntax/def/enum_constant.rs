use nom::bytes::complete::{tag, take, take_till, take_while};
use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list};
use syntax::def::{annotateds, class_body, param, type_params};
use syntax::expr::atom::{method_call, name};
use syntax::statement::block;
use syntax::tree::{Class, Method};
use syntax::tree::{EnumConstant, Span};
use syntax::{comment, tpe};

pub fn parse(input: Span) -> IResult<Span, EnumConstant> {
    let (input, annotateds) = annotateds::parse(input)?;
    let (input, name) = name::identifier(input)?;

    let (input, args_opt) = opt(method_call::parse_args)(input)?;

    let (input, body_opt) = opt(class_body::parse)(input)?;

    Ok((
        input,
        EnumConstant {
            annotateds,
            name,
            args_opt,
            body_opt,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, Block, ClassBody, ClassBodyItem, ClassType, EnumConstant, Expr, Int,
        MarkerAnnotated, Method, Param, PrimitiveType, ReturnStmt, Statement, Type, TypeArg,
        TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
@Anno FIRST
            "#
                .trim()
            )),
            Ok((
                span(1, 12, ""),
                EnumConstant {
                    annotateds: vec![Annotated::Marker(MarkerAnnotated {
                        name: span(1, 2, "Anno")
                    })],
                    name: span(1, 7, "FIRST"),
                    args_opt: None,
                    body_opt: None
                }
            ))
        );
    }

    #[test]
    fn test_with_args_and_body() {
        assert_eq!(
            parse(code(
                r#"
FIRST(1) {
  void method() {}
}
            "#
                .trim()
            )),
            Ok((
                span(3, 2, ""),
                EnumConstant {
                    annotateds: vec![],
                    name: span(1, 1, "FIRST"),
                    args_opt: Some(vec![Expr::Int(Int {
                        value: span(1, 7, "1")
                    })]),
                    body_opt: Some(ClassBody {
                        items: vec![ClassBodyItem::Method(Method {
                            modifiers: vec![],
                            return_type: primitive(2, 3, "void"),
                            name: span(2, 8, "method"),
                            type_params: vec![],
                            params: vec![],
                            throws: vec![],
                            block_opt: Some(Block { stmts: vec![] }),
                        })]
                    })
                }
            ))
        );
    }
}
