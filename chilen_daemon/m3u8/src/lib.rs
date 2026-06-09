#[cfg(test)]
mod tests;

use std::time::Duration;

use nom::{
    AsChar, IResult, Needed,
    bytes::complete::{tag, take_till, take_while, take_while1},
};

/// Media segment from a [media playlist](Playlist).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Track {
    pub uri: String,
    pub duration: Duration,
    pub title: Option<String>,
}

/// A media playlist containing [audio tracks](Track).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Playlist {
    tracks: Vec<Track>,
}

fn till_newline(i: &str) -> IResult<&str, &str> {
    take_till(|c: char| c.is_newline())(i)
}

fn m3u8_tag(i: &str) -> IResult<&str, &str> {
    tag("#EXT")(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    tag("#")(i)
}

fn parse_tag_x_version(i: &str) -> IResult<&str, &str> {
    tag("-X-VERSION:")(i)
}

fn newline(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_newline())(i)
}

fn opt_whitespace(i: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_whitespace())(i)
}

impl Playlist {
    /// Serialize the playlist struct to an M3U8 playlist.
    pub fn serialize(self) -> String {
        let mut content = String::from("#EXTM3U");

        for track in self.tracks {
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

    pub fn deserialize(i: &str) -> IResult<&str, Playlist> {
        let mut i = match tag("#EXTM3U\n")(i) {
            Ok((i, _)) => {
                if i.is_empty() {
                    return Ok((i, Playlist::default()));
                } else {
                    i
                }
            }
            Err(e) => {
                return if e == nom::Err::Incomplete(Needed::new(1)) {
                    Ok(("", Playlist::default()))
                } else {
                    Err(e)
                };
            }
        };

        if !i.is_empty() {
            eprintln!("Before whitespace trim: \"{i}\"");
            (i, _) = opt_whitespace(i)?;
            eprintln!("After whitespace trim: \"{i}\"");
        } else {
            eprintln!("Rest is empty, returning an empty playlist");
            return Ok((i, Playlist::default()));
        }

        let mut m3u8_version = None;
        while !i.is_empty() {
            eprintln!("Remaining: \"{i}\"");
            if let Ok((tag_id, _)) = m3u8_tag(i) {
                i = match till_newline(tag_id) {
                    Ok((i, tag)) => {
                        eprintln!("Processing tag: \"{}\"", tag);
                        if let Ok(version) = parse_tag_x_version(tag) {
                            if m3u8_version.is_none() {
                                m3u8_version = Some(version)
                            } else {
                                return Err(nom::Err::Error(nom::error::Error::new(
                                    i,
                                    nom::error::ErrorKind::Tag,
                                )));
                            }
                        } else {
                            panic!("Unknown tag: \"{tag}\"");
                        }
                        i
                    }
                    Err(e) => panic!("{e}"),
                };
            } else if let Ok((rest, _)) = comment(i) {
                i = match till_newline(rest) {
                    Ok((i, comment)) => {
                        eprintln!("Ignoring comment: \"{comment}\"");
                        i
                    }
                    Err(e) => panic!("{e}"),
                }
            } else {
                panic!("Couldn't match the remaining content: \"{i}\"",);
            }
            if !i.is_empty() {
                eprintln!("Before: \"{i}\"");
                (i, _) = newline(i)?;
                (i, _) = opt_whitespace(i)?;
                eprintln!("After: \"{i}\"");
            }
        }

        Ok((i, Playlist::default()))
    }
}
