use std::slice;
use tokenize::apply;
use tokenize::span::CharAt;
use tokenize::span::Span;

pub fn take_bit(input: Span) -> (Span, Span) {
    take_while(
        |index, s| {
            let c = s.char_at(index);
            c == '0' || c == '1' || c == '_' && index > 0
        },
        input,
    )
}

pub fn take_hex_number(input: Span) -> (Span, Span) {
    take_while(
        |index, s| {
            let c = s.char_at(index);
            c >= '0' && c <= '9'
                || c >= 'a' && c <= 'f'
                || c >= 'A' && c <= 'F'
                || c == '_' && index > 0
        },
        input,
    )
}

pub fn take_number(original: Span) -> (Span, Span) {
    take_while(
        |index, s| {
            let c = s.char_at(index);
            (c >= '0' && c <= '9') || (c == '_' && index > 0)
        },
        original,
    )
}

pub fn concat<'def, 'def_ref>(spans: &'def_ref [Span<'def>]) -> Span<'def> {
    let mut total_len = 0;

    for s in spans {
        assert_eq!(
            s.line, spans[0].line,
            "One of the spans isn't on the same line"
        );
        assert_eq!(
            s.col,
            spans[0].col + total_len,
            "One of the spans isn't adjacent to its previous span."
        );
        total_len += s.fragment.len();
    }

    Span {
        line: spans[0].line,
        col: spans[0].col,
        fragment: unsafe {
            std::str::from_utf8_unchecked(slice::from_raw_parts(
                spans[0].fragment.as_ptr(),
                total_len,
            ))
        },
        file: spans[0].file,
    }
}

pub fn take_one_if_case_insensitive<'a>(c: &'a str, input: Span<'a>) -> (Span<'a>, Span<'a>) {
    if c.to_ascii_uppercase()
        .contains(input.fragment.char_at(0).to_ascii_uppercase())
    {
        take(1, input)
    } else {
        (
            Span {
                line: input.line,
                col: input.col,
                fragment: "",
                file: input.file,
            },
            input,
        )
    }
}

pub fn take(size: usize, input: Span) -> (Span, Span) {
    if size > input.fragment.len() {
        return (
            Span {
                line: input.line,
                col: input.col,
                fragment: "",
                file: input.file,
            },
            input,
        );
    }

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
            file: input.file,
        },
        Span {
            line,
            col,
            fragment: after,
            file: input.file,
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
            file: input.file,
        },
        Span {
            line,
            col,
            fragment: after,
            file: input.file,
        },
    )
}
