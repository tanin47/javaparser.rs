use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::multispace1;
use nom::combinator::peek;
use nom::sequence::tuple;
use nom::{FindToken, IResult};
use syntax::expr::precedence_10;
use syntax::tree::{BinaryOperation, Expr, InstanceOf, Span};
use syntax::{tag, tag_and_followed_by, tpe};

fn op(input: Tokens) -> ParseResult<Span> {
    alt((
        tag("<="),
        tag(">="),
        tag_and_followed_by("<", is_not("<")),
        tag_and_followed_by(">", is_not(">")),
        tag_and_followed_by("instanceof", multispace1),
    ))(input)
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    if let Ok((input, operator)) = op(input) {
        if operator.fragment == "instanceof" {
            let (input, tpe) = tpe::parse(input)?;

            Ok((
                input,
                Expr::InstanceOf(InstanceOf {
                    expr: Box::new(left),
                    operator,
                    tpe,
                }),
            ))
        } else {
            let (input, right) = precedence_10::parse(input)?;

            let expr = Expr::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });

            parse_tail(expr, input)
        }
    } else {
        Ok((input, left))
    }
}

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, left) = precedence_10::parse(input)?;
    parse_tail(left, input)
}

#[cfg(test)]
mod tests {
    use syntax::tree::{
        ArrayAccess, Assigned, Assignment, BinaryOperation, ClassType, Expr, FieldAccess,
        InstanceOf, Int, LiteralString, Method, MethodCall, Name, ReturnStmt, Type, TypeArg,
    };
    use test_common::{code, span};

    use super::parse;

    #[test]
    fn test_instanceof() {
        assert_eq!(
            parse(code(
                r#"
a instanceof Class
            "#
                .trim()
            )),
            Ok((
                span(1, 19, ""),
                Expr::InstanceOf(InstanceOf {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "instanceof"),
                    tpe: Type::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 14, "Class"),
                        type_args_opt: None
                    })
                })
            ))
        );
    }
}
