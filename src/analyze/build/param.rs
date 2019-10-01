use analyze::definition::Param;
use parse;
use std::cell::RefCell;

pub fn build<'def, 'def_ref>(param: &'def_ref parse::tree::Param<'def>) -> Param<'def> {
    Param {
        tpe: RefCell::new(param.tpe.clone()),
        name: param.name.clone(),
    }
}
