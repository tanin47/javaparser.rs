use analyze::definition::Class;
use std::cell::RefCell;
use test_common::span;
use tokenize::span::Span;

pub fn mock_class<'a>(name: &'a Span<'a>) -> Class<'a> {
    Class {
        import_path: name.fragment.to_owned(),
        name,
        type_params: vec![],
        extend_opt: RefCell::new(None),
        implements: vec![],
        constructors: vec![],
        methods: vec![],
        field_groups: vec![],
        decls: vec![],
    }
}
