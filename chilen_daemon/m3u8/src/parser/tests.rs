use crate::parser::{self, Line, ParsedTag};

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
fn non_newline_whitespace() {
    assert_eq!(parser::non_newline_whitespace(" "), Ok(("", " ")));
    assert_eq!(parser::non_newline_whitespace("\n"), Ok(("\n", "")));
    assert_eq!(parser::non_newline_whitespace("\t  \n"), Ok(("\n", "\t  ")));
    assert_eq!(parser::non_newline_whitespace("\n   "), Ok(("\n   ", "")));
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
    assert_eq!(parser::parse_line(i), Ok(("", Line::Path(i.to_string()))));
}

#[test]
fn parse_lines() {
    let expected = vec![
        Line::Tag("#EXTM3U".to_string()),
        Line::Comment("#Test comment".to_string()),
        Line::Tag("#EXTINF:67, Artist - Track".to_string()),
        Line::Path("/some/nonexistent/path".to_string()),
        Line::Tag("#EXTINF:24310".to_string()),
        Line::Path("/doesnt/matter".to_string()),
        Line::Tag("#EXTINF:10123, AAAAAA".to_string()),
        Line::Path("./AAAAAAAA".to_string()),
        Line::Path("/some/other/path/wihtout/extinf".to_string()),
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
fn tags() {}
