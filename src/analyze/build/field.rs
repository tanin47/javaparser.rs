use analyze::build::{modifier, tpe};
use analyze::referenceable::{Field, FieldGroup};
use parse;

pub fn build<'a>(field: &'a parse::tree::VariableDeclarator<'a>) -> Field<'a> {
    Field {
        tpe: tpe::build(&field.tpe),
        name: &field.name,
    }
}

#[cfg(test)]
mod tests {
    use analyze::build::apply;
    use analyze::referenceable::{
        Class, Field, FieldGroup, Method, Modifier, Param, Root, TypeParam,
    };
    use analyze::tpe::{ArrayType, ClassType, Prefix, PrimitiveType, Type};
    use std::cell::Cell;
    use test_common::{code, parse, span};

    #[test]
    fn test() {
        assert_eq!(
            apply(&parse(&code(
                r#"
class Test {
    public int a, b[];
}
        "#,
            ))),
            Root {
                subpackages: vec![],
                interfaces: vec![],
                classes: vec![Class {
                    import_path: "Test".to_owned(),
                    name: &span(1, 7, "Test"),
                    type_params: vec![],
                    extend_opt: None,
                    classes: vec![],
                    interfaces: vec![],
                    constructors: vec![],
                    methods: vec![],
                    field_groups: vec![FieldGroup {
                        modifiers: vec![Modifier::Public],
                        items: vec![
                            Field {
                                tpe: Type::Primitive(PrimitiveType::Int),
                                name: &span(2, 16, "a")
                            },
                            Field {
                                tpe: Type::Array(ArrayType {
                                    elem_type: Box::new(Type::Primitive(PrimitiveType::Int))
                                }),
                                name: &span(2, 19, "b")
                            }
                        ]
                    }],
                    implements: vec![]
                }],
            }
        )
    }
}
