use parse::combinator::{identifier, many0, symbol};
use parse::def::{
    annotation, class, constructor, enum_def, field_declarators, interface, method, modifiers,
    type_params,
};
use parse::id_gen::IdGen;
use parse::statement::block;
use parse::tree::{ClassBody, ClassBodyItem, Modifier, Type, TypeParam};
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

fn parse_class<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, class) = class::parse_tail(input, modifiers, id_gen)?;
    Ok((input, ClassBodyItem::Class(class)))
}

fn parse_interface<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, interface) = interface::parse_tail(input, modifiers, id_gen)?;
    Ok((input, ClassBodyItem::Interface(interface)))
}

fn parse_annotation<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, annotation) = annotation::parse_tail(input, modifiers, id_gen)?;
    Ok((input, ClassBodyItem::Annotation(annotation)))
}

fn parse_enum<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, enum_def) = enum_def::parse_tail(input, modifiers, id_gen)?;
    Ok((input, ClassBodyItem::Enum(enum_def)))
}

fn parse_method<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    type_params: Vec<TypeParam<'def>>,
    tpe: Type<'def>,
    name: Span<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, method) = method::parse(input, modifiers, type_params, tpe, name, id_gen)?;
    Ok((input, ClassBodyItem::Method(method)))
}

fn parse_constructor<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    type_params: Vec<TypeParam<'def>>,
    name: Span<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, constructor) = constructor::parse(input, modifiers, type_params, name, id_gen)?;
    Ok((input, ClassBodyItem::Constructor(constructor)))
}

fn parse_field_declarators<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    tpe: Type<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, field_declarators) = field_declarators::parse(input, modifiers, tpe, id_gen)?;
    Ok((input, ClassBodyItem::FieldDeclarators(field_declarators)))
}

fn parse_static_block<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, block) = block::parse_block(input, id_gen)?;
    Ok((input, ClassBodyItem::StaticInitializer(block)))
}

fn parse_method_constructor_or_field<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, type_params) = type_params::parse(input, id_gen)?;

    if let Ok((input, ident)) = identifier(input) {
        if let Ok(_) = symbol('(')(input) {
            return parse_constructor(input, modifiers, type_params, ident, id_gen);
        }
    }

    let (input_before_name, tpe) = tpe::parse(input)?;
    let (input, name) = identifier(input_before_name)?;

    if let Ok(_) = symbol('(')(input) {
        parse_method(input, modifiers, type_params, tpe, name, id_gen)
    } else {
        parse_field_declarators(input_before_name, modifiers, tpe, id_gen)
    }
}

pub fn parse_item<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBodyItem<'def>> {
    let (input, _) = many0(symbol(';'))(input)?;
    let (input, modifiers) = modifiers::parse(input, id_gen)?;

    if let Ok((input, _)) = class::parse_prefix(input) {
        parse_class(input, modifiers, id_gen)
    } else if let Ok((input, _)) = enum_def::parse_prefix(input) {
        parse_enum(input, modifiers, id_gen)
    } else if let Ok((input, _)) = interface::parse_prefix(input) {
        parse_interface(input, modifiers, id_gen)
    } else if let Ok((input, _)) = annotation::parse_prefix(input) {
        parse_annotation(input, modifiers, id_gen)
    } else if let Ok(_) = symbol('{')(input) {
        parse_static_block(input, id_gen)
    } else {
        parse_method_constructor_or_field(input, modifiers, id_gen)
    }
}

pub fn parse_items<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Vec<ClassBodyItem<'def>>> {
    many0(|input| parse_item(input, id_gen))(input)
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ClassBody<'def>> {
    let (input, _) = symbol('{')(input)?;
    let (input, items) = parse_items(input, id_gen)?;
    let (input, _) = many0(symbol(';'))(input)?;
    let (input, _) = symbol('}')(input)?;

    Ok((input, ClassBody { items }))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        Annotation, AnnotationBody, Block, Class, ClassBody, ClassBodyItem, Constructor, Enum,
//        FieldDeclarators, Interface, Method, PrimitiveType, PrimitiveTypeType, Type,
//        VariableDeclarator, Void,
//    };
//    use parse::Tokens;
//    use std::cell::RefCell;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test_empty() {
//        assert_eq!(
//            parse(&generate_tokens("{}")),
//            Ok((&[] as Tokens, ClassBody { items: vec![] }))
//        );
//    }
//
//    #[test]
//    fn test_multiple() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//{
//  void method() {}
//  class Inner {}
//  int a;
//  static {}
//  Constructor() {}
//  interface Inner2 {}
//  enum Inner3 {}
//  @interface Inner4 {}
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBody {
//                    items: vec![
//                        ClassBodyItem::Method(Method {
//                            modifiers: vec![],
//                            return_type: Type::Void(Void {
//                                span_opt: Some(span(2, 3, "void"))
//                            }),
//                            name: span(2, 8, "method"),
//                            type_params: vec![],
//                            params: vec![],
//                            throws: vec![],
//                            block_opt: Some(Block { stmts: vec![] }),
//                            def_opt: RefCell::new(None)
//                        }),
//                        ClassBodyItem::Class(Class {
//                            modifiers: vec![],
//                            name: span(3, 9, "Inner"),
//                            type_params: vec![],
//                            extend_opt: None,
//                            implements: vec![],
//                            body: ClassBody { items: vec![] },
//                            def_opt: RefCell::new(None)
//                        }),
//                        ClassBodyItem::FieldDeclarators(FieldDeclarators {
//                            modifiers: vec![],
//                            declarators: vec![VariableDeclarator {
//                                tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                                    span_opt: Some(span(4, 3, "int")),
//                                    tpe: PrimitiveTypeType::Int
//                                })),
//                                name: span(4, 7, "a"),
//                                expr_opt: None
//                            }]
//                        }),
//                        ClassBodyItem::StaticInitializer(Block { stmts: vec![] }),
//                        ClassBodyItem::Constructor(Constructor {
//                            modifiers: vec![],
//                            name: span(6, 3, "Constructor"),
//                            type_params: vec![],
//                            params: vec![],
//                            throws: vec![],
//                            block: Block { stmts: vec![] },
//                        }),
//                        ClassBodyItem::Interface(Interface {
//                            modifiers: vec![],
//                            name: span(7, 13, "Inner2"),
//                            type_params: vec![],
//                            extends: vec![],
//                            body: ClassBody { items: vec![] }
//                        }),
//                        ClassBodyItem::Enum(Enum {
//                            modifiers: vec![],
//                            name: span(8, 8, "Inner3"),
//                            implements: vec![],
//                            constants: vec![],
//                            body_opt: None
//                        }),
//                        ClassBodyItem::Annotation(Annotation {
//                            modifiers: vec![],
//                            name: span(9, 14, "Inner4"),
//                            body: AnnotationBody { items: vec![] }
//                        }),
//                    ]
//                }
//            ))
//        );
//    }
//}
