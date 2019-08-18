use analyze::build::tpe;
use analyze::referenceable::Param;
use parse;

pub fn build<'a>(param: &'a parse::tree::Param<'a>) -> Param<'a> {
    Param {
        tpe: tpe::build(&param.tpe),
        name: &param.name,
    }
}
