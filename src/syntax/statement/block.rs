use nom::bytes::complete::{tag, take, take_till, take_while};
use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::multi::many0;
use syntax::tree::{Block, Span, Statement};
use syntax::tree::{Class, Method};
use syntax::{comment, statement};

pub fn parse_block_or_single_statement(input: Span) -> IResult<Span, Block> {
    match statement::block::parse_block(input) {
        Ok(result) => Ok(result),
        Err(_) => {
            let (input, stmt) = statement::parse(input)?;
            Ok((input, Block { stmts: vec![stmt] }))
        }
    }
}

pub fn parse_block(input: Span) -> IResult<Span, Block> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("{")(input)?;

    let (input, stmts) = many0(statement::parse)(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("}")(input)?;

    Ok((input, Block { stmts }))
}

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, block) = parse_block(input)?;
    Ok((input, Statement::Block(block)))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{Block, Expr, Int, Method, ReturnStmt, Statement};
    use test_common::{code, span};

    #[test]
    fn test_method() {
        assert_eq!(
            parse(code(
                r#"
{
    return 1;
}
            "#
                .trim()
            )),
            Ok((
                span(3, 2, ""),
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
