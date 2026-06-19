use std::{sync::LazyLock, time::Duration};

use nom::{Finish, error::ErrorKind};
use nom_language::error::{VerboseError, VerboseErrorKind, convert_error};

use crate::{
    MediaPlaylist, MediaSegment,
    parser::{
        self, Extinf, IGNORED_TAGS, IGNORED_TAGS_WITH_VALUES, Line, MULTIVARIANT_TAGS, SplitTag,
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
}

#[test]
fn newline_or_end() {
    assert_eq!(parser::newline_or_end("\n"), Ok(("", "\n")));
    assert_eq!(parser::newline_or_end(""), Ok(("", "")));
    assert_eq!(parser::newline_or_end("\naaaa\n"), Ok(("aaaa\n", "\n")));
    assert_eq!(
        parser::newline_or_end("aa"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![("aa", VerboseErrorKind::Nom(ErrorKind::TakeWhile1))]
        }))
    );
    assert_eq!(
        parser::newline_or_end("aa\n"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![("aa\n", VerboseErrorKind::Nom(ErrorKind::TakeWhile1))]
        }))
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
    assert_eq!(parser::opt_whitespace(""), Ok(("", "")));
}

#[test]
fn opt_split_whitespace() {
    assert_eq!(
        parser::split_tag_value("#SOME_TAG:67"),
        SplitTag {
            tag: "#SOME_TAG",
            value: Some("67")
        }
    );
    assert_eq!(
        parser::split_tag_value("aslkfdjlaskfd"),
        SplitTag {
            tag: "aslkfdjlaskfd",
            value: None
        }
    );
    assert_eq!(
        parser::split_tag_value(""),
        SplitTag {
            tag: "",
            value: None
        }
    );
    assert_eq!(
        parser::split_tag_value(":a"),
        SplitTag {
            tag: "",
            value: Some("a")
        }
    );
    assert_eq!(
        parser::split_tag_value("aaa:test\ntest"),
        SplitTag {
            tag: "aaa",
            value: Some("test\ntest")
        }
    );
}

#[test]
fn parse_f64() {
    assert_eq!(parser::parse_f64("128,aaaa"), Ok((",aaaa", 128.0)));
    assert_eq!(
        parser::parse_f64("test"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![
                ("test", VerboseErrorKind::Nom(ErrorKind::TakeWhile1)),
                ("test", VerboseErrorKind::Context("parse_f64"))
            ]
        }))
    );
}

#[test]
fn parse_extinf_value() {
    assert_eq!(
        parser::parse_extinf_value("67, Example Artist, Example Title\nhello"),
        Ok((
            "\nhello",
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
    assert_eq!(
        parser::parse_extinf_value("aa,aaa"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![
                ("aa,aaa", VerboseErrorKind::Nom(ErrorKind::TakeWhile1)),
                ("aa,aaa", VerboseErrorKind::Context("parse_f64")),
                ("aa,aaa", VerboseErrorKind::Context("parse_extinf_value"))
            ]
        }))
    );
    assert_eq!(
        parser::parse_extinf_value(",hello"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![
                (",hello", VerboseErrorKind::Nom(ErrorKind::TakeWhile1)),
                (",hello", VerboseErrorKind::Context("parse_f64")),
                (",hello", VerboseErrorKind::Context("parse_extinf_value"))
            ]
        }))
    );
    assert_eq!(
        parser::parse_extinf_value("1984,"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![("1984,", VerboseErrorKind::Context("parse_extinf_value"))]
        }))
    );
    assert_eq!(
        parser::parse_extinf_value("666,\nhello"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![(
                "666,\nhello",
                VerboseErrorKind::Context("parse_extinf_value")
            )]
        }))
    );
}

#[test]
fn parse_tag() {
    assert_eq!(parser::parse_tag("#EXTM3U"), Ok(("", Line::Tag("#EXTM3U"))));
    assert_eq!(
        parser::parse_tag("#EXTM3U\n"),
        Ok(("", Line::Tag("#EXTM3U")))
    );
    assert_eq!(
        parser::parse_tag("#EXTM3U\n\n"),
        Ok(("", Line::Tag("#EXTM3U")))
    );
    assert_eq!(
        parser::parse_tag("#EXTM3U\n\n\n#COMMENT"),
        Ok(("#COMMENT", Line::Tag("#EXTM3U")))
    );
    assert_eq!(
        parser::parse_tag("\n#EXTM3U\n\n\n#COMMENT"),
        Ok(("#COMMENT", Line::Tag("#EXTM3U")))
    );
    assert_eq!(
        parser::parse_tag("#AAA"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![
                ("#AAA", VerboseErrorKind::Nom(ErrorKind::Tag)),
                ("#AAA", VerboseErrorKind::Context("parse_tag"))
            ]
        }))
    );
}

#[test]
fn parse_comment() {
    assert_eq!(
        parser::parse_comment("#COMMENT"),
        Ok(("", Line::Comment("#COMMENT")))
    );
    assert_eq!(
        parser::parse_comment("#COMMENT\n"),
        Ok(("", Line::Comment("#COMMENT")))
    );
    assert_eq!(
        parser::parse_comment("#COMMENT\n\n\n"),
        Ok(("", Line::Comment("#COMMENT")))
    );
    assert_eq!(
        parser::parse_comment("#COMMENT\n#ANOTHER COMMENT"),
        Ok(("#ANOTHER COMMENT", Line::Comment("#COMMENT")))
    );
    assert_eq!(
        parser::parse_comment("#COMMENT\n\n\n\n#ANOTHER COMMENT"),
        Ok(("#ANOTHER COMMENT", Line::Comment("#COMMENT")))
    );
    assert_eq!(
        parser::parse_comment("\n#TEST"),
        Ok(("", Line::Comment("#TEST")))
    );
    assert_eq!(
        parser::parse_comment("#EXTEST"),
        Ok(("", Line::Comment("#EXTEST")))
    );
    assert_eq!(
        parser::parse_comment("AAAA"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![
                ("AAAA", VerboseErrorKind::Nom(ErrorKind::Tag)),
                ("AAAA", VerboseErrorKind::Context("parse_comment"))
            ]
        }))
    );
}

#[test]
fn parse_segment() {
    let i = "/some/path";
    assert_eq!(parser::parse_segment(i), Ok(("", Line::Segment(i))));
}

#[test]
fn parse_line() {
    // TEST: Test parse_line
}

#[test]
fn parse_lines() {
    let expected = vec![
        Line::Tag("#EXTM3U"),
        Line::Comment("#Test comment"),
        Line::Tag("#EXTINF:67, Title"),
        Line::Segment("/some/nonexistent/path"),
        Line::Tag("#EXTINF:24310"),
        Line::Segment("/doesnt/matter"),
        Line::Tag("#EXTINF:10123, AAAAAA"),
        Line::Segment("./AAAAAAAA"),
        Line::Tag("#EXTINF:10923, ZZZ, AAA"),
        Line::Segment("/some/other/path"),
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
fn extm3u_tag() {
    assert_eq!(parser::extm3u_tag("#EXTM3U"), Ok(("", "#EXTM3U")));
    assert_eq!(parser::extm3u_tag("#EXTM3U\n\n\n"), Ok(("", "#EXTM3U")));
    assert_eq!(parser::extm3u_tag("#EXTM3U\n\na"), Ok(("a", "#EXTM3U")));
    assert_eq!(
        parser::extm3u_tag("#EXTM3"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![
                ("#EXTM3", VerboseErrorKind::Nom(ErrorKind::Tag)),
                ("#EXTM3", VerboseErrorKind::Context("extm3u_tag"))
            ]
        }))
    );
    assert_eq!(
        parser::extm3u_tag("#EXTM3Ua"),
        Err(nom::Err::Error(VerboseError {
            errors: vec![("a", VerboseErrorKind::Nom(ErrorKind::TakeWhile1))]
        }))
    );
}

#[test]
fn multivariant_tags_fail() {
    let base = include_str!("../../assets/simple.m3u8").to_string();
    for tag in MULTIVARIANT_TAGS {
        let content = base.clone() + tag;
        assert_eq!(
            parser::parse_media_playlist(content.as_str()),
            Err(nom::Err::Error(VerboseError {
                errors: vec![
                    (*tag, VerboseErrorKind::Context("no_multivariant_tag")),
                    (
                        content.as_str(),
                        VerboseErrorKind::Context("parse_media_playlist")
                    )
                ]
            }))
        );
    }
}

#[test]
fn missing_extm3u_fails() {
    let simple = include_str!("../../assets/simple.m3u8")
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        parser::parse_media_playlist(&simple),
        Err(nom::Err::Error(VerboseError {
            errors: vec![
                (simple.as_str(), VerboseErrorKind::Nom(ErrorKind::Tag)),
                (simple.as_str(), VerboseErrorKind::Context("extm3u_tag")),
                (
                    simple.as_str(),
                    VerboseErrorKind::Context("parse_media_playlist")
                )
            ]
        }))
    );
    // panic!(
    //     "{}",
    //     convert_error(
    //         simple.as_str(),
    //         parser::parse_media_playlist(&simple).finish().unwrap_err()
    //     )
    // );
}

#[test]
fn duplicate_unique_ignored_tags_fail() {}

#[test]
fn ignored_tags_with_missing_values_fail() {}

#[test]
fn entry_with_no_extinf_fails() {}

#[test]
fn unknown_tags_fail() {}

#[test]
fn parsing_works() {
    assert_eq!(
        parse_media_playlist(include_str!("../../assets/simple.m3u8")),
        Ok(("", SIMPLE_PLAYLIST.clone()))
    );
}
