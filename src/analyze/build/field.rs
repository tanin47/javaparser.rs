use analyze::build::modifier;
use analyze::definition::{Field, FieldGroup};
use parse;
use std::cell::RefCell;

pub fn build<'def, 'def_ref>(
    field: &'def_ref parse::tree::VariableDeclarator<'def>,
) -> Field<'def> {
    Field {
        tpe: RefCell::new(field.tpe.clone()),
        name: field.name.clone(),
    }
}

//#[cfg(test)]
//mod tests {
//    use analyze::build::apply;
//    use analyze::definition::{Class, Field, FieldGroup, Method, Modifier, Param, Root, TypeParam};
//    use analyze::tpe::{ArrayType, ClassType, Prefix, PrimitiveType, Type};
//    use std::cell::Cell;
//    use test_common::{code, parse, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            apply(&parse(&code(
//                r#"
//class Test {
//    public int a, b[];
//}
//        "#,
//            )))
//            .0,
//            Root {
//                subpackages: vec![],
//                interfaces: vec![],
//                classes: vec![Class {
//                    import_path: "Test".to_owned(),
//                    name: &span(1, 7, "Test"),
//                    type_params: vec![],
//                    extend_opt: None,
//                    classes: vec![],
//                    interfaces: vec![],
//                    constructors: vec![],
//                    methods: vec![],
//                    field_groups: vec![FieldGroup {
//                        modifiers: vec![Modifier::Public],
//                        items: vec![
//                            Field {
//                                tpe: Type::Primitive(PrimitiveType::Int),
//                                name: &span(2, 16, "a")
//                            },
//                            Field {
//                                tpe: Type::Array(ArrayType {
//                                    elem_type: Box::new(Type::Primitive(PrimitiveType::Int))
//                                }),
//                                name: &span(2, 19, "b")
//                            }
//                        ]
//                    }],
//                    implements: vec![]
//                }],
//            }
//        )
//    }
//}
