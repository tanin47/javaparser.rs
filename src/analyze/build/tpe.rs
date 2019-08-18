use analyze::tpe::{ArrayType, ClassType, Prefix, PrimitiveType, Type};
use parse;
use std::cell::Cell;

pub fn build<'a>(tpe: &'a parse::tree::Type<'a>) -> Type<'a> {
    match tpe {
        parse::tree::Type::Primitive(p) => Type::Primitive(build_primitive(p)),
        parse::tree::Type::Array(a) => Type::Array(build_array(a)),
        parse::tree::Type::Class(c) => Type::Class(build_class(c)),
        parse::tree::Type::Void(v) => Type::Void,
        _ => panic!(),
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

fn build_class<'a>(class: &'a parse::tree::ClassType<'a>) -> ClassType<'a> {
    ClassType {
        prefix_opt: match &class.prefix_opt {
            Some(p) => Some(Box::new(Prefix::Class(build_class(p)))),
            None => None,
        },
        name: class.name.fragment,
        def_opt: Cell::new(None),
    }
}
