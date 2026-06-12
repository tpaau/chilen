#[cfg(test)]
mod tests;

use std::{collections::HashSet, sync::LazyLock};

use nom::{
    AsChar, IResult,
    bytes::complete::{tag, take_until, take_while, take_while1},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Tag {
    /// Indicates that the file is an M3U playlist, must be the first line of the file.
    EXTM3U,
    /// The version of the M3U format in the file, must only be specified once.
    Version(String),
    /// Unsupported media playlist tag.
    Unsupported,
    /// Tag that should only exist in multivariant playlists.
    Multivariant,
    /// Invalid tag.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ParsedTag {
    tag: String,
    value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Line {
    Tag(String),
    Comment(String),
    Path(String),
}

/// The parser must fail if any of these tags are encountered as only media playlists are supported.
static MUTIVARIANT_TAGS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    let tags = [
        "#EXT-X-MEDIA",
        "#EXT-X-STREAM-INF",
        "#EXT-X-I-FRAME-STREAM-INF",
        "#EXT-X-SESSION-DATA",
        "#EXT-X-SESSION-KEY",
        "#EXT-X-CONTENT-STEERING",
    ];
    let mut set = HashSet::new();
    // panic!("{}", set.capacity());
    for tag in tags {
        set.insert(tag);
    }
    set
});

pub(crate) fn newline(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_newline())(i)
}

pub(crate) fn newline_or_end(i: &str) -> IResult<&str, &str> {
    if i.is_empty() {
        Ok(("", ""))
    } else {
        newline(i)
    }
}

pub(crate) fn opt_whitespace(i: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_whitespace())(i)
}

pub(crate) fn non_newline_whitespace(i: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_whitespace() && !c.is_newline())(i)
}

pub(crate) fn parse_tag_value(i: &str) -> IResult<&str, ParsedTag> {
    match take_until::<_, &str, nom::error::Error<&str>>(":")(i) {
        Ok((i, t)) => {
            let (v, _) = tag(":")(i)?;
            Ok((
                "",
                ParsedTag {
                    tag: t.to_string(),
                    value: Some(v.to_string()),
                },
            ))
        }
        Err(_) => Ok((
            "",
            ParsedTag {
                tag: i.to_string(),
                value: None,
            },
        )),
    }
}

pub(crate) fn parse_line(i: &str) -> IResult<&str, Line> {
    let (i, _) = opt_whitespace(i)?;
    if tag::<&str, &str, nom::error::Error<&str>>("#")(i).is_ok() {
        if tag::<&str, &str, nom::error::Error<&str>>("#EXT")(i).is_ok() {
            let (i, o) = take_while(|c: char| !c.is_newline())(i)?;
            let (i, _) = newline_or_end(i)?;
            Ok((i, Line::Tag(o.trim_end().to_string())))
        } else {
            let (i, o) = take_while(|c: char| !c.is_newline())(i)?;
            let (i, _) = newline_or_end(i)?;
            Ok((i, Line::Comment(o.trim_end().to_string())))
        }
    } else {
        let (i, o) = take_while(|c: char| !c.is_newline())(i)?;
        let (i, _) = newline_or_end(i)?;
        Ok((i, Line::Path(o.trim_end().to_string())))
    }
}

pub(crate) fn parse_lines(i: &str) -> IResult<&str, Vec<Line>> {
    let mut i = i;
    let mut lines = Vec::new();
    while !i.is_empty() {
        let (remaining, line) = parse_line(i)?;
        i = remaining;
        lines.push(line);
    }
    Ok((i, lines))
}

pub fn parse_media_playlist(i: &str) -> IResult<&str, &str> {
    let (_, lines) = parse_lines(i)?;
    for line in lines {
        match line {
            Line::Tag(t) => eprintln!("Tag: {t}"),
            Line::Path(p) => eprintln!("Path: {p}"),
            Line::Comment(c) => eprintln!("Comment: {c}"),
        }
    }
    todo!()
}
