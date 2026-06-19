#[cfg(test)]
mod tests;

use std::{collections::HashSet, time::Duration};

use nom::{
    AsChar, Finish, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::char,
    combinator::{map, peek},
    error::{ContextError, context},
    sequence::{preceded, terminated},
};
use nom_language::error::{VerboseError, VerboseErrorKind};

use crate::{MediaPlaylist, MediaSegment};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Extinf {
    duration: Duration,
    title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SplitTag<'a> {
    tag: &'a str,
    value: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Line<'a> {
    Tag(&'a str),
    Comment(&'a str),
    Segment(&'a str),
}

/// Tags ignored by the parser.
pub(crate) const IGNORED_TAGS: &[&str] = &[
    "#EXT-X-DEFINE",
    "#EXT-X-TARGETDURATION",
    "#EXT-X-MEDIA-SEQUENCE",
    "#EXT-X-DISCONTINUITY-SEQUENCE",
    "#EXT-X-PLAYLIST-TYPE",
    "#EXT-X-I-FRAMES-ONLY",
    "#EXT-X-PART-INF",
    "#EXT-X-SERVER-CONTROL",
    "#EXT-X-BYTERANGE",
    "#EXT-X-DISCONTINUITY",
    "#EXT-X-KEY",
    "#EXT-X-MAP",
    "#EXT-X-PROGRAM-DATE-TIME",
    "#EXT-X-BITRATE",
    "#EXT-X-PART",
    "#EXT-X-DATERANGE",
    "#EXT-X-PRELOAD-HINT",
    "#EXT-X-RENDITION-REPORT",
];

/// Ignored tags that mustn't appear more than once in a playlist.
pub(crate) const UNIQUE_IGNORED_TAGS: &[&str] = &[
    "#EXT-X-VERSION",
    "#EXT-X-INDEPENDENT-SEGMENTS",
    "#EXT-X-START",
    "#EXT-X-SKIP",
];

/// Tags that must have values assigned to them.
///
/// Items in here must also appear in [`IGNORED_TAGS`] or [`UNIQUE_IGNORED_TAGS`].
pub(crate) const IGNORED_TAGS_WITH_VALUES: &[&str] = &[
    "#EXT-X-VERSION",
    "#EXT-X-START",
    "#EXT-X-DEFINE",
    "#EXT-X-TARGETDURATION",
    "#EXT-X-MEDIA-SEQUENCE",
    "#EXT-X-DISCONTINUITY-SEQUENCE",
    "#EXT-X-PLAYLIST-TYPE",
    "#EXT-X-PART-INF",
    "#EXT-X-BYTERANGE",
    "#EXT-X-KEY",
    "#EXT-X-PROGRAM-DATE-TIME",
    "#EXT-X-BITRATE",
    "#EXT-X-PART",
    "#EXT-X-DATERANGE",
    "#EXT-X-SKIP",
    "#EXT-X-PRELOAD-HINT",
    "#EXT-X-RENDITION-REPORT",
];

pub(crate) const MULTIVARIANT_TAGS: &[&str] = &[
    "#EXT-X-MEDIA",
    "#EXT-X-STREAM-INF",
    "#EXT-X-I-FRAME-STREAM-INF",
    "#EXT-X-SESSION-DATA",
    "#EXT-X-SESSION-KEY",
    "#EXT-X-CONTENT-STEERING",
];

pub(crate) fn newline(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while1(|c: char| c.is_newline())(i)
}

pub(crate) fn newline_or_end(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    if i.is_empty() {
        Ok(("", ""))
    } else {
        newline(i)
    }
}

pub(crate) fn till_newline(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while::<_, &str, VerboseError<&str>>(|c: char| !c.is_newline())(i)
}

pub(crate) fn opt_whitespace(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while(|c: char| c.is_ascii_whitespace())(i)
}

pub(crate) fn split_tag_value<'a>(i: &'a str) -> SplitTag<'a> {
    match take_until::<_, &str, nom::error::Error<&str>>(":")(i) {
        Ok((i, t)) => {
            let (v, _) = tag::<_, &str, nom::error::Error<&str>>(":")(i).unwrap();
            SplitTag {
                tag: t,
                value: Some(v),
            }
        }
        Err(_) => SplitTag {
            tag: i,
            value: None,
        },
    }
}

pub(crate) fn parse_f64(i: &str) -> IResult<&str, f64, VerboseError<&str>> {
    match take_while1::<_, &str, VerboseError<&str>>(|c: char| c.is_ascii_digit())(i).finish() {
        Ok((i, v)) => match v.parse::<f64>() {
            Ok(v) => Ok((i, v)),
            Err(_) => Err(nom::Err::Error(VerboseError {
                errors: vec![(i, VerboseErrorKind::Context("parse_f64"))],
            })),
        },
        Err(e) => Err(nom::Err::Error(VerboseError::add_context(
            i,
            "parse_f64",
            e,
        ))),
    }
}

pub(crate) fn parse_extinf_value(input: &str) -> IResult<&str, Extinf, VerboseError<&str>> {
    let (i, dur) = match parse_f64(input).finish() {
        Ok((i, dur)) => (i, dur),
        Err(e) => {
            return Err(nom::Err::Error(VerboseError::add_context(
                input,
                "parse_extinf_value",
                e,
            )));
        }
    };
    match char::<&str, VerboseError<&str>>(',')(i) {
        Ok((i, _)) => match till_newline(i) {
            Ok((i, t)) => {
                let t = match opt_whitespace(t).finish() {
                    Ok((t, _)) => t,
                    Err(e) => {
                        return Err(nom::Err::Error(VerboseError::add_context(
                            input,
                            "parse_extinf_value",
                            e,
                        )));
                    }
                };
                if t.is_empty() {
                    return Err(nom::Err::Error(VerboseError {
                        errors: vec![(input, VerboseErrorKind::Context("parse_extinf_value"))],
                    }));
                }
                Ok((
                    i,
                    Extinf {
                        duration: Duration::from_secs_f64(dur),
                        title: Some(t.to_string()),
                    },
                ))
            }
            Err(e) => Err(e),
        },
        Err(_) => match till_newline(i) {
            Ok((i, _)) => Ok((
                i,
                Extinf {
                    duration: Duration::from_secs_f64(dur),
                    title: None,
                },
            )),
            Err(e) => Err(e),
        },
    }
}

pub(crate) fn parse_tag<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    context(
        "parse_tag",
        map(
            terminated(preceded(peek(tag("#EXT")), till_newline), newline_or_end),
            |line: &'a str| Line::Tag(line.trim_end()),
        ),
    )
    .parse(i)
}

pub(crate) fn parse_comment<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    context(
        "parse_comment",
        map(
            terminated(preceded(peek(tag("#")), till_newline), newline_or_end),
            |line: &'a str| Line::Comment(line.trim_end()),
        ),
    )
    .parse(i)
}

pub(crate) fn parse_segment<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    context(
        "parse_segment",
        map(terminated(till_newline, newline_or_end), |line: &'a str| {
            Line::Segment(line.trim_end())
        }),
    )
    .parse(i)
}

pub(crate) fn parse_line<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    let (i, _) = opt_whitespace(i)?;
    context(
        "parse_line",
        preceded(
            opt_whitespace,
            alt((parse_tag, parse_comment, parse_segment)),
        ),
    )
    .parse(i)
}

pub(crate) fn parse_lines<'a>(
    i: &'a str,
) -> IResult<&'a str, Vec<Line<'a>>, VerboseError<&'a str>> {
    let mut i = i;
    let mut lines = Vec::new();
    while !i.is_empty() {
        let (remaining, line) = parse_line(i)?;
        i = remaining;
        lines.push(line);
    }
    Ok((i, lines))
}

pub(crate) fn extm3u_tag(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    match tag::<&str, &str, VerboseError<&str>>("#EXTM3U")(i).finish() {
        Ok((i, o)) => {
            let (i, _) = newline_or_end(i)?;
            Ok((i, o))
        }
        Err(e) => Err(nom::Err::Error(VerboseError::add_context(
            i,
            "extm3u_tag",
            e,
        ))),
    }
}

pub(crate) fn no_multivariant_tag(tag: &str) -> Result<(), VerboseError<&str>> {
    if MULTIVARIANT_TAGS.contains(&tag) {
        Err(VerboseError {
            errors: vec![(tag, VerboseErrorKind::Context("no_multivariant_tag"))],
        })
    } else {
        Ok(())
    }
}

pub(crate) fn tag_has_value<'a>(tag: &SplitTag<'a>) -> Result<(), VerboseError<&'a str>> {
    if tag.value.is_some() {
        Ok(())
    } else {
        Err(VerboseError {
            errors: vec![(tag.tag, VerboseErrorKind::Context("tag_has_value"))],
        })
    }
}

pub fn parse_media_playlist(input: &str) -> IResult<&str, MediaPlaylist, VerboseError<&str>> {
    let i = match extm3u_tag(input).finish() {
        Ok((i, _)) => i,
        Err(e) => {
            return Err(nom::Err::Error(VerboseError::add_context(
                input,
                "parse_media_playlist",
                e,
            )));
        }
    };
    if i.is_empty() {
        return Ok((
            i,
            MediaPlaylist {
                segments: Vec::new(),
            },
        ));
    }
    let (_, lines) = parse_lines(i)?;
    let mut seen_unique_tags: HashSet<_> = HashSet::new();
    let mut endlist = false; // Stop parsing media segments
    let mut extinf = None; // The last encountered #EXTINF tag content
    let mut gap = false; // Ignore the next media segment
    let mut segments = Vec::new();
    for line in lines.into_iter() {
        match line {
            Line::Tag(t) => {
                eprintln!("Tag: {t}");
                let tag = split_tag_value(t);
                if let Err(e) = no_multivariant_tag(tag.tag) {
                    return Err(nom::Err::Error(VerboseError::add_context(
                        input,
                        "parse_media_playlist",
                        e,
                    )));
                }
                if UNIQUE_IGNORED_TAGS.contains(&tag.tag) {
                    if seen_unique_tags.contains(tag.tag) {
                        return Err(nom::Err::Error(VerboseError {
                            errors: vec![(
                                tag.tag,
                                VerboseErrorKind::Context("parse_media_playlist"),
                            )],
                        }));
                    } else {
                        if IGNORED_TAGS_WITH_VALUES.contains(&tag.tag)
                            && let Err(e) = tag_has_value(&tag)
                        {
                            return Err(nom::Err::Error(VerboseError::add_context(
                                input,
                                "parse_media_playlist",
                                e,
                            )));
                        }
                        seen_unique_tags.insert(tag.tag);
                    }
                } else if IGNORED_TAGS.contains(&tag.tag)
                    && IGNORED_TAGS_WITH_VALUES.contains(&tag.tag)
                    && let Err(e) = tag_has_value(&tag)
                {
                    return Err(nom::Err::Error(VerboseError::add_context(
                        input,
                        "parse_media_playlist",
                        e,
                    )));
                } else if tag.tag == "#EXT-X-ENDLIST" {
                    if tag.value.is_some() {
                        return Err(nom::Err::Error(VerboseError {
                            errors: vec![(
                                tag.tag,
                                VerboseErrorKind::Context("parse_media_playlist"),
                            )],
                        }));
                    }
                    endlist = true;
                } else if tag.tag == "#EXTINF" {
                    if let Some(value) = tag.value {
                        match parse_extinf_value(value).finish() {
                            Ok((_, value)) => extinf = Some(value),
                            Err(e) => {
                                return Err(nom::Err::Error(VerboseError::add_context(
                                    value,
                                    "parse_media_playlist",
                                    e,
                                )));
                            }
                        };
                    } else {
                        return Err(nom::Err::Error(VerboseError {
                            errors: vec![(
                                tag.tag,
                                VerboseErrorKind::Context("parse_media_playlist"),
                            )],
                        }));
                    }
                } else if tag.tag == "#EXT-X-GAP" {
                    gap = true;
                } else {
                    return Err(nom::Err::Error(VerboseError {
                        errors: vec![(tag.tag, VerboseErrorKind::Context("parse_media_playlist"))],
                    }));
                }
            }
            Line::Segment(uri) => {
                if endlist || gap {
                    continue;
                }
                if let Some(ref inf) = extinf {
                    segments.push(MediaSegment {
                        uri: uri.to_string(),
                        duration: inf.duration,
                        title: inf.title.clone(),
                    });
                } else {
                    return Err(nom::Err::Error(VerboseError {
                        errors: vec![(uri, VerboseErrorKind::Context("parse_media_playlist"))],
                    }));
                }
            }
            Line::Comment(_) => {}
        }
    }
    Ok(("", MediaPlaylist { segments }))
}
