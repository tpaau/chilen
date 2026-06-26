#![doc = include_str!("../README.md")]
#[cfg(feature = "parser")]
mod parser;
#[cfg(test)]
mod tests;

use std::time::Duration;

#[cfg(feature = "parser")]
use nom::Finish;
#[cfg(feature = "parser")]
pub use nom::error::ErrorKind;

#[cfg(feature = "log")]
use log::warn;

/// Error indicating why lyrics couldn't be parsed as LRC.
#[cfg(feature = "parser")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseError<'a> {
    /// An overflow occurred while offsetting timestamps with the value of the LRC `offset` tag
    /// (eg. `[offset: +100]`).
    ///
    /// Try adjusting the offset value or removing the offset tag.
    TimestampOffsetOverflow,
    /// Encountered an ID tag with an unknown key.
    ///
    /// Remove the broken tag.
    UnknownKey(&'a str),
    /// Parsing failed due to a syntax error.
    Nom {
        /// The input for which the error occurred.
        input: &'a str,
        /// The error code.
        error: nom::error::ErrorKind,
    },
}

#[cfg(feature = "parser")]
impl<'a> From<nom::error::Error<&'a str>> for ParseError<'a> {
    fn from(value: nom::error::Error<&'a str>) -> Self {
        Self::Nom {
            input: value.input,
            error: value.code,
        }
    }
}

#[cfg(feature = "parser")]
impl<'a> std::fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimestampOffsetOverflow => {
                write!(f, "An overflow occurred while offsetting a timestamp")
            }
            Self::UnknownKey(key) => write!(f, "Unknown ID tag key: \"{key}\""),
            Self::Nom { input, error } => {
                write!(f, "Couldn't parse the lyrics at `{input}`: {error:?}`")
            }
        }
    }
}

/// Accessor trait for synced lyrics.
pub trait LyricsAccess: Sized {
    /// Returns unsynced lyrics without timestamps or additional metadata.
    fn to_unsynced(self) -> String;
    /// Returns lyrics content active at the timestamp or [`None`] if there is no content for the
    /// given timestamp.
    fn lyrics_at(&self, timestamp: Duration) -> Option<&str>;
}

/// Segment of lyrics in a [single line](LineTag), associated with a timestamp.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct SegmentTag {
    /// The timestamp at which the segment starts.
    pub timestamp: Duration,
    /// The content of the segment.
    pub content: String,
}

#[cfg(feature = "parser")]
impl<'a> From<parser::TimestampedSegment<'a>> for SegmentTag {
    fn from(value: parser::TimestampedSegment) -> Self {
        Self {
            timestamp: value.timestamp,
            content: value.content.to_string(),
        }
    }
}

impl SegmentTag {
    /// Checks if the segment is active at the given timestamp.
    pub fn is_active(&self, timestamp: Duration) -> bool {
        self.timestamp < timestamp
    }
}

/// A single line in the synced lyrics.
///
/// With regular LRC files, this will contain at most one element. If the enhanced LRC format is
/// used, it may contain more elements.
///
/// You can check if the enhanced LRC format is used with the
/// [`is_enhanced_lrc`](SyncedLyrics::is_enhanced_lrc) method.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LineTag {
    /// The timestamp at which the line starts.
    ///
    /// Can be the same as or earlier than the timestamp of first segment.
    pub timestamp: Duration,
    /// Timestamped segments of the line.
    ///
    /// Segments in lines with A2 tags can have timestamps that are later than the line timestamp.
    /// With regular LRC, there will always be at most one segment with the same timestamp as the
    /// line timestamp.
    pub segments: Vec<SegmentTag>,
}

#[cfg(feature = "parser")]
impl<'a> From<parser::TimestampedTag<'a>> for LineTag {
    fn from(value: parser::TimestampedTag) -> Self {
        Self {
            timestamp: value.timestamp,
            segments: value.segments.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl LyricsAccess for LineTag {
    fn to_unsynced(self) -> String {
        let segments: Vec<_> = self.segments.into_iter().map(|s| s.content).collect();
        segments.join("")
    }

    fn lyrics_at(&self, timestamp: Duration) -> Option<&str> {
        todo!()
    }
}

impl LineTag {
    #[cfg(feature = "parser")]
    fn offset<'a>(&mut self, offset_ms: i64) -> Result<(), ParseError<'a>> {
        self.timestamp = duration_offset(self.timestamp, offset_ms)?;
        for segment in self.segments.iter_mut() {
            segment.timestamp = duration_offset(segment.timestamp, offset_ms)?;
        }
        Ok(())
    }
}

/// The player or editor that created the LRC file
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LRCTool {
    /// Name of the program
    pub name: String,
    /// Version of the program
    pub version: Option<String>,
}

/// Lyrics grouped into timestamped segments with additional metadata.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct SyncedLyrics {
    /// Title of the song
    pub title: Option<String>,
    /// Artist performing the song
    pub artist: Option<String>,
    /// Album the song is from
    pub album: Option<String>,
    /// Author of the song
    pub author: Option<String>,
    /// Lyricist of the song
    pub lyricist: Option<String>,
    /// Length of the song
    pub length: Option<Duration>,
    /// The player or editor that created the LRC file
    pub tool: Option<LRCTool>,
    /// Author of the LRC file (not the song)
    pub file_author: Option<String>,
    /// Comments found in the lyrics
    ///
    /// **NOTE**: This field is omitted when serializing to LRC.
    pub comments: Vec<String>,
    /// LRC segments grouped by lines
    pub lines: Vec<LineTag>,
}

impl LyricsAccess for SyncedLyrics {
    fn to_unsynced(self) -> String {
        let lines: Vec<_> = self.lines.into_iter().map(|l| l.to_unsynced()).collect();
        lines.join("\n")
    }

    fn lyrics_at(&self, timestamp: Duration) -> Option<&str> {
        todo!()
    }
}

impl SyncedLyrics {
    #[cfg(feature = "parser")]
    fn parse_len<'a>(i: &'a str) -> Result<Duration, ParseError<'a>> {
        use nom::{Parser, combinator::eof};

        match (parser::timestamp, eof).parse(i).finish() {
            Ok((_, (len, _))) => Ok(len),
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse timestamp value: {e}");
                Err(ParseError::from(e))
            }
        }
    }

    #[cfg(feature = "parser")]
    fn parse_offset<'a>(i: &'a str) -> Result<i64, ParseError<'a>> {
        match parser::offset(i).finish() {
            Ok((_, offset)) => Ok(offset),
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse offset value: {e}");
                Err(ParseError::from(e))
            }
        }
    }

    /// Checks if the lyrics contain any tags from the A2 extension.
    ///
    /// If the enhanced LRC format is used, [line tags](LineTag) may contain more than one segment.
    pub fn is_enhanced_lrc(&self) -> bool {
        for line in &self.lines {
            if line.segments.is_empty() {
                #[cfg(feature = "log")]
                warn!("Line segments list is empty, skipping");
                continue;
            } else if line.segments.len() > 1 || line.timestamp < line.segments[0].timestamp {
                return true;
            }
        }
        false
    }

    /// Serialize the struct to LRC format.
    pub fn serialize(self) -> String {
        todo!()
    }

    /// Parses LRC lyrics data.
    #[cfg(feature = "parser")]
    pub fn parse<'a>(input: &'a str) -> Result<Self, ParseError<'a>> {
        match parser::parse(input) {
            Ok(lines) => {
                use crate::parser::Line;

                let id_tags: Vec<_> = lines
                    .iter()
                    .filter_map(|l| if let Line::ID(t) = l { Some(t) } else { None })
                    .collect();
                let comments: Vec<_> = lines
                    .iter()
                    .filter_map(|l| {
                        if let Line::Comment(c) = l {
                            Some(c.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut title = None;
                let mut artist = None;
                let mut album = None;
                let mut author = None;
                let mut lyricist = None;
                let mut length = None;
                let mut file_author = None;
                let mut offset = None;
                let mut tool_name = None;
                let mut tool_version = None;
                for tag in id_tags {
                    match tag.key {
                        "ti" => title = Some(tag.value.to_string()),
                        "ar" => artist = Some(tag.value.to_string()),
                        "al" => album = Some(tag.value.to_string()),
                        "au" => author = Some(tag.value.to_string()),
                        "lr" => lyricist = Some(tag.value.to_string()),
                        "length" => {
                            let l = Self::parse_len(tag.value)?;
                            length = Some(l)
                        }
                        "by" => file_author = Some(tag.value.to_string()),
                        "offset" => {
                            let l = Self::parse_offset(tag.value)?;
                            offset = Some(l)
                        }
                        "re" | "tool" => tool_name = Some(tag.value.to_string()),
                        "ve" => tool_version = Some(tag.value.to_string()),
                        _ => {
                            warn!("Unknown ID tag key \"{}\"", tag.key);
                            return Err(ParseError::UnknownKey(tag.key));
                        }
                    }
                }
                let tool = match tool_name {
                    Some(name) => Some(LRCTool {
                        name: name.to_string(),
                        version: tool_version.map(|version| version.to_string()),
                    }),
                    None => {
                        if tool_version.is_some() {
                            #[cfg(feature = "log")]
                            warn!("The tool version is specified but the name isn't, ignoring");
                        }
                        None
                    }
                };
                let lines = if let Some(offset) = offset {
                    let lines: Result<Vec<_>, ParseError<'a>> = lines
                        .into_iter()
                        .filter_map(|l| if let Line::Tag(t) = l { Some(t) } else { None })
                        .map(|t| t.into())
                        .map(|mut t: LineTag| {
                            t.offset(offset)?;
                            Ok(t)
                        })
                        .collect();
                    lines?
                } else {
                    lines
                        .into_iter()
                        .filter_map(|l| if let Line::Tag(t) = l { Some(t) } else { None })
                        .map(|t| t.into())
                        .collect()
                };
                Ok(SyncedLyrics {
                    title,
                    artist,
                    album,
                    author,
                    lyricist,
                    length,
                    tool,
                    file_author,
                    comments,
                    lines,
                })
            }
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse the LRC content: {e}");
                Err(ParseError::from(e))
            }
        }
    }
}

#[cfg(feature = "parser")]
pub(crate) fn duration_offset<'a>(
    dur: Duration,
    offset_ms: i64,
) -> Result<Duration, ParseError<'a>> {
    match offset_ms.try_into() {
        Ok(offset) => match dur.checked_add(Duration::from_millis(offset)) {
            Some(dur) => Ok(dur),
            None => Err(ParseError::TimestampOffsetOverflow),
        },
        Err(_) => match dur.checked_sub(Duration::from_millis(offset_ms.unsigned_abs())) {
            Some(dur) => Ok(dur),
            None => Err(ParseError::TimestampOffsetOverflow),
        },
    }
}
