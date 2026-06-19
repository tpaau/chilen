pub mod parser;
#[cfg(test)]
mod tests;

use std::{path::PathBuf, time::Duration};

use log::trace;

/// Media segment from a [media playlist](Playlist).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MediaSegment {
    pub uri: PathBuf,
    pub duration: Duration,
    pub title: Option<String>,
}

/// A media playlist containing [audio tracks](Track).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MediaPlaylist {
    pub segments: Vec<MediaSegment>,
}

impl MediaPlaylist {
    /// Create a new empty playlist.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Serialize the playlist struct to an M3U8 playlist.
    pub fn serialize(self) -> String {
        trace!("Serializing media playlist to String");
        let mut content = String::from("#EXTM3U");

        for track in self.segments {
            match track.title {
                Some(title) => {
                    content += &format!(
                        "\n#EXTINF:{},{title}\n{}",
                        track.duration.as_secs(),
                        track.uri.to_string_lossy()
                    )
                }
                None => {
                    content += &format!(
                        "\n#EXTINF:{}\n{}",
                        track.duration.as_secs(),
                        track.uri.to_string_lossy()
                    )
                }
            }
        }
        trace!("Done serializing the media playlist");
        content
    }
}
