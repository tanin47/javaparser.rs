use analyze::definition::Class;
use test_common::span;
use tokenize::span::Span;

pub fn mock_class<'a>(name: &'a Span<'a>) -> Class<'a> {
    Class {
        import_path: name.fragment.to_owned(),
        name,
        type_params: vec![],
        extend_opt: None,
        implements: vec![],
        constructors: vec![],
        methods: vec![],
        field_groups: vec![],
        classes: vec![],
        interfaces: vec![],
    }
}
