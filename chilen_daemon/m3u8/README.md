# M3U8

Simple parsing and serializing library for M3U8 media playlists.

The parser implemented in this library is limited in scope to what is needed for Chilen to function,
and is therefore not fully compliant with the
[HTTPS Live Streaming standard](https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-22).

# Examples
```rust
# use std::{time::Duration, path::PathBuf};
use m3u8::{MediaSegment, MediaPlaylist, parser::parse_media_playlist};
let mut pl = MediaPlaylist::default();
assert_eq!(pl.clone().serialize(), String::from("#EXTM3U"));

pl.segments.push( MediaSegment {
    uri: PathBuf::from("/test/path1"),
    duration: Duration::from_secs(69),
    title: None,
});
pl.segments.push( MediaSegment {
    uri: PathBuf::from("/test/path2"),
    duration: Duration::from_secs(420),
    title: Some(String::from("Example Title")),
});
pl.segments.push( MediaSegment {
    uri: PathBuf::from("/test/path3"),
    duration: Duration::from_secs(67),
    title: Some(String::from("Hello!")),
});
let serialized = pl.clone().serialize();
assert_eq!(parse_media_playlist(&serialized), Ok(("", pl)));
assert_eq!(serialized, String::from("#EXTM3U
#EXTINF:69
/test/path1
#EXTINF:420,Example Title
/test/path2
#EXTINF:67,Hello!
/test/path3"));
```
