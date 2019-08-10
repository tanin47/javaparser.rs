use nom::IResult;

use nom::branch::alt;
use nom::multi::many0;
use syntax::def::{annotation, class, constructor, enum_def, field_declarators, interface, method};
use syntax::statement::block;
use syntax::tree::{Class, ClassBody};
use syntax::tree::{ClassBodyItem, Method, Span};
use syntax::{comment, tag};

pub fn parse_class(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, class) = class::parse(input)?;
    Ok((input, ClassBodyItem::Class(class)))
}

pub fn parse_interface(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, interface) = interface::parse(input)?;
    Ok((input, ClassBodyItem::Interface(interface)))
}

pub fn parse_annotation(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, annotation) = annotation::parse(input)?;
    Ok((input, ClassBodyItem::Annotation(annotation)))
}

pub fn parse_enum(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, enum_def) = enum_def::parse(input)?;
    Ok((input, ClassBodyItem::Enum(enum_def)))
}

pub fn parse_method(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, method) = method::parse(input)?;
    Ok((input, ClassBodyItem::Method(method)))
}

pub fn parse_constructor(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, constructor) = constructor::parse(input)?;
    Ok((input, ClassBodyItem::Constructor(constructor)))
}

pub fn parse_field_declarators(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, field_declarators) = field_declarators::parse(input)?;
    Ok((input, ClassBodyItem::FieldDeclarators(field_declarators)))
}

pub fn parse_static_block(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("static")(input)?;

    let (input, _) = comment::parse1(input)?;
    let (input, block) = block::parse_block(input)?;
    Ok((input, ClassBodyItem::StaticInitializer(block)))
}

pub fn parse_item(input: Span) -> IResult<Span, ClassBodyItem> {
    let (input, _) = comment::parse(input)?;
    alt((
        parse_constructor,
        parse_method,
        parse_class,
        parse_interface,
        parse_annotation,
        parse_enum,
        parse_field_declarators,
        parse_static_block,
    ))(input)
}

pub fn parse_items(input: Span) -> IResult<Span, Vec<ClassBodyItem>> {
    many0(parse_item)(input)
}

pub fn parse(input: Span) -> IResult<Span, ClassBody> {
    let (input, _) = tag("{")(input)?;
    let (input, items) = parse_items(input)?;
    let (input, _) = tag("}")(input)?;

    Ok((input, ClassBody { items }))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotation, AnnotationBody, Block, Class, ClassBody, ClassBodyItem, Constructor, Enum,
        FieldDeclarators, Interface, Method, PrimitiveType, Type, VariableDeclarator,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_empty() {
        assert_eq!(
            parse(code("{}")),
            Ok((span(1, 3, ""), ClassBody { items: vec![] }))
        );
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            parse(code(
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
                .trim()
            )),
            Ok((
                span(10, 2, ""),
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
                            annotateds: vec![],
                            name: span(6, 3, "Constructor"),
                            modifiers: vec![],
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
