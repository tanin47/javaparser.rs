use analyze::definition::{Class, FieldDef, FieldGroup, MethodDef, Modifier, TypeParam};
use parse::tree::{
    ArrayType, ClassType, ParameterizedType, PrimitiveType, PrimitiveTypeType, Type, TypeArg,
    NATIVE_ARRAY_CLASS_NAME,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::pin::Pin;

pub fn apply<'def>() -> Class<'def> {
    Class {
        id: format!("class_{}", NATIVE_ARRAY_CLASS_NAME),
        name: NATIVE_ARRAY_CLASS_NAME.to_string(),
        import_path: NATIVE_ARRAY_CLASS_NAME.to_string(),
        span_opt: None,
        type_params: vec![TypeParam {
            name: "T".to_owned(),
            extends: RefCell::new(vec![]),
            span_opt: None,
            id: format!("{}_TypeParam_T", NATIVE_ARRAY_CLASS_NAME),
        }],
        extend_opt: RefCell::new(None),
        //        extend_opt: RefCell::new(Some(ClassType {
        //            prefix_opt: None,
        //            name: "Object".to_string(),
        //            span_opt: None,
        //            type_args_opt: None,
        //            def_opt: None,
        //        })),
        implements: vec![],
        constructors: vec![],
        methods: vec![MethodDef {
            modifiers: HashSet::from_iter(vec![Modifier::Public]),
            type_params: vec![],
            return_type: RefCell::new(Type::Array(ArrayType {
                // ClassType will be converted to ParameterizedType in analyze::resolve::assign_type.
                tpe: Box::new(Type::Class(ClassType {
                    prefix_opt: None,
                    name: "T".to_string(),
                    span_opt: None,
                    type_args_opt: None,
                    def_opt: None,
                })),
                size_opt: None,
                underlying: ClassType {
                    prefix_opt: None,
                    name: NATIVE_ARRAY_CLASS_NAME.to_string(),
                    span_opt: None,
                    // ClassType will be converted to ParameterizedType in analyze::resolve::assign_type.
                    type_args_opt: Some(vec![TypeArg::Class(ClassType {
                        prefix_opt: None,
                        name: "T".to_string(),
                        span_opt: None,
                        type_args_opt: None,
                        def_opt: None,
                    })]),
                    def_opt: None,
                },
            })),
            name: "clone".to_owned(),
            params: vec![],
            id: format!("{}_method_clone", NATIVE_ARRAY_CLASS_NAME),
            span_opt: None,
        }],
        field_groups: vec![FieldGroup {
            modifiers: HashSet::from_iter(vec![Modifier::Public]),
            items: vec![FieldDef {
                tpe: RefCell::new(Type::Primitive(PrimitiveType {
                    span_opt: None,
                    tpe: PrimitiveTypeType::Int,
                })),
                name: "length".to_owned(),
                span_opt: None,
                id: format!("{}_field_length", NATIVE_ARRAY_CLASS_NAME),
            }],
            parse_opt: None,
        }],
        decls: vec![],
    }
}
