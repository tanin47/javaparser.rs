use parse::combinator::symbol;
use parse::tree::{ArrayType, ClassType, Type, TypeArg, NATIVE_ARRAY_CLASS_NAME};
use parse::{ParseResult, Tokens};

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    tpe: Type<'def>,
) -> ParseResult<'def, 'r, Type<'def>> {
    if let Ok((input, _)) = symbol('[')(input) {
        let (input, end) = symbol(']')(input)?;
        parse_tail(
            input,
            Type::Array(ArrayType {
                size_opt: None,
                underlying: ClassType {
                    prefix_opt: None,
                    name: NATIVE_ARRAY_CLASS_NAME.to_owned(),
                    span_opt: None,
                    type_args_opt: Some(vec![tpe.clone().to_type_arg()]),
                    def_opt: None,
                },
                tpe: Box::new(tpe),
            }),
        )
    } else {
        Ok((input, tpe))
    }
}
