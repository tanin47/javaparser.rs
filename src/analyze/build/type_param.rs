use analyze::definition::{TypeParam, TypeParamExtend};
use parse;
use std::cell::RefCell;

pub fn build<'def, 'def_ref>(
    type_param: &'def_ref parse::tree::TypeParam<'def>,
) -> TypeParam<'def> {
    let mut extends = vec![];

    for t in &type_param.extends {
        extends.push(TypeParamExtend::Class(t.clone()))
    }

    TypeParam {
        name: type_param.name.fragment.to_owned(),
        extends: RefCell::new(extends),
        parse_opt: Some(type_param),
    }
}
