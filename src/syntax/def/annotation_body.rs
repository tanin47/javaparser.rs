use nom::bytes::complete::{tag, take, take_till, take_while};
use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::multi::many0;
use syntax::comment;
use syntax::def::{
    annotation, annotation_param, class, constructor, enum_def, field_declarators, interface,
    method,
};
use syntax::statement::block;
use syntax::tree::{AnnotationBody, AnnotationBodyItem, Class, ClassBody};
use syntax::tree::{ClassBodyItem, Method, Span};

pub fn parse_class(input: Span) -> IResult<Span, AnnotationBodyItem> {
    let (input, class) = class::parse(input)?;
    Ok((input, AnnotationBodyItem::Class(class)))
}

pub fn parse_interface(input: Span) -> IResult<Span, AnnotationBodyItem> {
    let (input, interface) = interface::parse(input)?;
    Ok((input, AnnotationBodyItem::Interface(interface)))
}

pub fn parse_annotation(input: Span) -> IResult<Span, AnnotationBodyItem> {
    let (input, annotation) = annotation::parse(input)?;
    Ok((input, AnnotationBodyItem::Annotation(annotation)))
}

pub fn parse_enum(input: Span) -> IResult<Span, AnnotationBodyItem> {
    let (input, enum_def) = enum_def::parse(input)?;
    Ok((input, AnnotationBodyItem::Enum(enum_def)))
}

pub fn parse_field_declarators(input: Span) -> IResult<Span, AnnotationBodyItem> {
    let (input, field_declarators) = field_declarators::parse(input)?;
    Ok((
        input,
        AnnotationBodyItem::FieldDeclarators(field_declarators),
    ))
}

pub fn parse_param(input: Span) -> IResult<Span, AnnotationBodyItem> {
    let (input, param) = annotation_param::parse(input)?;
    Ok((input, AnnotationBodyItem::Param(param)))
}

pub fn parse_item(input: Span) -> IResult<Span, AnnotationBodyItem> {
    alt((
        parse_param,
        parse_class,
        parse_interface,
        parse_enum,
        parse_annotation,
        parse_field_declarators,
    ))(input)
}

pub fn parse_items(input: Span) -> IResult<Span, Vec<AnnotationBodyItem>> {
    many0(parse_item)(input)
}

pub fn parse(input: Span) -> IResult<Span, AnnotationBody> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("{")(input)?;

    let (input, items) = parse_items(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("}")(input)?;

    Ok((input, AnnotationBody { items }))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotation, AnnotationBody, AnnotationBodyItem, AnnotationParam, Block, Class, ClassBody,
        ClassBodyItem, Constructor, Enum, EnumConstant, FieldDeclarators, Interface, Method,
        PrimitiveType, Type, VariableDeclarator,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_empty() {
        assert_eq!(
            parse(code("{}")),
            Ok((span(1, 3, ""), AnnotationBody { items: vec![] }))
        );
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            parse(code(
                r#"
{
  int method();
  class Inner {}
  interface Inner2 {}
  enum Inner3 {}
  @interface Inner4 {}
  int a;
}
            "#
                .trim()
            )),
            Ok((
                span(8, 2, ""),
                AnnotationBody {
                    items: vec![
                        AnnotationBodyItem::Param(AnnotationParam {
                            annotateds: vec![],
                            modifiers: vec![],
                            tpe: primitive(2, 3, "int"),
                            name: span(2, 7, "method"),
                            default_opt: None
                        }),
                        AnnotationBodyItem::Class(Class {
                            modifiers: vec![],
                            name: span(3, 9, "Inner"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            body: ClassBody { items: vec![] }
                        }),
                        AnnotationBodyItem::Interface(Interface {
                            modifiers: vec![],
                            name: span(4, 13, "Inner2"),
                            type_params: vec![],
                            extends: vec![],
                            body: ClassBody { items: vec![] }
                        }),
                        AnnotationBodyItem::Enum(Enum {
                            modifiers: vec![],
                            name: span(5, 8, "Inner3"),
                            implements: vec![],
                            constants: vec![],
                            body_opt: None
                        }),
                        AnnotationBodyItem::Annotation(Annotation {
                            modifiers: vec![],
                            name: span(6, 14, "Inner4"),
                            body: AnnotationBody { items: vec![] }
                        }),
                        AnnotationBodyItem::FieldDeclarators(FieldDeclarators {
                            modifiers: vec![],
                            declarators: vec![VariableDeclarator {
                                tpe: primitive(7, 3, "int"),
                                name: span(7, 7, "a"),
                                expr_opt: None
                            }]
                        }),
                    ]
                }
            ))
        );
    }
}
