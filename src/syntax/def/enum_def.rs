use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::combinator::opt;
use nom::multi::{many0, separated_list, separated_nonempty_list};
use syntax::def::{annotateds, class, class_body, enum_constant, modifiers, type_params};
use syntax::expr::atom::name;
use syntax::tree::{Class, ClassBody, ClassType, Modifier};
use syntax::tree::{Enum, Span};
use syntax::{comment, tag};

pub fn parse_tail<'a>(
    input: Span<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> IResult<Span<'a>, Enum<'a>> {
    let (input, name) = name::identifier(input)?;

    let (input, implements) = class::parse_implements(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("{")(input)?;

    let (input, constants) = separated_list(tag(","), enum_constant::parse)(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = opt(tag(","))(input)?;

    let (input, body_opt) = match tag(";")(input) {
        Ok((input, _)) => {
            let (input, items) = class_body::parse_items(input)?;
            (input, Some(ClassBody { items }))
        }
        Err(_) => (input, None),
    };

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("}")(input)?;

    Ok((
        input,
        Enum {
            modifiers,
            name,
            implements,
            constants,
            body_opt,
        },
    ))
}

pub fn parse_prefix(input: Span) -> IResult<Span, Span> {
    tag("enum")(input)
}

pub fn parse(input: Span) -> IResult<Span, Enum> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, _) = parse_prefix(input)?;
    parse_tail(input, modifiers)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, Block, Class, ClassBody, ClassBodyItem, ClassType, Enum, EnumConstant, Expr,
        FieldDeclarators, Int, Keyword, MarkerAnnotated, Method, Modifier, Param, PrimitiveType,
        ReturnStmt, Statement, Type, TypeArg, TypeParam, VariableDeclarator, VariableDeclarators,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
@Anno private enum Test implements Super {
  FIRST_CONSTANT;
  int a;
}
            "#
                .trim()
            )),
            Ok((
                span(4, 2, ""),
                Enum {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    name: span(1, 20, "Test"),
                    implements: vec![ClassType {
                        prefix_opt: None,
                        name: span(1, 36, "Super"),
                        type_args_opt: None
                    }],
                    constants: vec![EnumConstant {
                        annotateds: vec![],
                        name: span(2, 3, "FIRST_CONSTANT"),
                        args_opt: None,
                        body_opt: None
                    }],
                    body_opt: Some(ClassBody {
                        items: vec![ClassBodyItem::FieldDeclarators(FieldDeclarators {
                            modifiers: vec![],
                            declarators: vec![VariableDeclarator {
                                tpe: primitive(3, 3, "int"),
                                name: span(3, 7, "a"),
                                expr_opt: None
                            }]
                        })]
                    })
                }
            ))
        );
    }
}
