#[cfg(test)]
mod tests;

use std::{collections::HashSet, time::Duration};

use nom::{
    AsChar, IResult,
    bytes::complete::{tag, take_until, take_while, take_while1},
};

use crate::{MediaPlaylist, MediaSegment};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Extinf {
    duration: Duration,
    title: Option<String>,
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
    Segment(String),
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

// TODO: Test this
pub(crate) fn parse_extinf_value(i: &str) -> IResult<&str, Extinf> {
    match take_until::<_, &str, nom::error::Error<&str>>(",")(i) {
        Ok((title, d)) => {
            let (title, _) = tag(",")(title)?;
            if title.is_empty() {
                panic!("The value separator \",\" is present but the title wasn't provided")
            }
            let duration = match d.parse::<f64>() {
                Ok(d) => Duration::from_secs_f64(d),
                Err(e) => panic!("Couldn't parse the duration as f64: {e}"),
            };
            Ok((
                "",
                Extinf {
                    duration,
                    title: Some(title.trim().to_string()),
                },
            ))
        }
        Err(_) => {
            let duration = Duration::from_secs_f64(match i.parse::<f64>() {
                Ok(d) => d,
                Err(e) => panic!("Could not parse the duration: {e}"),
            });
            Ok((
                "",
                Extinf {
                    duration,
                    title: None,
                },
            ))
        }
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
        Ok((i, Line::Segment(o.trim_end().to_string())))
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

pub fn parse_media_playlist(i: &str) -> IResult<&str, MediaPlaylist> {
    let (_, lines) = parse_lines(i)?;
    let mut seen_unique_tags: HashSet<_> = HashSet::new();
    let mut endlist = false; // Stop parsing media segments
    let mut extinf = None; // The last encountered #EXTINF tag content
    let mut gap = false; // Ignore the next media segment
    let mut segments = Vec::new();
    for (i, line) in lines.into_iter().enumerate() {
        if i == 0 {
            if line != Line::Tag("#EXTM3U".to_string()) {
                panic!("The first line in an M3U playlist must be the #EXTM3U tag");
            }
        } else {
            match line {
                Line::Tag(t) => {
                    eprintln!("Tag: {t}");
                    let (_, tag) = parse_tag_value(t.as_str()).unwrap();
                    if MULTIVARIANT_TAGS.contains(&tag.tag.as_str()) {
                        panic!("Found a multivariant tag in a media playlist: {}", tag.tag);
                    } else if UNIQUE_IGNORED_TAGS.contains(&tag.tag.as_str()) {
                        if seen_unique_tags.contains(tag.tag.as_str()) {
                            panic!(
                                "Tag \"{}\" must not be specified more than once in a playlist",
                                tag.tag
                            )
                        } else {
                            if IGNORED_TAGS_WITH_VALUES.contains(&tag.tag.as_str())
                                && tag.value.is_none()
                            {
                                panic!("Tag requires a value: \"{}\"", tag.tag)
                            }
                            seen_unique_tags.insert(tag.tag);
                        }
                    } else if IGNORED_TAGS.contains(&tag.tag.as_str())
                        && IGNORED_TAGS_WITH_VALUES.contains(&tag.tag.as_str())
                        && tag.value.is_none()
                    {
                        panic!("Tag requires a value: \"{}\"", tag.tag)
                    } else if tag.tag == "#EXT-X-ENDLIST" {
                        if tag.value.is_some() {
                            panic!("Unexpected value for \"{}\"", tag.tag);
                        }
                        endlist = true;
                    } else if tag.tag == "#EXTINF" {
                        if let Some(value) = tag.value {
                            match parse_extinf_value(&value) {
                                Ok((_, value)) => extinf = Some(value),
                                Err(e) => {
                                    panic!("Could not parse the #EXTINF tag value: {e}");
                                }
                            }
                        } else {
                            panic!("Expected a value after \"#EXTINF\"");
                        }
                    } else if tag.tag == "#EXT-X-GAP" {
                        gap = true;
                    } else {
                        panic!("Unknown tag: \"{}\"", tag.tag);
                    }
                }
                Line::Segment(uri) => {
                    if endlist || gap {
                        continue;
                    }
                    if let Some(ref inf) = extinf {
                        segments.push(MediaSegment {
                            uri,
                            duration: inf.duration,
                            title: inf.title.clone(),
                        });
                    } else {
                        panic!("The media segment was not prepended with an \"#EXTINF\" tag");
                    }
                }
                Line::Comment(c) => eprintln!("Comment: {c}"),
            }
        }
    }
    Ok(("", MediaPlaylist { segments }))
}
