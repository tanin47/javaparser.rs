use tokenize::span::CharAt;
use tokenize::span::Span;

pub fn take(size: usize, input: Span) -> (Span, Span) {
    let mut line = input.line;
    let mut col = input.col;

    for index in 0..size {
        let c = input.fragment.char_at(index);
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    let (before, after) = input.fragment.split_at(size);

    (
        Span {
            line: input.line,
            col: input.col,
            fragment: before,
        },
        Span {
            line,
            col,
            fragment: after,
        },
    )
}

pub fn take_while<F>(cond: F, input: Span) -> (Span, Span)
where
    F: Fn(usize, &str) -> bool,
{
    let mut line = input.line;
    let mut col = input.col;
    let mut size = 0;

    for index in 0..input.fragment.len() {
        let c = input.fragment.char_at(index);
        if cond(index, input.fragment) {
            size += 1;
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1
            }
        } else {
            break;
        }
    }

    let (before, after) = input.fragment.split_at(size);

    (
        Span {
            line: input.line,
            col: input.col,
            fragment: before,
        },
        Span {
            line,
            col,
            fragment: after,
        },
    )
}
