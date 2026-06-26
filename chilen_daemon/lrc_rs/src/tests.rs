use std::{sync::LazyLock, time::Duration};

use crate::{LRCTool, LineTag, LyricsAccess, SegmentTag, SyncedLyrics};

static PARSED_EXAMPLE: LazyLock<SyncedLyrics> = LazyLock::new(|| SyncedLyrics {
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
            // I can't just do `Duration::from_secs_f32(<timestamp> + <offset>)` due to float drift
            timestamp: Duration::from_secs_f32(12.1) + Duration::from_millis(100),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs_f32(12.1) + Duration::from_millis(100),
                content: "Hello, this is an example line that will appear at 12.1s".to_string(),
            }],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(16.7) + Duration::from_millis(100),
            segments: vec![SegmentTag {
                timestamp: Duration::from_secs_f32(16.7) + Duration::from_millis(100),
                content: "You can also trim them numbers and it still works".to_string(),
            }],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(22.0) + Duration::from_millis(100),
            segments: vec![
                SegmentTag {
                    timestamp: Duration::from_secs_f32(22.5) + Duration::from_millis(100),
                    content: "Line segments ".to_string(),
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(23.9) + Duration::from_millis(100),
                    content: "can also have ".to_string(),
                },
                SegmentTag {
                    timestamp: Duration::from_secs_f32(25.1) + Duration::from_millis(100),
                    content: "timestamps :)".to_string(),
                },
            ],
        },
        LineTag {
            timestamp: Duration::from_secs_f32(28.8) + Duration::from_millis(100),
            segments: Vec::new(),
        },
    ],
});

#[cfg(feature = "parser")]
#[test]
fn parse() {
    assert_eq!(
        SyncedLyrics::parse(include_str!("../assets/example.lrc")),
        Ok(PARSED_EXAMPLE.clone())
    );
}

#[test]
fn to_unsynced() {
    let unsynced = include_str!("../assets/example-w-out-sync.txt");
    assert_eq!(
        PARSED_EXAMPLE.clone().to_unsynced().to_string(),
        unsynced.to_string()
    );
}
