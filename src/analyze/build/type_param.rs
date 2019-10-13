use analyze::definition::TypeParam;
use parse;
use parse::tree::TypeParamExtend;
use std::cell::RefCell;

pub fn build<'def, 'def_ref>(
    type_param: &'def_ref parse::tree::TypeParam<'def>,
) -> TypeParam<'def> {
    let mut extends = vec![];

    for t in &type_param.extends {
        extends.push(t.clone());
    }

    TypeParam {
        name: type_param.name.fragment.to_owned(),
        extends: RefCell::new(extends),
        span_opt: Some(type_param.name),
        id: type_param.id.to_owned(),
    }
}
