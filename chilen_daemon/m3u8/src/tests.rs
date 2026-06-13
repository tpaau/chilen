use std::time::Duration;

use crate::{MediaPlaylist, MediaSegment, parser::parse_media_playlist};

// TODO: All tests that serialize the playlist should also test if the serialized playlist is valid
// (deserialize it)

#[test]
fn serialize_empty_playlist() {
    assert_eq!(
        MediaPlaylist::default().serialize(),
        String::from("#EXTM3U")
    );
}

#[test]
fn serialize_with_playlists() {
    let playlist = MediaPlaylist {
        segments: vec![
            MediaSegment {
                uri: String::from("/some/path1"),
                duration: Duration::from_secs(67),
                title: None,
            },
            MediaSegment {
                uri: String::from("/some/path2"),
                duration: Duration::from_secs(0),
                title: Some(String::from("Fun Track")),
            },
            MediaSegment {
                uri: String::from("/some/path3"),
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

#[test]
fn empty_file_deserialization_fails() {}

#[test]
fn extm3u_w_trailing_junk_fails() {}

#[test]
fn comments_work() {}

#[test]
fn test_ext_x_version() {}

#[test]
fn unsupported_tags_ignored() {}

#[test]
fn deserialize_m3u8_tag() {}
