use nom::character::is_digit;
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::multi::separated_list;
use nom::IResult;
use syntax::def::class_body;
use syntax::expr::atom::name;
use syntax::tpe::type_args;
use syntax::tree::{ClassType, Expr, Int, NewObject, Span, Type, TypeArg};
use syntax::{comment, expr, tag, tpe};

pub fn parse_tail1(input: Span) -> IResult<Span, Expr> {
    let (input, constructor_type_args_opt) = type_args::parse(input)?;

    parse_tail2(input, constructor_type_args_opt)
}

pub fn parse_tail2<'a>(
    input: Span<'a>,
    constructor_type_args_opt: Option<Vec<TypeArg<'a>>>,
) -> IResult<Span<'a>, Expr<'a>> {
    let (input, tpe) = tpe::class::parse_no_array(input)?;

    parse_tail3(input, constructor_type_args_opt, tpe)
}

pub fn parse_tail3<'a>(
    input: Span<'a>,
    constructor_type_args_opt: Option<Vec<TypeArg<'a>>>,
    tpe: ClassType<'a>,
) -> IResult<Span<'a>, Expr<'a>> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("(")(input)?;

    let (input, args) = separated_list(tag(","), expr::parse)(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(")")(input)?;

    let (input, _) = comment::parse(input)?;

    let (input, body_opt) = opt(class_body::parse)(input)?;

    Ok((
        input,
        Expr::NewObject(NewObject {
            tpe,
            constructor_type_args_opt,
            args,
            body_opt,
        }),
    ))
}

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = comment::parse(input)?;
    let (input, name) = name::identifier(input)?;

    if name.fragment == "new" {
        parse_tail1(input)
    } else {
        Err(nom::Err::Error((input, ErrorKind::Tag)))
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Block, ClassBody, ClassType, Expr, Int, LiteralString, Method, NewObject, ReturnStmt,
        TypeArg,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_type_args() {
        assert_eq!(
            parse(code(
                r#"
new <String>Test<Integer>()
            "#
                .trim()
            )),
            Ok((
                span(1, 28, ""),
                Expr::NewObject(NewObject {
                    tpe: ClassType {
                        prefix_opt: None,
                        name: span(1, 13, "Test"),
                        type_args_opt: Some(vec![TypeArg::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 18, "Integer"),
                            type_args_opt: None
                        })])
                    },
                    constructor_type_args_opt: Some(vec![TypeArg::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 6, "String"),
                        type_args_opt: None
                    })]),
                    args: vec![],
                    body_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(code(
                r#"
new Test()
            "#
                .trim()
            )),
            Ok((
                span(1, 11, ""),
                Expr::NewObject(NewObject {
                    tpe: ClassType {
                        prefix_opt: None,
                        name: span(1, 5, "Test"),
                        type_args_opt: None
                    },
                    constructor_type_args_opt: None,
                    args: vec![],
                    body_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_with_args() {
        assert_eq!(
            parse(code(
                r#"
new Test(1, "a")
            "#
                .trim()
            )),
            Ok((
                span(1, 17, ""),
                Expr::NewObject(NewObject {
                    tpe: ClassType {
                        prefix_opt: None,
                        name: span(1, 5, "Test"),
                        type_args_opt: None
                    },
                    constructor_type_args_opt: None,
                    args: vec![
                        Expr::Int(Int {
                            value: span(1, 10, "1")
                        }),
                        Expr::String(LiteralString {
                            value: span(1, 14, "a")
                        })
                    ],
                    body_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_anonymous() {
        assert_eq!(
            parse(code(
                r#"
new Test() {
}
            "#
                .trim()
            )),
            Ok((
                span(2, 2, ""),
                Expr::NewObject(NewObject {
                    tpe: ClassType {
                        prefix_opt: None,
                        name: span(1, 5, "Test"),
                        type_args_opt: None
                    },
                    constructor_type_args_opt: None,
                    args: vec![],
                    body_opt: Some(ClassBody { items: vec![] })
                })
            ))
        );
    }
}
