use parse::combinator::{identifier, many0, symbol};
use parse::def::{
    annotation, annotation_param, class, enum_def, field_declarators, interface, modifiers,
};
use parse::id_gen::IdGen;
use parse::tree::{AnnotationBody, AnnotationBodyItem, Modifier, Type};
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

fn parse_class<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input, class) = class::parse_tail(input, modifiers, id_gen)?;
    Ok((input, AnnotationBodyItem::Class(class)))
}

fn parse_interface<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input, interface) = interface::parse_tail(input, modifiers, id_gen)?;
    Ok((input, AnnotationBodyItem::Interface(interface)))
}

fn parse_annotation<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input, annotation) = annotation::parse_tail(input, modifiers, id_gen)?;
    Ok((input, AnnotationBodyItem::Annotation(annotation)))
}

fn parse_enum<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input, enum_def) = enum_def::parse_tail(input, modifiers, id_gen)?;
    Ok((input, AnnotationBodyItem::Enum(enum_def)))
}

fn parse_field_declarators<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    tpe: Type<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input, field_declarators) = field_declarators::parse(input, modifiers, tpe, id_gen)?;
    Ok((
        input,
        AnnotationBodyItem::FieldDeclarators(field_declarators),
    ))
}

fn parse_param<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    tpe: Type<'def>,
    name: Span<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input, param) = annotation_param::parse(input, modifiers, tpe, name, id_gen)?;
    Ok((input, AnnotationBodyItem::Param(param)))
}

fn parse_param_or_field_declarators<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input_before_name, tpe) = tpe::parse(input)?;
    let (input, name) = identifier(input_before_name)?;

    if let Ok(_) = symbol('(')(input) {
        parse_param(input, modifiers, tpe, name, id_gen)
    } else {
        parse_field_declarators(input_before_name, modifiers, tpe, id_gen)
    }
}

pub fn parse_item<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBodyItem<'def>> {
    let (input, _) = many0(symbol(';'))(input)?;
    let (input, modifiers) = modifiers::parse(input, id_gen)?;

    if let Ok((input, _)) = enum_def::parse_prefix(input) {
        parse_enum(input, modifiers, id_gen)
    } else if let Ok((input, _)) = class::parse_prefix(input) {
        parse_class(input, modifiers, id_gen)
    } else if let Ok((input, _)) = interface::parse_prefix(input) {
        parse_interface(input, modifiers, id_gen)
    } else if let Ok((input, _)) = annotation::parse_prefix(input) {
        parse_annotation(input, modifiers, id_gen)
    } else {
        parse_param_or_field_declarators(input, modifiers, id_gen)
    }
}

pub fn parse_items<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Vec<AnnotationBodyItem<'def>>> {
    many0(|i| parse_item(i, id_gen))(input)
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotationBody<'def>> {
    let (input, _) = symbol('{')(input)?;
    let (input, items) = parse_items(input, id_gen)?;
    let (input, _) = many0(symbol(';'))(input)?;
    let (input, _) = symbol('}')(input)?;

    Ok((input, AnnotationBody { items }))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        Annotation, AnnotationBody, AnnotationBodyItem, AnnotationParam, Class, ClassBody, Enum,
//        FieldDeclarators, Interface, VariableDeclarator,
//    };
//    use parse::Tokens;
//    use std::cell::RefCell;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test_empty() {
//        assert_eq!(
//            parse(&generate_tokens("{}")),
//            Ok((&[] as Tokens, AnnotationBody { items: vec![] }))
//        );
//    }
//
//    #[test]
//    fn test_multiple() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//{
//  int method();
//  class Inner {}
//  interface Inner2 {}
//  enum Inner3 {}
//  @interface Inner4 {}
//  int a;
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                AnnotationBody {
//                    items: vec![
//                        AnnotationBodyItem::Param(AnnotationParam {
//                            modifiers: vec![],
//                            tpe: primitive(2, 3, "int"),
//                            name: span(2, 7, "method"),
//                            default_opt: None
//                        }),
//                        AnnotationBodyItem::Class(Class {
//                            modifiers: vec![],
//                            name: span(3, 9, "Inner"),
//                            type_params: vec![],
//                            extend_opt: None,
//                            implements: vec![],
//                            body: ClassBody { items: vec![] },
//                            def_opt: RefCell::new(None)
//                        }),
//                        AnnotationBodyItem::Interface(Interface {
//                            modifiers: vec![],
//                            name: span(4, 13, "Inner2"),
//                            type_params: vec![],
//                            extends: vec![],
//                            body: ClassBody { items: vec![] }
//                        }),
//                        AnnotationBodyItem::Enum(Enum {
//                            modifiers: vec![],
//                            name: span(5, 8, "Inner3"),
//                            implements: vec![],
//                            constants: vec![],
//                            body_opt: None
//                        }),
//                        AnnotationBodyItem::Annotation(Annotation {
//                            modifiers: vec![],
//                            name: span(6, 14, "Inner4"),
//                            body: AnnotationBody { items: vec![] }
//                        }),
//                        AnnotationBodyItem::FieldDeclarators(FieldDeclarators {
//                            modifiers: vec![],
//                            declarators: vec![VariableDeclarator {
//                                tpe: RefCell::new(primitive(7, 3, "int")),
//                                name: span(7, 7, "a"),
//                                expr_opt: None
//                            }]
//                        }),
//                    ]
//                }
//            ))
//        );
//    }
//}
