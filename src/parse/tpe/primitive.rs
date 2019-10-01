use parse::combinator::{any_keyword, identifier};
use parse::tpe::array;
use parse::tree::{PrimitiveType, PrimitiveTypeType, Type};
use parse::{ParseResult, Tokens};

pub fn valid(s: &str) -> bool {
    s == "boolean"
        || s == "byte"
        || s == "short"
        || s == "int"
        || s == "long"
        || s == "float"
        || s == "double"
        || s == "char"
}

pub fn build_type_type(name: &str) -> Result<PrimitiveTypeType, ()> {
    match name {
        "boolean" => Ok(PrimitiveTypeType::Boolean),
        "byte" => Ok(PrimitiveTypeType::Byte),
        "short" => Ok(PrimitiveTypeType::Short),
        "int" => Ok(PrimitiveTypeType::Int),
        "long" => Ok(PrimitiveTypeType::Long),
        "float" => Ok(PrimitiveTypeType::Float),
        "double" => Ok(PrimitiveTypeType::Double),
        "char" => Ok(PrimitiveTypeType::Char),
        _ => Err(()),
    }
}

pub fn parse_no_array(original: Tokens) -> ParseResult<PrimitiveType> {
    let (input, name) = any_keyword(original)?;
    if let Ok(tpe) = build_type_type(name.fragment) {
        Ok((input, PrimitiveType { name, tpe }))
    } else {
        Err(original)
    }
}

pub fn parse(input: Tokens) -> ParseResult<Type> {
    let (input, tpe) = parse_no_array(input)?;
    array::parse_tail(input, Type::Primitive(tpe))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{ArrayType, PrimitiveType, PrimitiveTypeType, Type};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_long() {
        assert_eq!(
            parse(&code(
                r#"
long
            "#
            )),
            Ok((
                &[] as Tokens,
                Type::Primitive(PrimitiveType {
                    name: span(1, 1, "long"),
                    tpe: PrimitiveTypeType::Long
                })
            ))
        );
    }

    #[test]
    fn test_array_2d() {
        assert_eq!(
            parse(&code(
                r#"
long[][]
            "#
            )),
            Ok((
                &[] as Tokens,
                Type::Array(ArrayType {
                    tpe: Box::new(Type::Array(ArrayType {
                        tpe: Box::new(Type::Primitive(PrimitiveType {
                            name: span(1, 1, "long"),
                            tpe: PrimitiveTypeType::Long
                        })),
                        size_opt: None
                    })),
                    size_opt: None
                })
            ))
        );
    }
}
