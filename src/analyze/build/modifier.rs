use analyze::definition::Modifier;
use parse;
use parse::tree::Keyword;
use std::collections::HashSet;

pub fn build<'def, 'def_ref>(
    modifiers: &'def_ref [parse::tree::Modifier<'def>],
) -> HashSet<Modifier> {
    let mut items = HashSet::new();

    for modi in modifiers {
        match modi {
            parse::tree::Modifier::Keyword(k) => {
                items.insert(build_keyword(k));
            }
            _ => (),
        };
    }

    items
}

fn build_keyword<'def, 'def_ref>(keyword: &'def_ref Keyword<'def>) -> Modifier {
    match keyword.name.fragment {
        "abstract" => Modifier::Abstract,
        "default" => Modifier::Default,
        "final" => Modifier::Final,
        "native" => Modifier::Native,
        "private" => Modifier::Private,
        "protected" => Modifier::Protected,
        "public" => Modifier::Public,
        "static" => Modifier::Static,
        "strictfp" => Modifier::Strictfp,
        "synchronized" => Modifier::Synchronized,
        "transient" => Modifier::Transient,
        "volatile" => Modifier::Volatile,
        _ => panic!(),
    }
}
