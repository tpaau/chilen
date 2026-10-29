//! Sorting helper functions for library items.

use std::sync::{Arc, RwLock};

use icu::collator::CollatorBorrowed;

use crate::music_lib::{Album, Artist, Genre, Playlist, Track};

pub(crate) static COLLATOR: RwLock<Option<Arc<CollatorBorrowed<'_>>>> = RwLock::new(None);

/// Sorts tracks alphabetically by titles.
pub fn sort_tracks(tracks: &mut [Arc<Track>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|t1, t2| {
        let t1 = t1.title.as_deref().unwrap_or_default();
        let t2 = t2.title.as_deref().unwrap_or_default();
        match collator {
            Some(collator) => collator.compare(t1, t2),
            None => t1.cmp(t2),
        }
    });
}

/// Sorts tracks by dates, falls back to alphabetic title sorting.
pub fn sort_tracks_chronologically(tracks: &mut [Arc<Track>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|a, b| match (a.date, b.date) {
        (Some(ad), Some(bd)) => bd.cmp(&ad),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => {
            let t1 = a.title.as_deref().unwrap_or_default();
            let t2 = b.title.as_deref().unwrap_or_default();

            match collator {
                Some(collator) => collator.compare(t1, t2),
                None => t1.cmp(t2),
            }
        }
    });
}

/// Sorts tracks by disc numbers, then track numbers, falls back to sorting by titles.
pub fn sort_tracks_in_album(tracks: &mut [Arc<Track>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|a, b| {
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
pub fn sort_tracks_by_artist(tracks: &mut [Arc<Track>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|t1, t2| match (t1.artists.as_ref(), t2.artists.as_ref()) {
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
    });
}

/// Sorts tracks by album titles alphabetically.
pub fn sort_tracks_by_album(tracks: &mut [Arc<Track>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    tracks.sort_by(|t1, t2| {
        let s1 = t1.album.as_deref().unwrap_or_default();
        let s2 = t2.album.as_deref().unwrap_or_default();

        match collator {
            Some(collator) => collator.compare(s1, s2),
            None => s1.cmp(s2),
        }
    });
}

/// Sorts albums by their titles alphabetically.
pub fn sort_albums(albums: &mut [Arc<Album>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            albums.sort_by(|a1, a2| collator.compare(&a1.title, &a2.title));
        }
        None => {
            albums.sort_by(|a1, a2| a1.title.cmp(&a2.title));
        }
    }
}

/// Sorts albums by dates, falls back to sorting by titles alphabetically.
pub fn sort_albums_chronologically(albums: &mut [Arc<Album>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    albums.sort_by(|a, b| match (a.date, b.date) {
        (Some(ad), Some(bd)) => bd.cmp(&ad),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => match collator {
            Some(collator) => collator.compare(&a.title, &b.title),
            None => a.title.cmp(&b.title),
        },
    });
}

/// Sorts albums by artist names alphabetically.
pub fn sort_albums_by_artist(albums: &mut [Arc<Album>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    albums.sort_by(|a1, a2| {
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
pub fn sort_artists(artists: &mut [Arc<Artist>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            artists.sort_by(|a1, a2| collator.compare(&a1.name, &a2.name));
        }
        None => {
            artists.sort_by(|a1, a2| a1.name.cmp(&a2.name));
        }
    }
}

/// Sorts playlists by titles alphabetically.
pub fn sort_playlists(playlists: &mut [Arc<Playlist>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            playlists.sort_by(|p1, p2| collator.compare(&p1.name, &p2.name));
        }
        None => {
            playlists.sort_by(|p1, p2| p1.name.cmp(&p2.name));
        }
    }
}

/// Sorts genres by names alphabetically.
pub fn sort_genres(genres: &mut [Arc<Genre>]) {
    let guard = COLLATOR.read().unwrap();
    let collator = guard.as_ref();

    match collator {
        Some(collator) => {
            genres.sort_by(|g1, g2| collator.compare(&g1.name, &g2.name));
        }
        None => {
            genres.sort_by(|g1, g2| g1.name.cmp(&g2.name));
        }
    }
}
