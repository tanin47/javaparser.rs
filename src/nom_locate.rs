//! nom_locate, a special input type to locate tokens
//!
//! The source code is available on [Github](https://github.com/fflorent/nom_locate)
//!
//! ## Features
//!
//! This crate exposes two cargo feature flags, `avx-accel` and `simd-accel`.
//! These correspond to the features exposed by [bytecount](https://github.com/llogiq/bytecount).
//! Compile with SSE support (available on most modern x86_64 processors) using `simd-accel`, or
//! with AVX support (which likely requires compiling for the native target CPU) with both.
//!
//! ## How to use it
//! The explanations are given in the [README](https://github.com/fflorent/nom_locate/blob/master/README.md) of the Github repository. You may also consult the [FAQ](https://github.com/fflorent/nom_locate/blob/master/FAQ.md).
//!
//! ## Extra information
//! You can add arbitrary extra information using LocatedSpanEx.
//!
//! ``̀
//! use nom_locate::LocatedSpanEx;
//! type Span<'a> = LocatedSpan<&'a str, String>;
//!
//! let input = Span::new("Lorem ipsum \n foobar", "filename");
//! let output = parse_foobar(input);
//! let extra = output.unwrap().1.extra;
//! ``̀

use std::iter::{Enumerate, Map};
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use std::slice;
use std::slice::Iter;
use std::str::{CharIndices, Chars, FromStr};
use std::string::{String, ToString};
use std::vec::Vec;

use bytecount::{naive_num_chars, num_chars};
use memchr::Memchr;
use nom::{
    error::{ErrorKind, ParseError},
    AsBytes, Compare, CompareResult, Err, FindSubstring, FindToken, IResult, InputIter,
    InputLength, InputTake, InputTakeAtPosition, Offset, ParseTo, Slice,
};

/// A LocatedSpan is a set of meta information about the location of a token.
/// It has the same properties as a LocatedSpanEx.
pub type LocatedSpan<T> = LocatedSpanEx<T, ()>;

/// A LocatedSpanEx is a set of meta information about the location of a token, including extra
/// information.
///
/// The `LocatedSpanEx` structure can be used as an input of the nom parsers.
/// It implements all the necessary traits for `LocatedSpanEx<&str,X>` and `LocatedSpanEx<&[u8],X>`
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct LocatedSpanEx<T, X> {
    /// The line number of the fragment relatively to the input of the
    /// parser. It starts at line 1.
    pub line: usize,

    /// The col represents the position of the fragment relatively to
    /// the input of the parser. It starts at offset 1.
    pub col: usize,

    /// The fragment that is spanned.
    /// The fragment represents a part of the input of the parser.
    pub fragment: T,

    /// Extra information that can be embededd by the user.
    /// Example: the parsed file name
    pub extra: X,
}

impl<T: AsBytes> LocatedSpanEx<T, ()> {
    /// Create a span for a particular input with default `col` and
    /// `line` values and empty extra data.
    /// You can compute the column through the `get_column` or `get_utf8_column`
    /// methods.
    ///
    /// `col` starts at 1, `line` starts at 1, and `column` starts at 1.
    ///
    /// # Example of use
    ///
    /// ```
    /// extern crate javaparser;
    /// use javaparser::nom_locate::LocatedSpanEx;
    ///
    /// # fn main() {
    /// let span = LocatedSpanEx::new(b"foobar");
    ///
    /// assert_eq!(span.col,         1);
    /// assert_eq!(span.line,           1);
    /// assert_eq!(span.fragment,       &b"foobar"[..]);
    /// # }
    /// ```
    pub fn new(program: T) -> LocatedSpanEx<T, ()> {
        LocatedSpanEx {
            line: 1,
            col: 1,
            fragment: program,
            extra: (),
        }
    }
}

impl<T: AsBytes, X> LocatedSpanEx<T, X> {
    /// Create a span for a particular input with default `offset` and
    /// `line` values. You can compute the column through the `get_column` or `get_utf8_column`
    /// methods.
    ///
    /// `offset` starts at 0, `line` starts at 1, and `column` starts at 1.
    ///
    /// # Example of use
    ///
    /// ```
    /// extern crate javaparser;
    /// use javaparser::nom_locate::LocatedSpanEx;
    ///
    /// # fn main() {
    /// let span = LocatedSpanEx::new_extra(b"foobar", "extra");
    ///
    /// assert_eq!(span.col,         1);
    /// assert_eq!(span.line,           1);
    /// assert_eq!(span.fragment,       &b"foobar"[..]);
    /// assert_eq!(span.extra,          "extra");
    /// # }
    /// ```
    pub fn new_extra(program: T, extra: X) -> LocatedSpanEx<T, X> {
        LocatedSpanEx {
            line: 1,
            col: 1,
            fragment: program,
            extra: extra,
        }
    }
}

impl<T: InputLength, X> InputLength for LocatedSpanEx<T, X> {
    fn input_len(&self) -> usize {
        self.fragment.input_len()
    }
}

impl<T, X> InputTake for LocatedSpanEx<T, X>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
{
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<T, X> InputTakeAtPosition for LocatedSpanEx<T, X>
where
    T: InputTakeAtPosition + InputLength + InputIter,
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Clone,
{
    type Item = <T as InputIter>::Item;

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.split_at_position(predicate) {
            Err(Err::Incomplete(_)) => Ok(self.take_split(self.input_len())),
            res => res,
        }
    }

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.fragment.position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::Size(1))),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.fragment.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::Size(1))),
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.fragment.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.fragment.input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

/// Implement nom::InputIter for a specific fragment type
///
/// # Parameters
/// * `$fragment_type` - The LocatedSpanEx's `fragment` type
/// * `$item` - The type of the item being iterated (a reference for fragments of type `&[T]`).
/// * `$raw_item` - The raw type of the item being iterating (dereferenced type of $item for
/// `&[T]`, otherwise same as `$item`)
/// * `$iter` - The iterator type for `iter_indices()`
/// * `$iter_elem` - The iterator type for `iter_elements()`
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ```ignore
/// #[macro_use]
/// extern crate nom_locate;
///
/// impl_input_iter!(&'a str, char, char, CharIndices<'a>, Chars<'a>);
/// impl_input_iter!(&'a [u8], &'a u8, u8, Enumerate<Iter<'a, Self::RawItem>>,
///                  Iter<'a, Self::RawItem>);
/// ```
#[macro_export]
macro_rules! impl_input_iter {
    ($fragment_type:ty, $item:ty, $raw_item:ty, $iter:ty, $iter_elem:ty) => {
        impl<'a, X> InputIter for LocatedSpanEx<$fragment_type, X> {
            type Item = $item;
            type Iter = $iter;
            type IterElem = $iter_elem;
            #[inline]
            fn iter_indices(&self) -> Self::Iter {
                self.fragment.iter_indices()
            }
            #[inline]
            fn iter_elements(&self) -> Self::IterElem {
                self.fragment.iter_elements()
            }
            #[inline]
            fn position<P>(&self, predicate: P) -> Option<usize>
            where
                P: Fn(Self::Item) -> bool,
            {
                self.fragment.position(predicate)
            }
            #[inline]
            fn slice_index(&self, count: usize) -> Option<usize> {
                self.fragment.slice_index(count)
            }
        }
    };
}

impl_input_iter!(&'a str, char, char, CharIndices<'a>, Chars<'a>);
impl_input_iter!(
    &'a [u8],
    u8,
    u8,
    Enumerate<Self::IterElem>,
    Map<Iter<'a, Self::Item>, fn(&u8) -> u8>
);

/// Implement nom::Compare for a specific fragment type.
///
/// # Parameters
/// * `$fragment_type` - The LocatedSpanEx's `fragment` type
/// * `$compare_to_type` - The type to be comparable to `LocatedSpanEx<$fragment_type, X>`
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ````ignore
/// #[macro_use]
/// extern crate nom_locate;
/// impl_compare!(&'b str, &'a str);
/// impl_compare!(&'b [u8], &'a [u8]);
/// impl_compare!(&'b [u8], &'a str);
/// ````
#[macro_export]
macro_rules! impl_compare {
    ( $fragment_type:ty, $compare_to_type:ty ) => {
        impl<'a, 'b, X> Compare<$compare_to_type> for LocatedSpanEx<$fragment_type, X> {
            #[inline(always)]
            fn compare(&self, t: $compare_to_type) -> CompareResult {
                self.fragment.compare(t)
            }

            #[inline(always)]
            fn compare_no_case(&self, t: $compare_to_type) -> CompareResult {
                self.fragment.compare_no_case(t)
            }
        }
    };
}

impl_compare!(&'b str, &'a str);
impl_compare!(&'b [u8], &'a [u8]);
impl_compare!(&'b [u8], &'a str);

impl<A: Compare<B>, B, X, Y> Compare<LocatedSpanEx<B, X>> for LocatedSpanEx<A, Y> {
    #[inline(always)]
    fn compare(&self, t: LocatedSpanEx<B, X>) -> CompareResult {
        self.fragment.compare(t.fragment)
    }

    #[inline(always)]
    fn compare_no_case(&self, t: LocatedSpanEx<B, X>) -> CompareResult {
        self.fragment.compare_no_case(t.fragment)
    }
}

// TODO(future): replace impl_compare! with below default specialization?
// default impl<A: Compare<B>, B, X> Compare<B> for LocatedSpanEx<A, X> {
//     #[inline(always)]
//     fn compare(&self, t: B) -> CompareResult {
//         self.fragment.compare(t)
//     }
//
//     #[inline(always)]
//     fn compare_no_case(&self, t: B) -> CompareResult {
//         self.fragment.compare_no_case(t)
//     }
// }

/// Implement nom::Slice for a specific fragment type and range type.
///
/// **You'd probably better use impl_`slice_ranges`**,
/// unless you use a specific Range.
///
/// # Parameters
/// * `$fragment_type` - The LocatedSpanEx's `fragment` type
/// * `$range_type` - The range type to be use with `slice()`.
/// * `$can_return_self` - A bool-returning lambda telling whether we
///    can avoid creating a new `LocatedSpanEx`. If unsure, use `|_| false`.
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ````ignore
/// #[macro_use]
/// extern crate nom_locate;
///
/// #[macro_export]
/// macro_rules! impl_slice_ranges {
///     ( $fragment_type:ty ) => {
///         impl_slice_range! {$fragment_type, Range<usize>, |_| false }
///         impl_slice_range! {$fragment_type, RangeTo<usize>, |_| false }
///         impl_slice_range! {$fragment_type, RangeFrom<usize>, |range:&RangeFrom<usize>| range.start == 0}
///         impl_slice_range! {$fragment_type, RangeFull, |_| true}
///     }
/// }
///
/// ````
#[macro_export]
macro_rules! impl_slice_range {
    ( $fragment_type:ty, $range_type:ty, $can_return_self:expr ) => {
        impl<'a, X: Clone> Slice<$range_type> for LocatedSpanEx<$fragment_type, X> {
            fn slice(&self, range: $range_type) -> Self {
                if $can_return_self(&range) {
                    return self.clone();
                }
                let next_fragment = self.fragment.slice(range);
                let consumed_len = self.fragment.offset(&next_fragment);
                if consumed_len == 0 {
                    return LocatedSpanEx {
                        line: self.line,
                        col: self.col,
                        fragment: next_fragment,
                        extra: self.extra.clone(),
                    };
                }

                let consumed = self.fragment.slice(..consumed_len);
                let consumed_as_bytes = consumed.as_bytes();

                let iter = Memchr::new(b'\n', consumed_as_bytes);

                let next_col = match memchr::memrchr(b'\n', consumed_as_bytes) {
                    None => self.col + consumed_len,
                    Some(pos) => consumed_len - pos,
                };

                let number_of_lines = iter.count();
                let next_line = self.line + number_of_lines;

                LocatedSpanEx {
                    line: next_line,
                    col: next_col,
                    fragment: next_fragment,
                    extra: self.extra.clone(),
                }
            }
        }
    };
}

/// Implement nom::Slice for a specific fragment type and for these types of range:
/// * `Range<usize>`
/// * `RangeTo<usize>`
/// * `RangeFrom<usize>`
/// * `RangeFull`
///
/// # Parameters
/// * `$fragment_type` - The LocatedSpanEx's `fragment` type
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ````ignore
/// #[macro_use]
/// extern crate nom_locate;
///
/// impl_slice_ranges! {&'a str}
/// impl_slice_ranges! {&'a [u8]}
/// ````
#[macro_export]
macro_rules! impl_slice_ranges {
    ( $fragment_type:ty ) => {
        impl_slice_range! {$fragment_type, Range<usize>, |_| false }
        impl_slice_range! {$fragment_type, RangeTo<usize>, |_| false }
        impl_slice_range! {$fragment_type, RangeFrom<usize>, |range:&RangeFrom<usize>| range.start == 0}
        impl_slice_range! {$fragment_type, RangeFull, |_| true}
    }
}

impl_slice_ranges! {&'a str}
impl_slice_ranges! {&'a [u8]}

impl<Fragment: FindToken<Token>, Token, X> FindToken<Token> for LocatedSpanEx<Fragment, X> {
    fn find_token(&self, token: Token) -> bool {
        self.fragment.find_token(token)
    }
}

impl<'a, T, X> FindSubstring<&'a str> for LocatedSpanEx<T, X>
where
    T: FindSubstring<&'a str>,
{
    #[inline]
    fn find_substring(&self, substr: &'a str) -> Option<usize> {
        self.fragment.find_substring(substr)
    }
}

impl<R: FromStr, T, X> ParseTo<R> for LocatedSpanEx<T, X>
where
    T: ParseTo<R>,
{
    #[inline]
    fn parse_to(&self) -> Option<R> {
        self.fragment.parse_to()
    }
}

//impl<T, X> Offset for LocatedSpanEx<T, X> {
//    fn offset(&self, second: &Self) -> usize {
//        let fst = self.offset;
//        let snd = second.offset;
//
//        snd - fst
//    }
//}

/// Implement nom::ExtendInto for a specific fragment type.
///
/// # Parameters
/// * `$fragment_type` - The LocatedSpanEx's `fragment` type
/// * `$item` - The type of the item being iterated (a reference for fragments of type `&[T]`).
/// * `$extender` - The type of the Extended.
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ````ignore
/// #[macro_use]
/// extern crate nom_locate;
///
/// impl_extend_into!(&'a str, char, String);
/// impl_extend_into!(&'a [u8], u8, Vec<u8>);
/// ````
#[macro_export]
macro_rules! impl_extend_into {
    ($fragment_type:ty, $item:ty, $extender:ty) => {
        impl<'a, X> ExtendInto for LocatedSpanEx<$fragment_type, X> {
            type Item = $item;
            type Extender = $extender;

            #[inline]
            fn new_builder(&self) -> Self::Extender {
                self.fragment.new_builder()
            }

            #[inline]
            fn extend_into(&self, acc: &mut Self::Extender) {
                self.fragment.extend_into(acc)
            }
        }
    };
}

#[cfg(feature = "alloc")]
impl_extend_into!(&'a str, char, String);
#[cfg(feature = "alloc")]
impl_extend_into!(&'a [u8], u8, Vec<u8>);

#[macro_export]
macro_rules! impl_hex_display {
    ($fragment_type:ty) => {
        #[cfg(feature = "alloc")]
        impl<'a, X> nom::HexDisplay for LocatedSpanEx<$fragment_type, X> {
            fn to_hex(&self, chunk_size: usize) -> String {
                self.fragment.to_hex(chunk_size)
            }

            fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
                self.fragment.to_hex_from(chunk_size, from)
            }
        }
    };
}

impl_hex_display!(&'a str);
impl_hex_display!(&'a [u8]);

/// Capture the position of the current fragment

#[macro_export]
macro_rules! position {
    ($input:expr,) => {
        tag!($input, "")
    };
}

/// Capture the position of the current fragment
pub fn position<T>(s: T) -> IResult<T, T>
where
    T: InputIter + InputTake,
{
    nom::bytes::complete::take(0usize)(s)
}
