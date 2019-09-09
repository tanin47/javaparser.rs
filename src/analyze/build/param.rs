use analyze::build::tpe;
use analyze::definition::Param;
use parse;
use std::cell::RefCell;

pub fn build<'a>(param: &'a parse::tree::Param<'a>) -> Param<'a> {
    Param {
        tpe: RefCell::new(tpe::build(&param.tpe)),
        name: &param.name,
    }
}
