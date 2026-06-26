#[cfg(test)]
mod tests;

use std::time::Duration;

use nom::{
    AsChar, Finish, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till, take_while},
    character::complete::{char, digit1, line_ending, space0},
    combinator::{eof, map, map_res, recognize},
    multi::{many0, many1},
    number::complete::float,
    sequence::{delimited, terminated},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct IDTag<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TimestampedSegment<'a> {
    pub timestamp: Duration,
    pub content: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TimestampedTag<'a> {
    pub timestamp: Duration,
    pub segments: Vec<TimestampedSegment<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Line<'a> {
    ID(IDTag<'a>),
    Tag(TimestampedTag<'a>),
    Comment(&'a str),
}

pub(crate) fn till_newline(i: &str) -> IResult<&str, &str> {
    take_while(|c: char| !c.is_newline())(i)
}

pub(crate) fn newline_or_end(i: &str) -> IResult<&str, &str> {
    alt((recognize(many1(line_ending)), eof)).parse(i)
}

fn till_a2_tag(i: &str) -> IResult<&str, &str> {
    take_till(|c: char| c == '<' || c == '\n')(i)
}

pub(crate) fn timestamp(i: &str) -> IResult<&str, Duration> {
    let (i, (minutes, _, seconds)) = (float, char(':'), float).parse(i)?;
    Ok((i, Duration::from_secs_f32(seconds + minutes * 60.0)))
}

fn unsigned_int(i: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse).parse(i)
}

pub(crate) fn offset(i: &str) -> IResult<&str, i64> {
    terminated(
        alt((
            map((char('+'), unsigned_int), |(_, num)| num as i64),
            map((char('-'), unsigned_int), |(_, num)| -(num as i64)),
        )),
        eof,
    )
    .parse(i)
}

fn standard_timestamp(i: &str) -> IResult<&str, Duration> {
    delimited(char('['), timestamp, char(']')).parse(i)
}

fn a2_timestamp(i: &str) -> IResult<&str, Duration> {
    delimited(char('<'), timestamp, char('>')).parse(i)
}

fn a2_tag<'a>(i: &'a str) -> IResult<&'a str, TimestampedSegment<'a>> {
    map(
        (a2_timestamp, space0, till_a2_tag),
        |(timestamp, _, content)| TimestampedSegment { timestamp, content },
    )
    .parse(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    map(
        delimited(
            char('['),
            (tag("#:"), space0, take_till(|c: char| c == ']')),
            (char(']'), space0, newline_or_end),
        ),
        |(_, _, comment)| comment,
    )
    .parse(i)
}

fn id_tag<'a>(i: &'a str) -> IResult<&'a str, IDTag<'a>> {
    map(
        delimited(
            char('['),
            (
                take_till(|c: char| c == ':'),
                char(':'),
                space0,
                take_till(|c: char| c == ']'),
            ),
            (char(']'), space0, newline_or_end),
        ),
        |(key, _, _, value)| IDTag { key, value },
    )
    .parse(i)
}

fn line_with_a2<'a>(i: &'a str) -> IResult<&'a str, TimestampedTag<'a>> {
    map(
        (standard_timestamp, space0, many0(a2_tag), newline_or_end),
        |(timestamp, _, tags, _)| TimestampedTag {
            timestamp,
            segments: tags,
        },
    )
    .parse(i)
}

fn standard_line<'a>(i: &'a str) -> IResult<&'a str, TimestampedTag<'a>> {
    map(
        (standard_timestamp, space0, till_newline, newline_or_end),
        |(timestamp, _, content, _)| TimestampedTag {
            timestamp,
            segments: vec![TimestampedSegment { timestamp, content }],
        },
    )
    .parse(i)
}

fn parse_line<'a>(i: &'a str) -> IResult<&'a str, Line<'a>> {
    alt((
        map(line_with_a2, Line::Tag),
        map(standard_line, Line::Tag),
        map(comment, Line::Comment),
        map(id_tag, Line::ID),
    ))
    .parse(i)
}

pub(crate) fn parse<'a>(i: &'a str) -> Result<Vec<Line<'a>>, nom::error::Error<&'a str>> {
    let (_, (lines, _)) = (many0(parse_line), eof).parse(i).finish()?;
    Ok(lines)
}
