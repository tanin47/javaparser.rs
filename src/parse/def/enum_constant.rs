use parse::combinator::{identifier, opt};
use parse::def::{annotateds, class_body};
use parse::expr::atom::invocation;
use parse::tree::EnumConstant;
use parse::{ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<EnumConstant> {
    let (input, annotateds) = annotateds::parse(input)?;
    let (input, name) = identifier(input)?;

    let (input, args_opt) = opt(invocation::parse_args)(input)?;

    let (input, body_opt) = opt(class_body::parse)(input)?;

    Ok((
        input,
        EnumConstant {
            annotateds,
            name,
            args_opt,
            body_opt,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Annotated, Block, ClassBody, ClassBodyItem, ClassType, EnumConstant, Expr, Int,
        MarkerAnnotated, Method, Type, Void,
    };
    use parse::Tokens;
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(&code(
                r#"
@Anno FIRST
            "#
            )),
            Ok((
                &[] as Tokens,
                EnumConstant {
                    annotateds: vec![Annotated::Marker(MarkerAnnotated {
                        class: ClassType {
                            prefix_opt: None,
                            name: span(1, 2, "Anno"),
                            type_args_opt: None
                        }
                    })],
                    name: span(1, 7, "FIRST"),
                    args_opt: None,
                    body_opt: None
                }
            ))
        );
    }

    #[test]
    fn test_with_args_and_body() {
        assert_eq!(
            parse(&code(
                r#"
FIRST(1) {
  void method() {}
}
            "#
            )),
            Ok((
                &[] as Tokens,
                EnumConstant {
                    annotateds: vec![],
                    name: span(1, 1, "FIRST"),
                    args_opt: Some(vec![Expr::Int(Int {
                        value: span(1, 7, "1")
                    })]),
                    body_opt: Some(ClassBody {
                        items: vec![ClassBodyItem::Method(Method {
                            modifiers: vec![],
                            return_type: Type::Void(Void {
                                span: span(2, 3, "void")
                            }),
                            name: span(2, 8, "method"),
                            type_params: vec![],
                            params: vec![],
                            throws: vec![],
                            block_opt: Some(Block { stmts: vec![] }),
                        })]
                    })
                }
            ))
        );
    }
}
