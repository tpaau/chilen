pub mod parser;
#[cfg(test)]
mod tests;

use std::time::Duration;

/// Media segment from a [media playlist](Playlist).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MediaSegment {
    pub uri: String,
    pub duration: Duration,
    pub title: Option<String>,
}

/// A media playlist containing [audio tracks](Track).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MediaPlaylist {
    pub segments: Vec<MediaSegment>,
}

impl MediaPlaylist {
    /// Serialize the playlist struct to an M3U8 playlist.
    pub fn serialize(self) -> String {
        let mut content = String::from("#EXTM3U");

        for track in self.segments {
            match track.title {
                Some(title) => {
                    content += &format!(
                        "\n#EXTINF:{},{title}\n{}",
                        track.duration.as_secs(),
                        track.uri
                    )
                }
                None => {
                    content += &format!("\n#EXTINF:{}\n{}", track.duration.as_secs(), track.uri)
                }
            }
        }

        content
    }
}
