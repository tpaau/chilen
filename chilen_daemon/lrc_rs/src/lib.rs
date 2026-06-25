#![doc = include_str!("../README.md")]
#[cfg(feature = "parser")]
pub mod parser;
#[cfg(test)]
mod tests;

use std::time::Duration;

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
pub struct TimestampedSegment {
    /// The time at which this segment begins to play in the song.
    pub timestamp: Duration,
    /// The actual lyrics content of this segment.
    pub content: String,
}

impl TimestampedSegment {
    /// Checks if the segment is active at the given timestamp.
    pub fn is_active(&self, timestamp: Duration) -> bool {
        self.timestamp < timestamp
    }
}

/// A single line in the parsed lyrics.
///
/// With regular LRC files, this will only contain one element. If the enhanced LRC format is used
/// it may contain more elements.
///
/// You can check if the enhanced LRC format is used with the
/// [is_enhanced_lrc](SyncedLyrics::is_enhanced_lrc) method.
pub struct SyncedLyricsLine {
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

impl LyricsAccess for SyncedLyricsLine {
    fn to_unsynced(self) -> String {
        let segments: Vec<_> = self.segments.into_iter().map(|s| s.content).collect();
        segments.join(" ")
    }

    fn lyrics_at(&self, timestamp: Duration) -> Option<String> {
        todo!()
    }
}

/// The player or editor that created the LRC file
pub struct LRCTool {
    /// Name of the program
    pub name: String,
    /// Version of the program
    pub version: Option<String>,
}

/// Lyrics grouped into timestamped segments, possibly with some additional data.
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
    // TODO: Offset - Specifies a global offset value for the lyric times, in milliseconds. The value is prefixed with either + or -, with + causing lyrics to appear sooner
    /// LRC segments grouped by lines
    lines: Vec<SyncedLyricsLine>,
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
    /// If the enhanced LRC format is used in the parsed lyrics, [SyncedLyricsLine] can contain more
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

/// Parsed lyrics, can either by synced or unsynced
pub enum Lyrics {
    /// Lyrics without timestamps
    Unsynced(String),
    /// Parsed LRC lyrics
    Synced(Box<SyncedLyrics>),
}

impl Lyrics {
    // TODO: Add an error type or use the nom error type
    #[cfg(feature = "parser")]
    pub fn parse(i: &str) -> Result<Lyrics, (Lyrics, String)> {
        todo!()
    }

    /// Returns the content of unsynced lyrics or serializes synced lyrics to LRC format.
    pub fn serialize(self) -> String {
        match self {
            Self::Unsynced(l) => l,
            Self::Synced(l) => l.serialize(),
        }
    }
}
