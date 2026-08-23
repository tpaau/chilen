use std::sync::Arc;

use icu::collator::{Collator, options::CollatorOptions};

use crate::{
    music_lib::state::{MusicLibrary, Track},
    testing_init_config,
};

#[test]
fn album_track_sorting() {
    testing_init_config();
    let mut tracks = Track::unique_tracks(5);
    tracks[0].track = Some(4);
    tracks[1].track = Some(1);
    tracks[2].title = Some("aaa".to_string());
    tracks[3].track = Some(2);
    tracks[4].title = Some("bbb".to_string());
    let mut tracks: Vec<_> = tracks.into_iter().map(Arc::new).collect();
    let tracks_unsorted = tracks.clone();

    let config = crate::get_config();
    let collator = Arc::new(
        Collator::try_new(config.locale.clone().into(), CollatorOptions::default()).unwrap(),
    );

    MusicLibrary::sort_tracks_chronologically(&mut tracks, Some(&collator));
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
