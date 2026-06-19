#[cfg(test)]
mod tests;

use std::{collections::HashSet, path::PathBuf, time::Duration};

use log::{error, trace, warn};
use nom::{
    AsChar, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::char,
    combinator::{map, peek},
    error::{ContextError, context},
    sequence::{pair, preceded, terminated},
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
        context("newline_or_end", newline).parse(i)
    }
}

pub(crate) fn till_newline(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while::<_, &str, VerboseError<&str>>(|c: char| !c.is_newline())(i)
}

pub(crate) fn opt_whitespace(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while(|c: char| c.is_ascii_whitespace())(i)
}

pub(crate) fn split_tag_value<'a>(i: &'a str) -> SplitTag<'a> {
    if let Some((t, v)) = i.split_once(':') {
        SplitTag {
            tag: t,
            value: Some(v),
        }
    } else {
        SplitTag {
            tag: i,
            value: None,
        }
    }
}

pub(crate) fn parse_f64(i: &str) -> IResult<&str, f64, VerboseError<&str>> {
    let (i, v) = context("parse_f64", take_while1(|c: char| c.is_ascii_digit())).parse(i)?;
    match v.parse::<f64>() {
        Ok(v) => Ok((i, v)),
        Err(e) => {
            error!("Could not parse the string as f64: {e}");
            Err(nom::Err::Error(VerboseError {
                errors: vec![(i, VerboseErrorKind::Context("parse_f64"))],
            }))
        }
    }
}

pub(crate) fn parse_extinf_value(input: &str) -> IResult<&str, Extinf, VerboseError<&str>> {
    let (i, dur) = context("parse_extinf_value", parse_f64).parse(input)?;
    match char::<&str, VerboseError<&str>>(',')(i) {
        Ok((i, _)) => match till_newline(i) {
            Ok((i, t)) => {
                let (t, _) = context("parse_extinf_value", opt_whitespace).parse(t)?;
                if t.is_empty() {
                    error!("The \",\" separator is present, but no title was provided");
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
        Err(_) => {
            let (i, _) = context("parse_extinf_value", till_newline).parse(i)?;
            Ok((
                i,
                Extinf {
                    duration: Duration::from_secs_f64(dur),
                    title: None,
                },
            ))
        }
    }
}

pub(crate) fn parse_tag<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    context(
        "parse_tag",
        map(
            terminated(
                preceded(preceded(opt_whitespace, peek(tag("#EXT"))), till_newline),
                newline_or_end,
            ),
            |line: &'a str| Line::Tag(line.trim_end()),
        ),
    )
    .parse(i)
}

pub(crate) fn parse_comment<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    context(
        "parse_comment",
        map(
            terminated(
                preceded(preceded(opt_whitespace, peek(tag("#"))), till_newline),
                newline_or_end,
            ),
            |line: &'a str| Line::Comment(line.trim_end()),
        ),
    )
    .parse(i)
}

pub(crate) fn parse_segment<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    context(
        "parse_segment",
        map(
            terminated(preceded(opt_whitespace, till_newline), newline_or_end),
            |line: &'a str| Line::Segment(line.trim_end()),
        ),
    )
    .parse(i)
}

pub(crate) fn parse_line<'a>(i: &'a str) -> IResult<&'a str, Line<'a>, VerboseError<&'a str>> {
    context("parse_line", alt((parse_tag, parse_comment, parse_segment))).parse(i)
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
    context(
        "extm3u_tag",
        terminated(
            tag("#EXTM3U"),
            pair(
                take_while(|c: char| c.is_ascii_whitespace() && !c.is_newline()),
                newline_or_end,
            ),
        ),
    )
    .parse(i)
}

pub(crate) fn no_multivariant_tag(tag: &str) -> IResult<&str, &str, VerboseError<&str>> {
    if MULTIVARIANT_TAGS.contains(&tag) {
        error!("Encountered a multivariant tag \"{tag}\" in a media playlist");
        Err(nom::Err::Error(VerboseError {
            errors: vec![(tag, VerboseErrorKind::Context("no_multivariant_tag"))],
        }))
    } else {
        Ok(("", tag))
    }
}

pub(crate) fn tag_has_value<'a>(tag: &SplitTag<'a>) -> Result<(), VerboseError<&'a str>> {
    if tag.value.is_some() {
        Ok(())
    } else {
        error!("Expected a value for tag \"{}\"", tag.tag);
        Err(VerboseError {
            errors: vec![(tag.tag, VerboseErrorKind::Context("tag_has_value"))],
        })
    }
}

pub fn parse_media_playlist(i: &str) -> IResult<&str, MediaPlaylist, VerboseError<&str>> {
    trace!("Parsing media playlist");
    let (i, _) = context("parse_media_playlist", extm3u_tag).parse(i)?;
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
                let tag = split_tag_value(t);
                context("parse_media_playlist", no_multivariant_tag).parse(tag.tag)?;
                if UNIQUE_IGNORED_TAGS.contains(&tag.tag) {
                    if seen_unique_tags.contains(tag.tag) {
                        error!(
                            "Tag \"{}\" must not appear more than once in a playlist",
                            tag.tag
                        );
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
                                tag.tag,
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
                        tag.tag,
                        "parse_media_playlist",
                        e,
                    )));
                } else if tag.tag == "#EXT-X-ENDLIST" {
                    if let Err(e) = tag_has_value(&tag) {
                        return Err(nom::Err::Error(VerboseError::add_context(
                            tag.tag,
                            "parse_media_playlist",
                            e,
                        )));
                    }
                    endlist = true;
                } else if tag.tag == "#EXTINF" {
                    if let Some(value) = tag.value {
                        extinf = Some(
                            context("parse_media_playlist", parse_extinf_value)
                                .parse(value)?
                                .1,
                        );
                    } else {
                        error!("Expected a value for tag \"{}\"", tag.tag);
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
                    warn!("Unknown tag: {}, ignoring", tag.tag);
                }
            }
            Line::Segment(uri) => {
                if endlist || gap {
                    continue;
                }
                if let Some(ref inf) = extinf {
                    segments.push(MediaSegment {
                        uri: PathBuf::from(uri),
                        duration: inf.duration,
                        title: inf.title.clone(),
                    });
                } else {
                    error!("No \"#EXTINF\" tag before a media segment");
                    return Err(nom::Err::Error(VerboseError {
                        errors: vec![(uri, VerboseErrorKind::Context("parse_media_playlist"))],
                    }));
                }
            }
            Line::Comment(_) => {}
        }
    }
    trace!("Parsed the media playlist");
    Ok(("", MediaPlaylist { segments }))
}
