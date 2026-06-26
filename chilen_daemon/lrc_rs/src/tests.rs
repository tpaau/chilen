use std::time::Duration;

use crate::{LRCTool, LineTag, Lyrics, LyricsAccess, SegmentTag, SyncedLyrics, duration_offset};

#[test]
fn parse() {
    let expected = Ok(Lyrics::Synced(Box::new(SyncedLyrics {
        title: Some("example".to_string()),
        artist: Some("tpaau".to_string()),
        album: Some("lrc_rs".to_string()),
        author: Some("aaa".to_string()),
        lyricist: Some("help".to_string()),
        length: Some(Duration::from_secs_f32(25217.0)),
        file_author: Some("Helix".to_string()),
        tool: Some(LRCTool {
            name: "me1".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        comments: vec!["Hello, this is a comment".to_string()],
        lines: vec![
            LineTag {
                timestamp: duration_offset(Duration::from_secs_f32(12.1), 100).unwrap(),
                segments: vec![SegmentTag {
                    timestamp: duration_offset(Duration::from_secs_f32(12.1), 100).unwrap(),
                    content: "Hello, this is an example line that will appear at 12.1s".to_string(),
                }],
            },
            LineTag {
                timestamp: duration_offset(Duration::from_secs_f32(16.7), 100).unwrap(),
                segments: vec![SegmentTag {
                    timestamp: duration_offset(Duration::from_secs_f32(16.7), 100).unwrap(),
                    content: "You can also trim them numbers and it still works".to_string(),
                }],
            },
            LineTag {
                timestamp: duration_offset(Duration::from_secs_f32(22.0), 100).unwrap(),
                segments: vec![
                    SegmentTag {
                        timestamp: duration_offset(Duration::from_secs_f32(22.5), 100).unwrap(),
                        content: "Line segments ".to_string(),
                    },
                    SegmentTag {
                        timestamp: duration_offset(Duration::from_secs_f32(23.9), 100).unwrap(),
                        content: "can also have ".to_string(),
                    },
                    SegmentTag {
                        timestamp: duration_offset(Duration::from_secs_f32(25.1), 100).unwrap(),
                        content: "timestamps :)".to_string(),
                    },
                ],
            },
            LineTag {
                timestamp: duration_offset(Duration::from_secs_f32(28.8), 100).unwrap(),
                segments: Vec::new(),
            },
        ],
    })));
    assert_eq!(
        Lyrics::parse(include_str!("../assets/example.lrc")),
        expected
    );
}

#[test]
fn to_unsynced() {
    let (parsed_unsynced, _) =
        Lyrics::parse(include_str!("../assets/example-w-out-sync.txt")).unwrap_err();
    let parsed_synced = Lyrics::parse(include_str!("../assets/example.lrc")).unwrap();
    if let Lyrics::Synced(synced) = parsed_synced {
        assert_eq!(
            Lyrics::Unsynced(synced.to_unsynced().to_string()),
            parsed_unsynced
        );
    } else {
        panic!()
    }
}
