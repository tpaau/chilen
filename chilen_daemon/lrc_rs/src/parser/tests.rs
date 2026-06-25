use std::time::Duration;

use nom::{Parser, error::ErrorKind, multi::many0};

use crate::parser::{self, IDTag, Line, TimestampedSegment, TimestampedTag};

#[test]
fn newline_or_end() {
    assert_eq!(parser::newline_or_end("\n"), Ok(("", "\n")));
    assert_eq!(parser::newline_or_end(""), Ok(("", "")));
    assert_eq!(parser::newline_or_end("\naaaa\n"), Ok(("aaaa\n", "\n")));
    assert_eq!(
        parser::newline_or_end("aa"),
        Err(nom::Err::Error(nom::error::Error::new(
            "aa",
            ErrorKind::Eof
        )))
    );
    assert_eq!(
        parser::newline_or_end("aa\n"),
        Err(nom::Err::Error(nom::error::Error::new(
            "aa\n",
            ErrorKind::Eof
        )))
    );
}

#[test]
fn till_a2_tag_or_end() {
    assert_eq!(
        parser::till_a2_tag("aaaaaaaaaa <"),
        Ok(("<", "aaaaaaaaaa "))
    );
    assert_eq!(parser::till_a2_tag("hello"), Ok(("", "hello")));
    assert_eq!(parser::till_a2_tag(""), Ok(("", "")));
}

#[test]
fn timestamp() {
    assert_eq!(
        parser::timestamp("11:53.20"),
        Ok(("", Duration::from_secs_f32(713.2)))
    );
    assert_eq!(parser::timestamp("00:00.00"), Ok(("", Duration::default())));
    assert_eq!(
        parser::timestamp("01:21"),
        Ok(("", Duration::from_secs(81)))
    );
    assert_eq!(
        parser::timestamp(""),
        Err(nom::Err::Error(nom::error::Error::new(
            "",
            ErrorKind::Float
        )))
    );
    assert_eq!(
        parser::timestamp("01::21.5"),
        Err(nom::Err::Error(nom::error::Error::new(
            ":21.5",
            ErrorKind::Float
        )))
    );
}

#[test]
fn int() {
    assert_eq!(parser::unsigned_int("420"), Ok(("", 420)));
    assert_eq!(parser::unsigned_int("69.0"), Ok((".0", 69)));
    assert_eq!(
        parser::unsigned_int("-67"),
        Err(nom::Err::Error(nom::error::Error::new(
            "-67",
            ErrorKind::Digit
        )))
    );
}

#[test]
fn offset() {
    assert_eq!(parser::offset("+1337"), Ok(("", 1337)));
    assert_eq!(parser::offset("-1984"), Ok(("", -1984)));
    assert_eq!(
        parser::offset("+2137.1"),
        Err(nom::Err::Error(nom::error::Error::new(
            ".1",
            ErrorKind::Eof
        )))
    );
    assert_eq!(
        parser::offset("+69a"),
        Err(nom::Err::Error(nom::error::Error::new("a", ErrorKind::Eof)))
    );
}

#[test]
fn standard_timestamp() {
    assert_eq!(
        parser::standard_timestamp("[11:53.20]"),
        Ok(("", Duration::from_secs_f32(713.2)))
    );
    assert_eq!(
        parser::standard_timestamp("11:01.00]"),
        Err(nom::Err::Error(nom::error::Error::new(
            "11:01.00]",
            ErrorKind::Char
        )))
    );
    assert_eq!(
        parser::standard_timestamp("[11:01.00"),
        Err(nom::Err::Error(nom::error::Error::new("", ErrorKind::Char)))
    );
}

#[test]
fn a2_timestamp() {
    assert_eq!(
        parser::a2_timestamp("<11:53.20>"),
        Ok(("", Duration::from_secs_f32(713.2)))
    );
    assert_eq!(
        parser::a2_timestamp("11:01.00>"),
        Err(nom::Err::Error(nom::error::Error::new(
            "11:01.00>",
            ErrorKind::Char
        )))
    );
    assert_eq!(
        parser::a2_timestamp("<11:01.00"),
        Err(nom::Err::Error(nom::error::Error::new("", ErrorKind::Char)))
    );
}

#[test]
fn a2_tag() {
    assert_eq!(
        parser::a2_tag("<69:00.42> a"),
        Ok((
            "",
            parser::TimestampedSegment {
                timestamp: Duration::from_secs_f32(4140.42),
                content: "a"
            }
        ))
    );
    assert_eq!(
        many0(parser::a2_tag).parse("<00:00.00> Hello <00:01.00> World!"),
        Ok((
            "",
            vec![
                parser::TimestampedSegment {
                    timestamp: Duration::default(),
                    content: "Hello "
                },
                parser::TimestampedSegment {
                    timestamp: Duration::from_secs_f32(1.0),
                    content: "World!"
                }
            ]
        ))
    );
    assert_eq!(
        parser::a2_tag("<00:00.00> Hello <00:01.00> World!"),
        Ok((
            "<00:01.00> World!",
            parser::TimestampedSegment {
                timestamp: Duration::default(),
                content: "Hello "
            }
        ))
    );
    assert_eq!(
        parser::a2_tag("<00:12.00 Hello!"),
        Err(nom::Err::Error(nom::error::Error::new(
            " Hello!",
            ErrorKind::Char
        )))
    );
    assert_eq!(
        parser::a2_tag("00:12.00> Hello!"),
        Err(nom::Err::Error(nom::error::Error::new(
            "00:12.00> Hello!",
            ErrorKind::Char
        )))
    );
    assert_eq!(
        parser::a2_tag("<00:12.00> Hello!\n"),
        Ok((
            "\n",
            parser::TimestampedSegment {
                timestamp: Duration::from_secs_f32(12.0),
                content: "Hello!"
            }
        ))
    );
}

#[test]
fn comment() {
    assert_eq!(
        parser::comment("[#: I am a comment]"),
        Ok(("", "I am a comment"))
    );
    assert_eq!(
        parser::comment("[#: Hello, this is a comment]"),
        Ok(("", "Hello, this is a comment"))
    );
    assert_eq!(
        parser::comment("[#: Hello, this is a comment]\n"),
        Ok(("", "Hello, this is a comment"))
    );
    assert_eq!(
        parser::comment("[#: Hello, this is a comment]\na"),
        Ok(("a", "Hello, this is a comment"))
    );
    assert_eq!(
        parser::comment("[#: Hello, this is a comment] \na"),
        Ok(("a", "Hello, this is a comment"))
    );
    assert_eq!(
        parser::comment("[#: I am a comment]  "),
        Ok(("", "I am a comment"))
    );
    assert_eq!(
        parser::comment("[a: I am a comment]"),
        Err(nom::Err::Error(nom::error::Error::new(
            "a: I am a comment]",
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        parser::comment("[# I am]"),
        Err(nom::Err::Error(nom::error::Error::new(
            "# I am]",
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        parser::comment("#: therefore I think]"),
        Err(nom::Err::Error(nom::error::Error::new(
            "#: therefore I think]",
            ErrorKind::Char
        )))
    );
    assert_eq!(
        parser::comment("[#: So yeah"),
        Err(nom::Err::Error(nom::error::Error::new("", ErrorKind::Char)))
    );
}

#[test]
fn id_tag() {
    assert_eq!(
        parser::id_tag("[by: Some program]"),
        Ok((
            "",
            IDTag {
                key: "by",
                value: "Some program"
            }
        ))
    );
    assert_eq!(
        parser::id_tag("[by: Some program]\n"),
        Ok((
            "",
            IDTag {
                key: "by",
                value: "Some program"
            }
        ))
    );
    assert_eq!(
        parser::id_tag("[by: Some program]  "),
        Ok((
            "",
            IDTag {
                key: "by",
                value: "Some program"
            }
        ))
    );
    assert_eq!(
        parser::id_tag("[by: Some program]  \n"),
        Ok((
            "",
            IDTag {
                key: "by",
                value: "Some program"
            }
        ))
    );
    assert_eq!(
        parser::id_tag("[by: Some program]a"),
        Err(nom::Err::Error(nom::error::Error::new("a", ErrorKind::Eof)))
    );
    assert_eq!(
        parser::id_tag("by: Hello World!]"),
        Err(nom::Err::Error(nom::error::Error::new(
            "by: Hello World!]",
            ErrorKind::Char
        )))
    );
    assert_eq!(
        parser::id_tag("[by: Hello World!"),
        Err(nom::Err::Error(nom::error::Error::new("", ErrorKind::Char)))
    );
}

#[test]
fn line_with_a2() {
    let expected = Ok((
        "",
        TimestampedTag {
            timestamp: Duration::from_secs_f32(1.1),
            segments: vec![
                TimestampedSegment {
                    timestamp: Duration::from_secs_f32(1.1),
                    content: "Hello ",
                },
                TimestampedSegment {
                    timestamp: Duration::from_secs_f32(2.0),
                    content: "World!",
                },
            ],
        },
    ));
    assert_eq!(
        parser::line_with_a2("[00:01.10]<00:01.10> Hello <00:02.00> World!"),
        expected
    );
    assert_eq!(
        parser::line_with_a2("[00:01.10] <00:01.10> Hello <00:02.00> World!"),
        expected
    );
    assert_eq!(
        parser::line_with_a2("[00:01.10] <00:01.10> Hello <00:02.00> World!\n"),
        expected
    );
    assert_eq!(
        parser::line_with_a2(
            "[00:22.00] <00:22.50> Line segments <00:23.90> can also have <00:25.10> timestamps :)"
        ),
        Ok((
            "",
            TimestampedTag {
                timestamp: Duration::from_secs_f32(22.0),
                segments: vec![
                    TimestampedSegment {
                        timestamp: Duration::from_secs_f32(22.5),
                        content: "Line segments ",
                    },
                    TimestampedSegment {
                        timestamp: Duration::from_secs_f32(23.9),
                        content: "can also have ",
                    },
                    TimestampedSegment {
                        timestamp: Duration::from_secs_f32(25.1),
                        content: "timestamps :)",
                    },
                ],
            }
        ),)
    );
}

#[test]
fn standard_line() {
    let expected = Ok((
        "",
        TimestampedTag {
            timestamp: Duration::from_secs_f32(12.3),
            segments: vec![TimestampedSegment {
                timestamp: Duration::from_secs_f32(12.3),
                content: "Hello, I am a standard LRC file with no A2 tags!",
            }],
        },
    ));
    assert_eq!(
        parser::standard_line("[00:12.30]Hello, I am a standard LRC file with no A2 tags!"),
        expected
    );
    assert_eq!(
        parser::standard_line("[00:12.30] Hello, I am a standard LRC file with no A2 tags!"),
        expected
    );
    assert_eq!(
        parser::standard_line(
            "[00:12.30]   Hello, I am a standard LRC file with no A2 tags!\n\n\n"
        ),
        expected
    );
    assert_eq!(
        parser::standard_line("[0:15.9]"),
        Ok((
            "",
            TimestampedTag {
                timestamp: Duration::from_secs_f32(15.9),
                segments: vec![TimestampedSegment {
                    timestamp: Duration::from_secs_f32(15.9),
                    content: ""
                }]
            }
        ))
    );
}

#[test]
fn parse() {
    let expected = Ok(vec![
        Line::ID(IDTag {
            key: "ti",
            value: "example",
        }),
        Line::ID(IDTag {
            key: "ar",
            value: "tpaau",
        }),
        Line::ID(IDTag {
            key: "al",
            value: "lrc_rs",
        }),
        Line::ID(IDTag {
            key: "au",
            value: "aaa",
        }),
        Line::ID(IDTag {
            key: "lr",
            value: "help",
        }),
        Line::ID(IDTag {
            key: "length",
            value: "420:17",
        }),
        Line::ID(IDTag {
            key: "by",
            value: "Helix",
        }),
        Line::ID(IDTag {
            key: "offset",
            value: "+100",
        }),
        Line::ID(IDTag {
            key: "tool",
            value: "me",
        }),
        Line::ID(IDTag {
            key: "re",
            value: "me1",
        }),
        Line::ID(IDTag {
            key: "ve",
            value: "1.0.0",
        }),
        Line::Comment("Hello, this is a comment"),
        Line::Tag(TimestampedTag {
            timestamp: Duration::from_secs_f32(12.1),
            segments: vec![TimestampedSegment {
                timestamp: Duration::from_secs_f32(12.1),
                content: "Hello, this is an example line that will appear at 12.1s",
            }],
        }),
        Line::Tag(TimestampedTag {
            timestamp: Duration::from_secs_f32(16.7),
            segments: vec![TimestampedSegment {
                timestamp: Duration::from_secs_f32(16.7),
                content: "You can also trim them numbers and it still works",
            }],
        }),
        Line::Tag(TimestampedTag {
            timestamp: Duration::from_secs_f32(22.0),
            segments: vec![
                TimestampedSegment {
                    timestamp: Duration::from_secs_f32(22.5),
                    content: "Line segments ",
                },
                TimestampedSegment {
                    timestamp: Duration::from_secs_f32(23.9),
                    content: "can also have ",
                },
                TimestampedSegment {
                    timestamp: Duration::from_secs_f32(25.1),
                    content: "timestamps :)",
                },
            ],
        }),
        Line::Tag(TimestampedTag {
            timestamp: Duration::from_secs_f32(28.8),
            segments: Vec::new(),
        }),
    ]);
    assert_eq!(
        parser::parse(include_str!("../../assets/example.lrc")),
        expected
    );
    assert_eq!(
        parser::parse(include_str!("../../assets/example-w-whitespace.lrc")),
        expected
    );
}
