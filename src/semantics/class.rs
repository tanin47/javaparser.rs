use analyze::resolve::scope::Scope;
use parse;
use parse::tree::Class;

pub fn apply<'def, 'def_ref>(unit: &parse::tree::Class<'def>, scope: &mut Scope<'def, 'def_ref>) {}
