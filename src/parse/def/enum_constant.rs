use parse::combinator::{identifier, opt};
use parse::def::{annotateds, class_body};
use parse::expr::atom::method_call;
use parse::id_gen::IdGen;
use parse::tree::EnumConstant;
use parse::{ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, EnumConstant<'def>> {
    let (input, annotateds) = annotateds::parse(input, id_gen)?;
    let (input, name) = identifier(input)?;

    let (input, args_opt) = opt(|i| method_call::parse_args(i, id_gen))(input)?;

    let (input, body_opt) = opt(|i| class_body::parse(i, id_gen))(input)?;

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

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        Annotated, Block, ClassBody, ClassBodyItem, ClassType, EnumConstant, Expr, Int,
//        MarkerAnnotated, Method, Type, Void,
//    };
//    use parse::Tokens;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//@Anno FIRST
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                EnumConstant {
//                    annotateds: vec![Annotated::Marker(MarkerAnnotated {
//                        class: ClassType {
//                            prefix_opt: None,
//                            name: span(1, 2, "Anno"),
//                            type_args_opt: None,
//                            def_opt: None
//                        }
//                    })],
//                    name: span(1, 7, "FIRST"),
//                    args_opt: None,
//                    body_opt: None
//                }
//            ))
//        );
//    }
//
//    #[test]
//    fn test_with_args_and_body() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//FIRST(1) {
//  void method() {}
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                EnumConstant {
//                    annotateds: vec![],
//                    name: span(1, 1, "FIRST"),
//                    args_opt: Some(vec![Expr::Int(Int {
//                        value: span(1, 7, "1")
//                    })]),
//                    body_opt: Some(ClassBody {
//                        items: vec![ClassBodyItem::Method(Method {
//                            modifiers: vec![],
//                            return_type: Type::Void(Void {
//                                span: span(2, 3, "void")
//                            }),
//                            name: span(2, 8, "method"),
//                            type_params: vec![],
//                            params: vec![],
//                            throws: vec![],
//                            block_opt: Some(Block { stmts: vec![] }),
//                        })]
//                    })
//                }
//            ))
//        );
//    }
//}
