use analyze::build::tpe;
use analyze::referenceable::TypeParam;
use parse;

pub fn build<'a>(type_param: &'a parse::tree::TypeParam<'a>) -> TypeParam<'a> {
    let mut extends = vec![];

    for t in &type_param.extends {
        extends.push(tpe::build_class(t))
    }

    TypeParam {
        name: &type_param.name,
        extends,
    }
}
