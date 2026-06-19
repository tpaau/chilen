use std::{path::PathBuf, time::Duration};

use crate::{MediaPlaylist, MediaSegment, parser::parse_media_playlist};

#[test]
fn serialize_empty_playlist() {
    assert_eq!(
        MediaPlaylist::default().serialize(),
        String::from("#EXTM3U")
    );
    assert_eq!(
        parse_media_playlist(MediaPlaylist::default().serialize().as_str()),
        Ok(("", MediaPlaylist::default()))
    );
}

#[test]
fn serialize_with_playlists() {
    let playlist = MediaPlaylist {
        segments: vec![
            MediaSegment {
                uri: PathBuf::from("/some/path1"),
                duration: Duration::from_secs(67),
                title: None,
            },
            MediaSegment {
                uri: PathBuf::from("/some/path2"),
                duration: Duration::from_secs(0),
                title: Some(String::from("Fun Track")),
            },
            MediaSegment {
                uri: PathBuf::from("/some/path3"),
                duration: Duration::from_secs(u32::MAX.into()),
                title: Some(String::from("Funnier Track")),
            },
        ],
    };

    let expected_content = format!(
        "#EXTM3U
#EXTINF:67
/some/path1
#EXTINF:0,Fun Track
/some/path2
#EXTINF:{},Funnier Track
/some/path3",
        u32::MAX
    );

    assert_eq!(playlist.clone().serialize(), expected_content);
    assert_eq!(parse_media_playlist(&expected_content), Ok(("", playlist)));
}
