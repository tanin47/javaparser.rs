use analyze::tpe::{
    ArrayType, ClassType, Prefix, PrimitiveType, ReferenceType, Type, TypeArg, WildcardType,
};
use parse;
use std::cell::{Cell, RefCell};

pub fn build<'a>(tpe: &'a parse::tree::Type<'a>) -> Type<'a> {
    match tpe {
        parse::tree::Type::Primitive(p) => Type::Primitive(build_primitive(p)),
        parse::tree::Type::Array(a) => Type::Array(build_array(a)),
        parse::tree::Type::Class(c) => Type::Class(build_class(c)),
        parse::tree::Type::Void(v) => Type::Void,
        _ => panic!(),
    }
}

pub fn build_reference<'a>(tpe: &'a parse::tree::ReferenceType<'a>) -> ReferenceType<'a> {
    match tpe {
        parse::tree::ReferenceType::Array(a) => ReferenceType::Array(build_array(a)),
        parse::tree::ReferenceType::Class(c) => ReferenceType::Class(build_class(c)),
    }
}

fn build_primitive<'a>(primitive: &'a parse::tree::PrimitiveType<'a>) -> PrimitiveType {
    match primitive.name.fragment {
        "boolean" => PrimitiveType::Boolean,
        "byte" => PrimitiveType::Byte,
        "short" => PrimitiveType::Short,
        "int" => PrimitiveType::Int,
        "long" => PrimitiveType::Long,
        "float" => PrimitiveType::Float,
        "double" => PrimitiveType::Double,
        "char" => PrimitiveType::Char,
        _ => panic!(),
    }
}

fn build_array<'a>(array: &'a parse::tree::ArrayType<'a>) -> ArrayType<'a> {
    ArrayType {
        elem_type: Box::new(build(&array.tpe)),
    }
}

pub fn build_class<'a>(class: &'a parse::tree::ClassType<'a>) -> ClassType<'a> {
    let mut type_args = vec![];

    if let Some(tas) = &class.type_args_opt {
        for t in tas {
            type_args.push(build_type_arg(t));
        }
    }

    ClassType {
        prefix_opt: match &class.prefix_opt {
            Some(p) => Some(Box::new(Prefix::Class(build_class(p)))),
            None => None,
        },
        name: class.name.fragment,
        type_args,
        def_opt: Cell::new(None),
    }
}

fn build_wildcard<'a>(wildcard: &'a parse::tree::WildcardType<'a>) -> WildcardType<'a> {
    let mut extends = vec![];

    for e in &wildcard.extends {
        extends.push(build_reference(e));
    }

    WildcardType {
        name: &wildcard.name,
        super_opt: match &wildcard.super_opt {
            Some(r) => Some(Box::new(build_reference(r))),
            None => None,
        },
        extends,
    }
}

fn build_type_arg<'a>(type_arg: &'a parse::tree::TypeArg<'a>) -> TypeArg<'a> {
    match type_arg {
        parse::tree::TypeArg::Array(a) => TypeArg::Array(build_array(a)),
        parse::tree::TypeArg::Class(c) => TypeArg::Class(build_class(c)),
        parse::tree::TypeArg::Wildcard(w) => TypeArg::Wildcard(build_wildcard(w)),
    }
}
