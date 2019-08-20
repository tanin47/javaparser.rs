use analyze::definition::Constructor;
use parse;

pub fn build<'a>(constructor: &'a parse::tree::Constructor<'a>) -> Constructor<'a> {
    Constructor {
        name: &constructor.name,
    }
}
