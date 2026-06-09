use std::time::Duration;

use nom::error::ErrorKind;

use crate::{Playlist, Track};

// TODO: All tests that serialize the playlist should also test if the serialized playlist is valid
// (deserialize it)

#[test]
fn serialize_empty_playlist() {
    assert_eq!(Playlist::default().serialize(), String::from("#EXTM3U"));
}

#[test]
fn serialize_with_playlists() {
    let playlist = Playlist {
        tracks: vec![
            Track {
                uri: String::from("/some/path1"),
                duration: Duration::from_secs(67),
                title: None,
            },
            Track {
                uri: String::from("/some/path2"),
                duration: Duration::from_secs(0),
                title: Some(String::from("Fun Track")),
            },
            Track {
                uri: String::from("/some/path3"),
                duration: Duration::from_secs(u64::MAX),
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
        u64::MAX
    );

    assert_eq!(playlist.serialize(), expected_content);
}

#[test]
fn empty_file_deserialization_fails() {
    assert_eq!(
        Playlist::deserialize("").unwrap_err(),
        nom::Err::Error(nom::error::Error::new("", ErrorKind::Tag))
    );
}

#[test]
fn extm3u_w_trailing_junk_fails() {
    let junk = "aslkfdjafd";
    let contents = String::from("#EXTM3U") + junk;
    assert_eq!(
        Playlist::deserialize(contents.as_str()).unwrap_err(),
        nom::Err::Error(nom::error::Error::new(contents.as_str(), ErrorKind::Tag))
    );
}

#[test]
fn comments_work() {
    let contents = String::from(
        "#EXTM3U
#HELLO!, This is a comment
#I always start with a '#' with 'EXT' after it",
    );
    assert_eq!(
        Playlist::deserialize(contents.as_str()).unwrap(),
        ("", Playlist::default())
    );
}

#[test]
fn test_ext_x_version() {
    let contents = String::from(
        "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-VERSION:1",
    );
    Playlist::deserialize(contents.as_str()).unwrap_err();
    let contents = String::from(
        "#EXTM3U
#EXT-X-VERSION:3",
    );
    Playlist::deserialize(contents.as_str()).unwrap();
}

#[test]
fn unsupported_tags_ignored() {}

// #[test]
// fn deserialize_m3u8_tag() {
//     Playlist::deserialize(b"#EXTM3Uaslkfdjafd").unwrap_err();
//     assert_eq!(
//         Playlist::deserialize(b"#EXTM3U"),
//         Ok((&b""[..], Playlist::default()))
//     );
//     let i = b"aslkjfdalksdf";
//     assert_eq!(
//         Playlist::deserialize(i),
//         Err(nom::Err::Error(nom::error::Error::new(
//             &i[..],
//             nom::error::ErrorKind::Tag
//         )))
//     );
//     let i = b"";
//     assert_eq!(
//         Playlist::deserialize(i),
//         Err(nom::Err::Error(nom::error::Error::new(
//             &i[..],
//             nom::error::ErrorKind::Tag
//         )))
//     );
// }
