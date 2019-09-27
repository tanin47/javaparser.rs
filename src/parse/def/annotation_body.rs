use parse::combinator::{identifier, many0, symbol};
use parse::def::{
    annotation, annotation_param, class, enum_def, field_declarators, interface, modifiers,
};
use parse::tree::{AnnotationBody, AnnotationBodyItem, Modifier, Type};
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

fn parse_class<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, AnnotationBodyItem<'a>> {
    let (input, class) = class::parse_tail(input, modifiers)?;
    Ok((input, AnnotationBodyItem::Class(class)))
}

fn parse_interface<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, AnnotationBodyItem<'a>> {
    let (input, interface) = interface::parse_tail(input, modifiers)?;
    Ok((input, AnnotationBodyItem::Interface(interface)))
}

fn parse_annotation<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, AnnotationBodyItem<'a>> {
    let (input, annotation) = annotation::parse_tail(input, modifiers)?;
    Ok((input, AnnotationBodyItem::Annotation(annotation)))
}

fn parse_enum<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, AnnotationBodyItem<'a>> {
    let (input, enum_def) = enum_def::parse_tail(input, modifiers)?;
    Ok((input, AnnotationBodyItem::Enum(enum_def)))
}

fn parse_field_declarators<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
    tpe: Type<'a>,
) -> ParseResult<'a, AnnotationBodyItem<'a>> {
    let (input, field_declarators) = field_declarators::parse(input, modifiers, tpe)?;
    Ok((
        input,
        AnnotationBodyItem::FieldDeclarators(field_declarators),
    ))
}

fn parse_param<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
    tpe: Type<'a>,
    name: Span<'a>,
) -> ParseResult<'a, AnnotationBodyItem<'a>> {
    let (input, param) = annotation_param::parse(input, modifiers, tpe, name)?;
    Ok((input, AnnotationBodyItem::Param(param)))
}

fn parse_param_or_field_declarators<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, AnnotationBodyItem<'a>> {
    let (input_before_name, tpe) = tpe::parse(input)?;
    let (input, name) = identifier(input_before_name)?;

    if let Ok(_) = symbol('(')(input) {
        parse_param(input, modifiers, tpe, name)
    } else {
        parse_field_declarators(input_before_name, modifiers, tpe)
    }
}

pub fn parse_item(input: Tokens) -> ParseResult<AnnotationBodyItem> {
    let (input, _) = many0(symbol(';'))(input)?;
    let (input, modifiers) = modifiers::parse(input)?;

    if let Ok((input, _)) = enum_def::parse_prefix(input) {
        parse_enum(input, modifiers)
    } else if let Ok((input, _)) = class::parse_prefix(input) {
        parse_class(input, modifiers)
    } else if let Ok((input, _)) = interface::parse_prefix(input) {
        parse_interface(input, modifiers)
    } else if let Ok((input, _)) = annotation::parse_prefix(input) {
        parse_annotation(input, modifiers)
    } else {
        parse_param_or_field_declarators(input, modifiers)
    }
}

pub fn parse_items(input: Tokens) -> ParseResult<Vec<AnnotationBodyItem>> {
    many0(parse_item)(input)
}

pub fn parse(input: Tokens) -> ParseResult<AnnotationBody> {
    let (input, _) = symbol('{')(input)?;
    let (input, items) = parse_items(input)?;
    let (input, _) = many0(symbol(';'))(input)?;
    let (input, _) = symbol('}')(input)?;

    Ok((input, AnnotationBody { items }))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Annotation, AnnotationBody, AnnotationBodyItem, AnnotationParam, Class, ClassBody, Enum,
        FieldDeclarators, Interface, VariableDeclarator,
    };
    use parse::Tokens;
    use std::cell::RefCell;
    use test_common::{code, primitive, span};

    #[test]
    fn test_empty() {
        assert_eq!(
            parse(&code("{}")),
            Ok((&[] as Tokens, AnnotationBody { items: vec![] }))
        );
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            parse(&code(
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
            )),
            Ok((
                &[] as Tokens,
                AnnotationBody {
                    items: vec![
                        AnnotationBodyItem::Param(AnnotationParam {
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
                            body: ClassBody { items: vec![] },
                            def_opt: RefCell::new(None)
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
