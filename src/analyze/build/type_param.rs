use analyze::build::tpe;
use analyze::definition::{TypeParam, TypeParamExtend};
use parse;
use std::cell::RefCell;

pub fn build<'a>(type_param: &'a parse::tree::TypeParam<'a>) -> TypeParam<'a> {
    let mut extends = vec![];

    for t in &type_param.extends {
        extends.push(TypeParamExtend::Class(tpe::build_class(t)))
    }

    TypeParam {
        name: &type_param.name,
        extends: RefCell::new(extends),
    }
}
