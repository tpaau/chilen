use crate::data::music_lib::{Playlist, Track};

#[test]
fn playlist_track_removal() {
    let tracks = Track::unique_tracks(10);
    let tracks_cloned = tracks.clone();
    let mut playlist = Playlist {
        name: "Test".to_string(),
        tracks: tracks_cloned,
    };
    playlist.remove_tracks(vec![3, 5, 8]).unwrap();
    assert_eq!(playlist.tracks[0], tracks[0]);
    assert_eq!(playlist.tracks[1], tracks[1]);
    assert_eq!(playlist.tracks[2], tracks[2]);
    assert_eq!(playlist.tracks[3], tracks[4]);
    assert_eq!(playlist.tracks[4], tracks[6]);
    assert_eq!(playlist.tracks[5], tracks[7]);
    assert_eq!(playlist.tracks[6], tracks[9]);
}
