use nom::bytes::complete::{is_a, is_not, tag, take, take_till, take_until, take_while};
use nom::character::complete::{line_ending, multispace0, multispace1, newline};
use nom::character::is_space;
use nom::IResult;

use nom::sequence::preceded;
use syntax::tree::Span;
use syntax::tree::{Class, Comment};

use nom::branch::alt;
use nom::multi::{many0, separated_list};
use std::slice;

fn is_ending(c: char) -> bool {
    c == '\r' || c == '\n'
}

fn oneline_comment(input: Span) -> IResult<Span, Comment> {
    let (input, _) = multispace0(input)?;
    let (input, opening) = tag("//")(input)?;
    let (input, body) = take_till(is_ending)(input)?;

    let (input, _) = take_while(is_ending)(input)?;

    Ok((
        input,
        Comment {
            content: Span {
                line: opening.line,
                col: opening.col,
                fragment: unsafe {
                    std::str::from_utf8(slice::from_raw_parts(
                        opening.fragment.as_ptr(),
                        opening.fragment.len() + body.fragment.len(),
                    ))
                    .unwrap()
                },
                extra: (),
            },
        },
    ))
}

fn multiline_comment(input: Span) -> IResult<Span, Comment> {
    let (input, _) = multispace0(input)?;
    let (input, opening) = tag("/*")(input)?;
    let (input, body) = take_until("*/")(input)?;

    let (input, ending) = tag("*/")(input)?;

    Ok((
        input,
        Comment {
            content: Span {
                line: opening.line,
                col: opening.col,
                fragment: unsafe {
                    std::str::from_utf8(slice::from_raw_parts(
                        opening.fragment.as_ptr(),
                        opening.fragment.len() + body.fragment.len() + ending.fragment.len(),
                    ))
                    .unwrap()
                },
                extra: (),
            },
        },
    ))
}

pub fn parse_comments(input: Span) -> IResult<Span, Vec<Comment>> {
    many0(alt((oneline_comment, multiline_comment)))(input)
}

pub fn parse1(input: Span) -> IResult<Span, Vec<Comment>> {
    let (input, _) = multispace1(input)?;
    parse(input)
}

pub fn parse(input: Span) -> IResult<Span, Vec<Comment>> {
    let (input, _) = multispace0(input)?;
    let (input, comments) = parse_comments(input)?;

    let (input, _) = multispace0(input)?;
    Ok((input, comments))
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntax::tree::Comment;
    use test_common::{code, span};

    #[test]
    fn test_not_comment() {
        assert_eq!(
            parse(code("something")),
            Ok((span(1, 1, "something"), vec![]))
        )
    }

    #[test]
    fn test_oneline() {
        assert_eq!(
            parse(code("  // hello comment")),
            Ok((
                span(1, 19, ""),
                vec![Comment {
                    content: span(1, 3, "// hello comment")
                }]
            ))
        )
    }

    #[test]
    fn test_multiline() {
        assert_eq!(
            parse(code(
                r#"
/* test
* test
*/
            "#
                .trim()
            )),
            Ok((
                span(3, 3, ""),
                vec![Comment {
                    content: span(1, 1, "/* test\n* test\n*/")
                }]
            ))
        )
    }
}
