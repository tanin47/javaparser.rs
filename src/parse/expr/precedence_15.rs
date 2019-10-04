use either::Either;
use parse::combinator::symbol2;
use parse::expr::atom::name;
use parse::expr::precedence_16;
use parse::tpe::type_args;
use parse::tree::{
    ClassType, ConstructorReference, EnclosingType, Expr, FieldAccess, MethodReference,
    MethodReferencePrimary, ReferenceType,
};
use parse::{ParseResult, Tokens};
use std::cell::Cell;

fn convert_field_to_class(field: FieldAccess) -> Result<ClassType, ()> {
    let prefix = match *field.expr {
        Expr::FieldAccess(parent) => convert_field_to_class(parent)?,
        Expr::Name(parent) => ClassType {
            prefix_opt: None,
            name: parent.name,
            type_args_opt: None,
            def_opt: None,
        },
        _ => return Err(()),
    };

    Ok(ClassType {
        prefix_opt: Some(Box::new(EnclosingType::Class(prefix))),
        name: field.field.name,
        type_args_opt: None,
        def_opt: None,
    })
}

pub fn convert_to_type(expr: Expr) -> Result<ClassType, ()> {
    match expr {
        Expr::Name(name) => Ok(ClassType {
            prefix_opt: None,
            name: name.name,
            type_args_opt: None,
            def_opt: None,
        }),
        Expr::FieldAccess(field) => {
            if let Ok(class) = convert_field_to_class(field) {
                Ok(class)
            } else {
                Err(())
            }
        }
        _ => Err(()),
    }
}

pub fn parse_tail<'def, 'r>(
    primary: MethodReferencePrimary<'def>,
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, _) = symbol2(':', ':')(input)?;

    let (input, type_args_opt) = type_args::parse(input)?;
    let (input, keyword_or_name) = name::parse(input)?;

    match keyword_or_name {
        Either::Left(keyword) => {
            if keyword.name.fragment == "new" {
                let ref_type = match primary {
                    MethodReferencePrimary::Array(arr) => ReferenceType::Array(arr),
                    MethodReferencePrimary::Class(class) => ReferenceType::Class(class),
                    MethodReferencePrimary::Expr(expr) => {
                        if let Ok(class) = convert_to_type(*expr) {
                            ReferenceType::Class(class)
                        } else {
                            return Err(input);
                        }
                    }
                };

                Ok((
                    input,
                    Expr::ConstructorReference(ConstructorReference {
                        tpe: ref_type,
                        type_args_opt,
                    }),
                ))
            } else {
                Err(input)
            }
        }
        Either::Right(name) => Ok((
            input,
            Expr::MethodReference(MethodReference {
                primary,
                type_args_opt,
                name: name.name,
            }),
        )),
    }
}

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, expr) = precedence_16::parse(input)?;

    if let Ok(_) = symbol2(':', ':')(input) {
        let (input, method_ref) = parse_tail(MethodReferencePrimary::Expr(Box::new(expr)), input)?;
        Ok((input, method_ref))
    } else {
        Ok((input, expr))
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        ArrayAccess, ClassType, Expr, FieldAccess, LiteralString, MethodReference,
        MethodReferencePrimary, Name, TypeArg,
    };
    use parse::Tokens;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_method_ref() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
"abc"::length
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodReference(MethodReference {
                    primary: MethodReferencePrimary::Expr(Box::new(Expr::String(LiteralString {
                        value: span(1, 1, "\"abc\"")
                    }))),
                    type_args_opt: None,
                    name: span(1, 8, "length")
                })
            ))
        );
    }

    #[test]
    fn test_method_ref_2() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
foo[x]::<A>bar
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodReference(MethodReference {
                    primary: MethodReferencePrimary::Expr(Box::new(Expr::ArrayAccess(
                        ArrayAccess {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 1, "foo")
                            })),
                            index: Box::new(Expr::Name(Name {
                                name: span(1, 5, "x")
                            }))
                        }
                    ))),
                    type_args_opt: Some(vec![TypeArg::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 10, "A"),
                        type_args_opt: None,
                        def_opt: None
                    })]),
                    name: span(1, 12, "bar")
                })
            ))
        );
    }

    #[test]
    fn test_method_ref_3() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
foo.bar::zzz
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodReference(MethodReference {
                    primary: MethodReferencePrimary::Expr(Box::new(Expr::FieldAccess(
                        FieldAccess {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 1, "foo")
                            })),
                            field: Name {
                                name: span(1, 5, "bar")
                            }
                        }
                    ))),
                    type_args_opt: None,
                    name: span(1, 10, "zzz")
                })
            ))
        );
    }
}
