use parse;
use parse::tree::{
    ArrayType, ClassType, EnclosingType, PrimitiveType, ReferenceType, Type, TypeArg, WildcardType,
};
use std::cell::{Cell, RefCell};

pub fn build<'def, 'def_ref>(tpe: &'def_ref parse::tree::Type<'def>) -> Type<'def> {
    match tpe {
        parse::tree::Type::Primitive(p) => Type::Primitive(build_primitive(p)),
        parse::tree::Type::Array(a) => Type::Array(build_array(a)),
        parse::tree::Type::Class(c) => Type::Class(build_class(c)),
        parse::tree::Type::Void(v) => Type::Void,
        _ => panic!(),
    }
}

fn build_primitive<'def, 'def_ref>(
    primitive: &'def_ref parse::tree::PrimitiveType<'def>,
) -> PrimitiveType<'def> {
    let tpe = match primitive.name.fragment {
        "boolean" => PrimitiveType::Boolean,
        "byte" => PrimitiveType::Byte,
        "short" => PrimitiveType::Short,
        "int" => PrimitiveType::Int,
        "long" => PrimitiveType::Long,
        "float" => PrimitiveType::Float,
        "double" => PrimitiveType::Double,
        "char" => PrimitiveType::Char,
        _ => panic!(),
    };

    PrimitiveType {
        name: primitive.name,
        tpe: tpe,
    }
}

fn build_array<'def, 'def_ref>(array: &'def_ref parse::tree::ArrayType<'def>) -> ArrayType<'def> {
    ArrayType {
        tpe: Box::new(build(&array.tpe)),
        size_opt: None,
    }
}

pub fn build_class<'def, 'def_ref>(
    class: &'def_ref parse::tree::ClassType<'def>,
) -> ClassType<'def> {
    let mut type_args_opt: Option<Vec<TypeArg>> = None;

    if let Some(tas) = &class.type_args_opt {
        let mut type_args = vec![];
        for t in tas {
            type_args.push(build_type_arg(t));
        }
        type_args_opt = Some(type_args)
    }

    ClassType {
        prefix_opt: match &class.prefix_opt {
            Some(p) => Some(p.clone()),
            None => None,
        },
        name: class.name,
        type_args_opt,
        def_opt: Cell::new(None),
    }
}

fn build_wildcard<'def, 'def_ref>(
    wildcard: &'def_ref parse::tree::WildcardType<'def>,
) -> WildcardType<'def> {
    let mut extends = vec![];

    for e in &wildcard.extends {
        extends.push(build_reference(e));
    }

    WildcardType {
        name: wildcard.name.clone(),
        super_opt: match &wildcard.super_opt {
            Some(r) => Some(Box::new(build_reference(r))),
            None => None,
        },
        extends,
    }
}

fn build_type_arg<'def, 'def_ref>(type_arg: &'def_ref parse::tree::TypeArg<'def>) -> TypeArg<'def> {
    match type_arg {
        parse::tree::TypeArg::Array(a) => TypeArg::Array(build_array(a)),
        parse::tree::TypeArg::Class(c) => TypeArg::Class(build_class(c)),
        parse::tree::TypeArg::Wildcard(w) => TypeArg::Wildcard(build_wildcard(w)),
    }
}
