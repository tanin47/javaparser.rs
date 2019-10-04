use parse::combinator::{many0, symbol};
use parse::tree::{Block, Statement};
use parse::{statement, ParseResult, Tokens};

pub fn parse_block_or_single_statement<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Block<'def>> {
    if let Ok(ok) = statement::block::parse_block(input) {
        Ok(ok)
    } else {
        let (input, stmt) = statement::parse(input)?;
        Ok((input, Block { stmts: vec![stmt] }))
    }
}

pub fn parse_block<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Block<'def>> {
    let (input, _) = symbol('{')(input)?;
    let (input, stmts) = many0(statement::parse)(input)?;
    let (input, _) = symbol('}')(input)?;

    Ok((input, Block { stmts }))
}

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, block) = parse_block(input)?;
    Ok((input, Statement::Block(block)))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Block, Expr, Int, ReturnStmt, Statement};
    use parse::Tokens;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_method() {
        assert_eq!(
            parse(&generate_tokens(
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
