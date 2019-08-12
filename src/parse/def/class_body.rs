use parse::combinator::{identifier, many0, symbol};
use parse::def::{class, interface, method, modifiers, type_params};
use parse::tree::{ClassBody, ClassBodyItem, Modifier, Type, TypeParam};
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

fn parse_class<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, ClassBodyItem<'a>> {
    let (input, class) = class::parse_tail(input, modifiers)?;
    Ok((input, ClassBodyItem::Class(class)))
}

fn parse_interface<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, ClassBodyItem<'a>> {
    let (input, interface) = interface::parse_tail(input, modifiers)?;
    Ok((input, ClassBodyItem::Interface(interface)))
}
//
//fn parse_annotation<'a>(
//    input: Tokens<'a>,
//    modifiers: Vec<Modifier<'a>>,
//) -> ParseResult<ClassBodyItem<'a>> {
//    let (input, annotation) = annotation::parse_tail(input, modifiers)?;
//    Ok((input, ClassBodyItem::Annotation(annotation)))
//}
//
//fn parse_enum<'a>(
//    input: Tokens<'a>,
//    modifiers: Vec<Modifier<'a>>,
//) -> ParseResult<ClassBodyItem<'a>> {
//    let (input, enum_def) = enum_def::parse_tail(input, modifiers)?;
//    Ok((input, ClassBodyItem::Enum(enum_def)))
//}

fn parse_method<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
    type_params: Vec<TypeParam<'a>>,
    tpe: Type<'a>,
    name: Span<'a>,
) -> ParseResult<'a, ClassBodyItem<'a>> {
    let (input, method) = method::parse(input, modifiers, type_params, tpe, name)?;
    Ok((input, ClassBodyItem::Method(method)))
}

//fn parse_constructor<'a>(
//    input: Tokens<'a>,
//    modifiers: Vec<Modifier<'a>>,
//    type_params: Vec<TypeParam<'a>>,
//    name: Span<'a>,
//) -> ParseResult<ClassBodyItem<'a>> {
//    let (input, constructor) = constructor::parse(input, modifiers, type_params, name)?;
//    Ok((input, ClassBodyItem::Constructor(constructor)))
//}
//
//fn parse_field_declarators<'a>(
//    input: Tokens<'a>,
//    modifiers: Vec<Modifier<'a>>,
//    tpe: Type<'a>,
//) -> ParseResult<ClassBodyItem<'a>> {
//    let (input, field_declarators) = field_declarators::parse(input, modifiers, tpe)?;
//    Ok((input, ClassBodyItem::FieldDeclarators(field_declarators)))
//}
//
//fn parse_static_block(input: Tokens) -> ParseResult<ClassBodyItem> {
//    let (input, block) = block::parse_block(input)?;
//    Ok((input, ClassBodyItem::StaticInitializer(block)))
//}
//
fn parse_method_constructor_or_field<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, ClassBodyItem<'a>> {
    let (input_before_type, type_params) = type_params::parse(input)?;
    let (input, ident) = identifier(input_before_type)?;

    if let Ok(_) = symbol("(")(input) {
        Err(input)
    //        parse_constructor(input, modifiers, type_params, ident)
    } else {
        let (input_before_name, tpe) = tpe::parse(input_before_type)?;
        let (input, name) = identifier(input_before_name)?;

        if let Ok(_) = symbol("(")(input) {
            parse_method(input, modifiers, type_params, tpe, name)
        } else {
            Err(input)
            //            parse_field_declarators(input_before_name, modifiers, tpe)
        }
    }
}

pub fn parse_item(input: Tokens) -> ParseResult<ClassBodyItem> {
    let (input, modifiers) = modifiers::parse(input)?;

    if let Ok((input, _)) = class::parse_prefix(input) {
        parse_class(input, modifiers)
    //    } else if let Ok((input, _)) = enum_def::parse_prefix(input) {
    //        parse_enum(input, modifiers)
    } else if let Ok((input, _)) = interface::parse_prefix(input) {
        parse_interface(input, modifiers)
    //    } else if let Ok((input, _)) = annotation::parse_prefix(input) {
    //        parse_annotation(input, modifiers)
    //    } else if let Ok((input, _)) = peek(tag("{"))(input) {
    //        parse_static_block(input)
    } else {
        parse_method_constructor_or_field(input, modifiers)
    }
}

pub fn parse_items(input: Tokens) -> ParseResult<Vec<ClassBodyItem>> {
    many0(parse_item)(input)
}

pub fn parse(input: Tokens) -> ParseResult<ClassBody> {
    let (input, _) = symbol("{")(input)?;
    let (input, items) = parse_items(input)?;
    let (input, _) = symbol("}")(input)?;

    Ok((input, ClassBody { items }))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Annotation, AnnotationBody, Block, Class, ClassBody, ClassBodyItem, Constructor, Enum,
        FieldDeclarators, Interface, Method, PrimitiveType, Type, VariableDeclarator,
    };
    use parse::Tokens;
    use test_common::{code, primitive, span};

    #[test]
    fn test_empty() {
        assert_eq!(
            parse(&code("{}")),
            Ok((&[] as Tokens, ClassBody { items: vec![] }))
        );
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            parse(&code(
                r#"
{
  void method() {}
  class Inner {}
  int a;
  static {}
  Constructor() {}
  interface Inner2 {}
  enum Inner3 {}
  @interface Inner4 {}
}
            "#
            )),
            Ok((
                &[] as Tokens,
                ClassBody {
                    items: vec![
                        ClassBodyItem::Method(Method {
                            modifiers: vec![],
                            return_type: primitive(2, 3, "void"),
                            name: span(2, 8, "method"),
                            type_params: vec![],
                            params: vec![],
                            throws: vec![],
                            block_opt: Some(Block { stmts: vec![] }),
                        }),
                        ClassBodyItem::Class(Class {
                            modifiers: vec![],
                            name: span(3, 9, "Inner"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            body: ClassBody { items: vec![] }
                        }),
                        ClassBodyItem::FieldDeclarators(FieldDeclarators {
                            modifiers: vec![],
                            declarators: vec![VariableDeclarator {
                                tpe: Type::Primitive(PrimitiveType {
                                    name: span(4, 3, "int")
                                }),
                                name: span(4, 7, "a"),
                                expr_opt: None
                            }]
                        }),
                        ClassBodyItem::StaticInitializer(Block { stmts: vec![] }),
                        ClassBodyItem::Constructor(Constructor {
                            modifiers: vec![],
                            name: span(6, 3, "Constructor"),
                            type_params: vec![],
                            params: vec![],
                            block: Block { stmts: vec![] },
                        }),
                        ClassBodyItem::Interface(Interface {
                            modifiers: vec![],
                            name: span(7, 13, "Inner2"),
                            type_params: vec![],
                            extends: vec![],
                            body: ClassBody { items: vec![] }
                        }),
                        ClassBodyItem::Enum(Enum {
                            modifiers: vec![],
                            name: span(8, 8, "Inner3"),
                            implements: vec![],
                            constants: vec![],
                            body_opt: None
                        }),
                        ClassBodyItem::Annotation(Annotation {
                            modifiers: vec![],
                            name: span(9, 14, "Inner4"),
                            body: AnnotationBody { items: vec![] }
                        }),
                    ]
                }
            ))
        );
    }
}
