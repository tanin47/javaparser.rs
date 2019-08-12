use parse::combinator::{many0, symbol};
use parse::tree::{Block, Statement};
use parse::{statement, ParseResult, Tokens};

pub fn parse_block_or_single_statement(input: Tokens) -> ParseResult<Block> {
    if let Ok(ok) = statement::block::parse_block(input) {
        Ok(ok)
    } else {
        let (input, stmt) = statement::parse(input)?;
        Ok((input, Block { stmts: vec![stmt] }))
    }
}

pub fn parse_block(input: Tokens) -> ParseResult<Block> {
    let (input, _) = symbol("{")(input)?;
    let (input, stmts) = many0(statement::parse)(input)?;
    let (input, _) = symbol("}")(input)?;

    Ok((input, Block { stmts }))
}

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, block) = parse_block(input)?;
    Ok((input, Statement::Block(block)))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Block, Expr, Int, ReturnStmt, Statement};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_method() {
        assert_eq!(
            parse(&code(
                r#"
{
    return 1;
}
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Block(Block {
                    stmts: vec![Statement::Return(ReturnStmt {
                        expr_opt: Some(Expr::Int(Int {
                            value: span(2, 12, "1")
                        }))
                    })],
                })
            ))
        );
    }
}
