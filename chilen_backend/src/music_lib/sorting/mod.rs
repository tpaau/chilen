//! Sorting helper functions for library items.

use std::sync::{Arc, RwLock};

use icu::collator::CollatorBorrowed;

use crate::music_lib::{AsAlbum, AsArtist, AsGenre, AsPlaylist, AsTrack};

pub(crate) static COLLATOR: RwLock<Option<Arc<CollatorBorrowed<'_>>>> = RwLock::new(None);

/// Sorts tracks alphabetically by titles.
pub fn sort_tracks<T: AsTrack>(tracks: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|t1, t2| {
        let t1 = t1.as_track().title.as_deref().unwrap_or_default();
        let t2 = t2.as_track().title.as_deref().unwrap_or_default();
        match collator {
            Some(collator) => collator.compare(t1, t2),
            None => t1.cmp(t2),
        }
    });
}

/// Sorts tracks by dates, falls back to alphabetic title sorting.
pub fn sort_tracks_chronologically<T: AsTrack>(tracks: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|a, b| match (a.as_track().date, b.as_track().date) {
        (Some(ad), Some(bd)) => bd.cmp(&ad),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => {
            let t1 = a.as_track().title.as_deref().unwrap_or_default();
            let t2 = b.as_track().title.as_deref().unwrap_or_default();

            match collator {
                Some(collator) => collator.compare(t1, t2),
                None => t1.cmp(t2),
            }
        }
    });
}

/// Sorts tracks by disc numbers, then track numbers, falls back to sorting by titles.
pub fn sort_tracks_in_album<T: AsTrack>(tracks: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|a, b| {
        let a = a.as_track();
        let b = b.as_track();

        let disc_cmp = match (a.disc, b.disc) {
            (Some(ad), Some(bd)) => ad.cmp(&bd),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        };

        let track_cmp = match (a.track, b.track) {
            (Some(at), Some(bt)) => at.cmp(&bt),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        };

        disc_cmp.then(track_cmp).then_with(|| {
            let s1 = a.title.as_deref().unwrap_or_default();
            let s2 = b.title.as_deref().unwrap_or_default();
            match collator {
                Some(collator) => collator.compare(s1, s2),
                None => s1.cmp(s2),
            }
        })
    });
}

/// Sorts tracks by artist names alphabetically.
pub fn sort_tracks_by_artist<T: AsTrack>(tracks: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|t1, t2| {
        match (
            t1.as_track().artists.as_ref(),
            t2.as_track().artists.as_ref(),
        ) {
            (Some(l1), Some(l2)) => {
                for (s1, s2) in l1.iter().zip(l2.iter()) {
                    let res = match collator {
                        Some(collator) => collator.compare(s1, s2),
                        None => s1.cmp(s2),
                    };
                    if res != std::cmp::Ordering::Equal {
                        return res;
                    }
                }
                l1.len().cmp(&l2.len())
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
}

/// Sorts tracks by album titles alphabetically.
pub fn sort_tracks_by_album<T: AsTrack>(tracks: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|t1, t2| {
        let s1 = t1.as_track().album.as_deref().unwrap_or_default();
        let s2 = t2.as_track().album.as_deref().unwrap_or_default();

        match collator {
            Some(collator) => collator.compare(s1, s2),
            None => s1.cmp(s2),
        }
    });
}

/// Sorts albums by their titles alphabetically.
pub fn sort_albums<T: AsAlbum>(albums: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            albums.sort_by(|a1, a2| collator.compare(&a1.as_album().title, &a2.as_album().title));
        }
        None => {
            albums.sort_by(|a1, a2| a1.as_album().title.cmp(&a2.as_album().title));
        }
    }
}

/// Sorts albums by dates, falls back to sorting by titles alphabetically.
pub fn sort_albums_chronologically<T: AsAlbum>(albums: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    albums.sort_by(|a, b| match (a.as_album().date, b.as_album().date) {
        (Some(ad), Some(bd)) => bd.cmp(&ad),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => match collator {
            Some(collator) => collator.compare(&a.as_album().title, &b.as_album().title),
            None => a.as_album().title.cmp(&b.as_album().title),
        },
    });
}

/// Sorts albums by artist names alphabetically.
pub fn sort_albums_by_artist<T: AsAlbum>(albums: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    albums.sort_by(|a1, a2| {
        let a1 = a1.as_album();
        let a2 = a2.as_album();
        for (s1, s2) in a1.artists.iter().zip(a2.artists.iter()) {
            let res = match collator {
                Some(collator) => collator.compare(s1, s2),
                None => s1.cmp(s2),
            };
            if res != std::cmp::Ordering::Equal {
                return res;
            }
        }
        a1.artists.len().cmp(&a2.artists.len())
    });
}

/// Sorts artists by names alphabetically.
pub fn sort_artists<T: AsArtist>(artists: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            artists.sort_by(|a1, a2| collator.compare(&a1.as_artist().name, &a2.as_artist().name));
        }
        None => {
            artists.sort_by(|a1, a2| a1.as_artist().name.cmp(&a2.as_artist().name));
        }
    }
}

/// Sorts playlists by titles alphabetically.
pub fn sort_playlists<T: AsPlaylist>(playlists: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            playlists
                .sort_by(|p1, p2| collator.compare(&p1.as_playlist().name, &p2.as_playlist().name));
        }
        None => {
            playlists.sort_by(|p1, p2| p1.as_playlist().name.cmp(&p2.as_playlist().name));
        }
    }
}

/// Sorts genres by names alphabetically.
pub fn sort_genres<T: AsGenre>(genres: &mut [T]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            genres.sort_by(|g1, g2| collator.compare(&g1.as_genre().name, &g2.as_genre().name));
        }
        None => {
            genres.sort_by(|g1, g2| g1.as_genre().name.cmp(&g2.as_genre().name));
        }
    }
}
