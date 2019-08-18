use analyze::referenceable::Method;
use parse;

pub fn build<'a>(method: &'a parse::tree::Method<'a>) -> Method<'a> {
    Method { name: &method.name }
}
