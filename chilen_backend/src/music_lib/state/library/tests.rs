use std::sync::Arc;

use icu::collator::{Collator, options::CollatorOptions};

use crate::{
    music_lib::{
        MusicLibrary,
        sorting::{COLLATOR, sort_tracks_in_album},
        state::Track,
    },
    testing_init_config,
};

#[test]
fn album_track_sorting() {
    testing_init_config();
    let mut tracks = Track::unique_tracks(5);
    tracks[0].track = Some(4);
    tracks[1].track = Some(1);
    tracks[2].title = Some("aaa".to_string().into());
    tracks[3].track = Some(2);
    tracks[4].title = Some("bbb".to_string().into());
    let mut tracks: Vec<_> = tracks.into_iter().map(Arc::new).collect();
    let tracks_unsorted = tracks.clone();

    let config = crate::get_config();
    let collator = Arc::new(
        Collator::try_new(config.locale.clone().into(), CollatorOptions::default()).unwrap(),
    );
    *COLLATOR.write().unwrap() = Some(collator);

    sort_tracks_in_album(&mut tracks);
    assert_eq!(
        &tracks,
        &[
            tracks_unsorted[1].clone(),
            tracks_unsorted[3].clone(),
            tracks_unsorted[0].clone(),
            tracks_unsorted[2].clone(),
            tracks_unsorted[4].clone()
        ]
    )
}

#[test]
fn genre_without_album_or_artist_does_not_panic() {
    testing_init_config();

    let mut track = Track::unique_tracks(1).remove(0);
    track.genres = Some(["Rock".to_string().into()].into());
    track.artists = Some(["Someone".to_string().into()].into());
    let lib = MusicLibrary::new_testing(vec![track]);
    assert_eq!(lib.genres.len(), 1);
    assert!(lib.genres[0].albums.is_empty());

    let mut track = Track::unique_tracks(1).remove(0);
    track.genres = Some(["Rock".to_string().into()].into());
    track.album = Some("Album".into());
    let lib = MusicLibrary::new_testing(vec![track]);
    assert_eq!(lib.genres.len(), 1);
    assert!(lib.genres[0].artists.is_empty());
}
