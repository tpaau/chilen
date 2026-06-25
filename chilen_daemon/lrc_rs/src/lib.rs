#![doc = include_str!("../README.md")]
#[cfg(feature = "parser")]
pub mod parser;
#[cfg(test)]
mod tests;

use std::time::Duration;

#[cfg(feature = "parser")]
pub use nom::{Finish, error::Error};

#[cfg(feature = "log")]
use log::warn;

/// Accessor trait for synced lyrics.
pub trait LyricsAccess: Sized {
    /// Removes timestamp data from the lyrics data and returns the unsynced content.
    fn to_unsynced(self) -> String;
    /// Returns lyrics content active at the timestamp or [`None`] if there is no content for the
    /// given timestamp.
    fn lyrics_at(&self, timestamp: Duration) -> Option<String>;
}

/// Segment of lyrics in a song, associated with a timestamp.
#[derive(Debug, PartialEq, Eq)]
pub struct TimestampedSegment {
    /// The time at which this segment begins to play in the song.
    pub timestamp: Duration,
    /// The actual lyrics content of this segment.
    pub content: String,
}

#[cfg(feature = "parser")]
impl<'a> From<parser::TimestampedSegment<'a>> for TimestampedSegment {
    fn from(value: parser::TimestampedSegment) -> Self {
        Self {
            timestamp: value.timestamp,
            content: value.content.to_string(),
        }
    }
}

impl TimestampedSegment {
    /// Checks if the segment is active at the given timestamp.
    pub fn is_active(&self, timestamp: Duration) -> bool {
        self.timestamp < timestamp
    }

    pub fn offset<'a>(&mut self, offset_ms: i64) -> Result<(), ParseError<'a>> {
        self.timestamp = duration_offset(self.timestamp, offset_ms)?;
        Ok(())
    }
}

/// A single line in the parsed lyrics.
///
/// With regular LRC files, this will only contain one element. If the enhanced LRC format is used
/// it may contain more elements.
///
/// You can check if the enhanced LRC format is used with the
/// [is_enhanced_lrc](SyncedLyrics::is_enhanced_lrc) method.
#[derive(Debug, PartialEq, Eq)]
pub struct TimestampedLine {
    /// The timestamp at which the line starts.
    ///
    /// Can be the same as or earlier than the timestamp of first segment.
    pub timestamp: Duration,
    /// Timestamped segments of the line.
    ///
    /// In regular LRC, there is only one segment with the same timestamp as the line timestamp.
    /// In enhanced LRC, there may be more than one segment, and the first segment’s timestamp
    /// may be later than the line timestamp.
    pub segments: Vec<TimestampedSegment>,
}

#[cfg(feature = "parser")]
impl<'a> From<parser::TimestampedTag<'a>> for TimestampedLine {
    fn from(value: parser::TimestampedTag) -> Self {
        Self {
            timestamp: value.timestamp,
            segments: value.segments.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl LyricsAccess for TimestampedLine {
    fn to_unsynced(self) -> String {
        let segments: Vec<_> = self.segments.into_iter().map(|s| s.content).collect();
        segments.join(" ")
    }

    fn lyrics_at(&self, timestamp: Duration) -> Option<String> {
        todo!()
    }
}

impl TimestampedLine {
    pub fn offset<'a>(&mut self, offset_ms: i64) -> Result<(), ParseError<'a>> {
        self.timestamp = duration_offset(self.timestamp, offset_ms)?;
        for segment in self.segments.iter_mut() {
            segment.timestamp = duration_offset(segment.timestamp, offset_ms)?;
        }
        Ok(())
    }
}

/// The player or editor that created the LRC file
#[derive(Debug, PartialEq, Eq)]
pub struct LRCTool {
    /// Name of the program
    pub name: String,
    /// Version of the program
    pub version: Option<String>,
}

/// Lyrics grouped into timestamped segments, possibly with some additional data.
#[derive(Debug, PartialEq, Eq)]
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
    /// This is omitted when serializing to LRC.
    pub comments: Vec<String>,
    // TODO: Offset - Specifies a global offset value for the lyric times, in milliseconds. The value is prefixed with either + or -, with + causing lyrics to appear sooner
    /// LRC segments grouped by lines
    pub lines: Vec<TimestampedLine>,
}

impl LyricsAccess for SyncedLyrics {
    fn to_unsynced(self) -> String {
        let lines: Vec<_> = self.lines.into_iter().map(|l| l.to_unsynced()).collect();
        lines.join("\n")
    }

    fn lyrics_at(&self, timestamp: Duration) -> Option<String> {
        todo!()
    }
}

impl SyncedLyrics {
    /// Checks if the lyrics contain any tags from the A2 extension.
    ///
    /// If the enhanced LRC format is used in the parsed lyrics, [TimestampedLine] can contain more
    /// than one segment.
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
}

#[derive(Debug, PartialEq, Eq)]
#[cfg(feature = "parser")]
pub enum ParseError<'a> {
    TimestampOffsetOverflow,
    UnknownKey(&'a str),
    Nom(nom::error::Error<&'a str>),
}

/// Parsed lyrics, can either by synced or unsynced
#[derive(Debug, PartialEq, Eq)]
pub enum Lyrics {
    /// Lyrics without timestamps
    Unsynced(String),
    /// Lyrics with time syncing
    Synced(Box<SyncedLyrics>),
}

impl Lyrics {
    #[cfg(feature = "parser")]
    fn parse_len<'a>(i: &'a str) -> Result<Duration, ParseError<'a>> {
        use nom::{Parser, combinator::eof};

        match (parser::timestamp, eof).parse(i).finish() {
            Ok((_, (len, _))) => Ok(len),
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse timestamp value: {e}");
                Err(ParseError::Nom(e))
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
                Err(ParseError::Nom(e))
            }
        }
    }

    // TODO: Add an error type or use the nom error type
    #[cfg(feature = "parser")]
    pub fn parse<'a>(input: &'a str) -> Result<Lyrics, (Lyrics, ParseError<'a>)> {
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
                        "length" => match Self::parse_len(tag.value) {
                            Ok(l) => length = Some(l),
                            Err(e) => return Err((Lyrics::Unsynced(input.to_string()), e)),
                        },
                        "by" => file_author = Some(tag.value.to_string()),
                        "offset" => match Self::parse_offset(tag.value) {
                            Ok(l) => offset = Some(l),
                            Err(e) => return Err((Lyrics::Unsynced(input.to_string()), e)),
                        },
                        "re" | "tool" => tool_name = Some(tag.value.to_string()),
                        "ve" => tool_version = Some(tag.value.to_string()),
                        _ => {
                            warn!("Unknown ID tag key \"{}\"", tag.key);
                            return Err((
                                Lyrics::Unsynced(input.to_string()),
                                ParseError::UnknownKey(tag.key),
                            ));
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
                    let lines: Result<Vec<_>, _> = lines
                        .into_iter()
                        .filter_map(|l| if let Line::Tag(t) = l { Some(t) } else { None })
                        .map(|t| t.into())
                        .map(|mut t: TimestampedLine| {
                            t.offset(offset)?;
                            Ok(t)
                        })
                        .collect();
                    match lines {
                        Ok(l) => l,
                        Err(e) => return Err((Lyrics::Unsynced(input.to_string()), e)),
                    }
                } else {
                    lines
                        .into_iter()
                        .filter_map(|l| if let Line::Tag(t) = l { Some(t) } else { None })
                        .map(|t| t.into())
                        .collect()
                };
                Ok(Lyrics::Synced(Box::new(SyncedLyrics {
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
                })))
            }
            Err(e) => {
                #[cfg(feature = "log")]
                warn!("Couldn't parse the LRC content: {e}");
                Err((Lyrics::Unsynced(input.to_string()), ParseError::Nom(e)))
            }
        }
    }

    /// Returns the content of unsynced lyrics or serializes synced lyrics to LRC format.
    pub fn serialize(self) -> String {
        match self {
            Self::Unsynced(l) => l,
            Self::Synced(l) => l.serialize(),
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
