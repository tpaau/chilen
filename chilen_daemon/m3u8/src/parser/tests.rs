use std::{sync::LazyLock, time::Duration};

use crate::{
    MediaPlaylist, MediaSegment,
    parser::{
        self, Extinf, IGNORED_TAGS, IGNORED_TAGS_WITH_VALUES, Line, MULTIVARIANT_TAGS, ParsedTag,
        UNIQUE_IGNORED_TAGS, parse_media_playlist,
    },
};

static SIMPLE_PLAYLIST: LazyLock<MediaPlaylist> = LazyLock::new(|| MediaPlaylist {
    segments: vec![
        MediaSegment {
            uri: "/some/nonexistent/path".to_string(),
            duration: Duration::from_secs(67),
            title: Some("Title".to_string()),
        },
        MediaSegment {
            uri: "/doesnt/matter".to_string(),
            duration: Duration::from_secs(24310),
            title: None,
        },
        MediaSegment {
            uri: "./AAAAAAAA".to_string(),
            duration: Duration::from_secs(10123),
            title: Some("AAAAAA".to_string()),
        },
        MediaSegment {
            uri: "/some/other/path".to_string(),
            duration: Duration::from_secs(10923),
            title: Some("ZZZ, AAA".to_string()),
        },
    ],
});

#[test]
fn tags_start_correctly() {
    let mut tags: Vec<&str> = Vec::new();
    for s in [
        IGNORED_TAGS,
        UNIQUE_IGNORED_TAGS,
        IGNORED_TAGS_WITH_VALUES,
        MULTIVARIANT_TAGS,
    ] {
        tags.extend_from_slice(s);
    }
    for tag in tags {
        eprintln!("{tag}");
        assert_eq!(&tag[..4], "#EXT");
    }
}

#[test]
fn tags_overlap_properly() {
    for tag in IGNORED_TAGS_WITH_VALUES {
        eprintln!("{tag}");
        assert!(IGNORED_TAGS.contains(tag) || UNIQUE_IGNORED_TAGS.contains(tag));
    }
    for tag in MULTIVARIANT_TAGS {
        eprintln!("{tag}");
        assert!(!IGNORED_TAGS.contains(tag) && !UNIQUE_IGNORED_TAGS.contains(tag));
    }
    for tag in IGNORED_TAGS {
        eprintln!("{tag}");
        assert!(!UNIQUE_IGNORED_TAGS.contains(tag));
    }
}

#[test]
fn all_tags_covered() {
    let tags = [
        "#EXT-X-VERSION",
        "#EXT-X-INDEPENDENT-SEGMENTS",
        "#EXT-X-START",
        "#EXT-X-DEFINE",
        "#EXT-X-TARGETDURATION",
        "#EXT-X-MEDIA-SEQUENCE",
        "#EXT-X-DISCONTINUITY-SEQUENCE",
        // "#EXT-X-ENDLIST", // Handled by the parser
        "#EXT-X-PLAYLIST-TYPE",
        "#EXT-X-I-FRAMES-ONLY",
        "#EXT-X-PART-INF",
        "#EXT-X-SERVER-CONTROL",
        // "#EXTINF", // Handled by the parser
        "#EXT-X-BYTERANGE",
        "#EXT-X-DISCONTINUITY",
        "#EXT-X-KEY",
        "#EXT-X-MAP",
        "#EXT-X-PROGRAM-DATE-TIME",
        // "#EXT-X-GAP", // Handled by the parser
        "#EXT-X-BITRATE",
        "#EXT-X-PART",
        "#EXT-X-DATERANGE",
        "#EXT-X-SKIP",
        "#EXT-X-PRELOAD-HINT",
        "#EXT-X-RENDITION-REPORT",
        "#EXT-X-MEDIA",
        "#EXT-X-STREAM-INF",
        "#EXT-X-I-FRAME-STREAM-INF",
        "#EXT-X-SESSION-DATA",
        "#EXT-X-SESSION-KEY",
        "#EXT-X-CONTENT-STEERING",
    ];
    for tag in tags {
        eprintln!("{tag}");
        assert!(
            IGNORED_TAGS.contains(&tag)
                || UNIQUE_IGNORED_TAGS.contains(&tag)
                || MULTIVARIANT_TAGS.contains(&tag)
        )
    }
}

#[test]
fn newline() {
    assert_eq!(parser::newline("\n\n\na"), Ok(("a", "\n\n\n")));
    assert_eq!(parser::newline("\nsome text\n"), Ok(("some text\n", "\n")));
    assert_eq!(
        parser::newline("\n\nsome\nfunny\ntext\n"),
        Ok(("some\nfunny\ntext\n", "\n\n"))
    );
    assert_eq!(
        parser::newline_or_end("7"),
        Err(nom::Err::Error(nom::error::Error::new(
            "7",
            nom::error::ErrorKind::TakeWhile1
        )))
    );
    assert_eq!(
        parser::newline_or_end("7\n"),
        Err(nom::Err::Error(nom::error::Error::new(
            "7\n",
            nom::error::ErrorKind::TakeWhile1
        )))
    );
}

#[test]
fn newline_or_end() {
    assert_eq!(parser::newline_or_end("\n"), Ok(("", "\n")));
    assert_eq!(parser::newline_or_end(""), Ok(("", "")));
    assert_eq!(parser::newline_or_end("\naaaa\n"), Ok(("aaaa\n", "\n")));
    assert_eq!(
        parser::newline_or_end("aa"),
        Err(nom::Err::Error(nom::error::Error::new(
            "aa",
            nom::error::ErrorKind::TakeWhile1
        )))
    );
    assert_eq!(
        parser::newline_or_end("aa\n"),
        Err(nom::Err::Error(nom::error::Error::new(
            "aa\n",
            nom::error::ErrorKind::TakeWhile1
        )))
    );
}

#[test]
fn opt_whitespace() {
    assert_eq!(
        parser::opt_whitespace("   a asdf   "),
        Ok(("a asdf   ", "   "))
    );
    assert_eq!(
        parser::opt_whitespace("no whitespace :("),
        Ok(("no whitespace :(", ""))
    );
}

#[test]
fn parse_tag_value() {
    assert_eq!(
        parser::parse_tag_value("#SOME_TAG:67"),
        Ok((
            "",
            ParsedTag {
                tag: "#SOME_TAG".to_string(),
                value: Some("67".to_string())
            }
        ))
    );
    assert_eq!(
        parser::parse_tag_value("aslkfdjlaskfd"),
        Ok((
            "",
            ParsedTag {
                tag: "aslkfdjlaskfd".to_string(),
                value: None
            }
        ))
    );
    assert_eq!(
        parser::parse_tag_value(""),
        Ok((
            "",
            ParsedTag {
                tag: "".to_string(),
                value: None
            }
        ))
    );
    assert_eq!(
        parser::parse_tag_value(":a"),
        Ok((
            "",
            ParsedTag {
                tag: "".to_string(),
                value: Some("a".to_string())
            }
        ))
    );
}

#[test]
fn parse_extinf_value() {
    assert_eq!(
        parser::parse_extinf_value("67, Example Artist, Example Title"),
        Ok((
            "",
            Extinf {
                duration: Duration::from_secs(67),
                title: Some("Example Artist, Example Title".to_string())
            }
        ))
    );
    assert_eq!(
        parser::parse_extinf_value("69"),
        Ok((
            "",
            Extinf {
                duration: Duration::from_secs(69),
                title: None,
            }
        ))
    );
    assert_eq!(
        parser::parse_extinf_value("420,                  Title"),
        Ok((
            "",
            Extinf {
                duration: Duration::from_secs(420),
                title: Some("Title".to_string()),
            }
        ))
    );
    assert_eq!(
        parser::parse_extinf_value(format!("{},Long Track", u32::MAX).as_str()),
        Ok((
            "",
            Extinf {
                duration: Duration::from_secs(u32::MAX.into()),
                title: Some("Long Track".to_string()),
            }
        ))
    );
    // TODO: Error cases
}

#[test]
fn parse_line_comments() {
    assert_eq!(
        parser::parse_line("#COMMENT"),
        Ok(("", Line::Comment("#COMMENT".to_string())))
    );
    assert_eq!(
        parser::parse_line("#COMMENT\n"),
        Ok(("", Line::Comment("#COMMENT".to_string())))
    );
    assert_eq!(
        parser::parse_line("#COMMENT\n\n\n"),
        Ok(("", Line::Comment("#COMMENT".to_string())))
    );
    assert_eq!(
        parser::parse_line("#COMMENT\n#ANOTHER COMMENT"),
        Ok(("#ANOTHER COMMENT", Line::Comment("#COMMENT".to_string())))
    );
    assert_eq!(
        parser::parse_line("#COMMENT\n\n\n\n#ANOTHER COMMENT"),
        Ok(("#ANOTHER COMMENT", Line::Comment("#COMMENT".to_string())))
    );
    assert_eq!(
        parser::parse_line("\n#TEST"),
        Ok(("", Line::Comment("#TEST".to_string())))
    );
}

#[test]
fn parse_line_tag() {
    assert_eq!(
        parser::parse_line("#EXTM3U"),
        Ok(("", Line::Tag("#EXTM3U".to_string())))
    );
    assert_eq!(
        parser::parse_line("#EXTM3U\n"),
        Ok(("", Line::Tag("#EXTM3U".to_string())))
    );
    assert_eq!(
        parser::parse_line("#EXTM3U\n\n"),
        Ok(("", Line::Tag("#EXTM3U".to_string())))
    );
    assert_eq!(
        parser::parse_line("#EXTM3U\n\n\n#COMMENT"),
        Ok(("#COMMENT", Line::Tag("#EXTM3U".to_string())))
    );
    assert_eq!(
        parser::parse_line("\n#EXTM3U\n\n\n#COMMENT"),
        Ok(("#COMMENT", Line::Tag("#EXTM3U".to_string())))
    );
}

#[test]
fn parse_line_path() {
    let i = "/some/path, ./another/path";
    assert_eq!(
        parser::parse_line(i),
        Ok(("", Line::Segment(i.to_string())))
    );
}

#[test]
fn parse_lines() {
    let expected = vec![
        Line::Tag("#EXTM3U".to_string()),
        Line::Comment("#Test comment".to_string()),
        Line::Tag("#EXTINF:67, Title".to_string()),
        Line::Segment("/some/nonexistent/path".to_string()),
        Line::Tag("#EXTINF:24310".to_string()),
        Line::Segment("/doesnt/matter".to_string()),
        Line::Tag("#EXTINF:10123, AAAAAA".to_string()),
        Line::Segment("./AAAAAAAA".to_string()),
        Line::Tag("#EXTINF:10923, ZZZ, AAA".to_string()),
        Line::Segment("/some/other/path".to_string()),
    ];
    assert_eq!(
        parser::parse_lines(include_str!("../../assets/simple.m3u8")),
        Ok(("", expected.clone()))
    );
    assert_eq!(
        parser::parse_lines(include_str!("../../assets/simple-w-whitespace.m3u8")),
        Ok(("", expected))
    );
}

#[test]
fn multivariant_tags_fail() {
    let base = include_str!("../../assets/simple.m3u8").to_string();
    for tag in MULTIVARIANT_TAGS {
        let pl = base.clone() + tag;
        // TODO: Playlist fails because it contains a multivariant tag
    }
}

#[test]
fn parsing_works() {
    assert_eq!(
        parse_media_playlist(include_str!("../../assets/simple.m3u8")),
        Ok(("", SIMPLE_PLAYLIST.clone()))
    );
}
