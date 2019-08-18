use analyze::build::{modifier, param, tpe};
use analyze::referenceable::Method;
use parse;

pub fn build<'a>(method: &'a parse::tree::Method<'a>) -> Method<'a> {
    let mut params = vec![];

    for p in &method.params {
        params.push(param::build(p));
    }

    Method {
        modifiers: modifier::build(&method.modifiers),
        return_type: tpe::build(&method.return_type),
        name: &method.name,
        params,
    }
}

#[cfg(test)]
mod tests {
    use analyze::build::apply;
    use analyze::referenceable::{Class, Method, Modifier, Param, Root};
    use analyze::tpe::{ClassType, Prefix, PrimitiveType, Type};
    use std::cell::Cell;
    use test_common::{code, parse, span};

    #[test]
    fn test() {
        assert_eq!(
            apply(&parse(&code(
                r#"
class Test {
    public void method(int a) {}
    boolean method2() {}
    Parent.Test method3() {}
}
        "#,
            ))),
            Root {
                subpackages: vec![],
                interfaces: vec![],
                classes: vec![Class {
                    import_path: "Test".to_owned(),
                    name: &span(1, 7, "Test"),
                    classes: vec![],
                    interfaces: vec![],
                    constructors: vec![],
                    methods: vec![
                        Method {
                            modifiers: vec![Modifier::Public],
                            return_type: Type::Void,
                            name: &span(2, 17, "method"),
                            params: vec![Param {
                                tpe: Type::Primitive(PrimitiveType::Int),
                                name: &span(2, 28, "a")
                            }]
                        },
                        Method {
                            modifiers: vec![],
                            return_type: Type::Primitive(PrimitiveType::Boolean),
                            name: &span(3, 13, "method2"),
                            params: vec![]
                        },
                        Method {
                            modifiers: vec![],
                            return_type: Type::Class(ClassType {
                                prefix_opt: Some(Box::new(Prefix::Class(ClassType {
                                    prefix_opt: None,
                                    name: "Parent",
                                    def_opt: Cell::new(None)
                                }))),
                                name: "Test",
                                def_opt: Cell::new(None)
                            }),
                            name: &span(4, 17, "method3"),
                            params: vec![]
                        },
                    ],
                    field_groups: vec![]
                }],
            }
        )
    }
}
