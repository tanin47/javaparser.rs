use analyze::definition::Constructor;
use parse;

pub fn build<'def, 'def_ref>(
    constructor: &'def_ref parse::tree::Constructor<'def>,
) -> Constructor<'def> {
    Constructor {
        name: constructor.name.clone(),
    }
}
