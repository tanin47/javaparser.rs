use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::multi::{many0, separated_list, separated_nonempty_list};
use syntax::def::{annotateds, annotation_body, class_body, modifiers, type_params};
use syntax::expr::atom::name;
use syntax::tpe::class;
use syntax::tree::{Annotation, Modifier, Span};
use syntax::tree::{Class, ClassType};
use syntax::{comment, tag};

pub fn parse_tail<'a>(
    input: Span<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> IResult<Span<'a>, Annotation<'a>> {
    let (input, name) = name::identifier(input)?;

    let (input, body) = annotation_body::parse(input)?;

    Ok((
        input,
        Annotation {
            modifiers,
            name,
            body,
        },
    ))
}

pub fn parse_prefix(input: Span) -> IResult<Span, Span> {
    let (input, _) = tag("@")(input)?;
    tag("interface")(input)
}

pub fn parse(input: Span) -> IResult<Span, Annotation> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, _) = parse_prefix(input)?;
    parse_tail(input, modifiers)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, Annotation, AnnotationBody, Block, Class, ClassBody, ClassType, Expr, Int,
        Interface, Keyword, MarkerAnnotated, Method, Modifier, Param, PrimitiveType, ReturnStmt,
        Statement, Type, TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
@Anno private @interface Test {}
            "#
                .trim()
            )),
            Ok((
                span(1, 33, ""),
                Annotation {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    name: span(1, 26, "Test"),
                    body: AnnotationBody { items: vec![] }
                }
            ))
        );
    }
}
