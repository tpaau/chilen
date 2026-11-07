use std::{path::PathBuf, sync::Arc};

use chilen_backend::music_lib::{LibraryItem, Playlist, Track, toggle_favorite};
use dirs::home_dir;
use iced::Task;
use log::{debug, error};

use crate::gui::{Chilen, dialog::Dialog, event_loop};

#[derive(Clone)]
pub enum Message {
    CreatePlaylist(String),
    ImportPlaylist(Option<String>, rfd::FileHandle),
    RenamePlaylist {
        playlist: Arc<Playlist>,
        name: String,
    },
    DeletePlaylist(Arc<Playlist>),
    AddTracksToPlaylist {
        tracks: Vec<Arc<Track>>,
        playlist: Arc<Playlist>,
    },
    ExportPlaylist(Arc<Playlist>),
    SaveExportedPlaylist(Arc<Playlist>, Option<rfd::FileHandle>),
    ToggleFavorite(LibraryItem),
    RemoveTrackFromPlaylist {
        playlist: Arc<Playlist>,
        index: usize,
    },
}

pub fn update(state: &mut Chilen, message: Message) -> Task<event_loop::Message> {
    match message {
        Message::CreatePlaylist(name) => {
            if let Err(e) = chilen_backend::music_lib::create_playlist(&name, &None) {
                error!("Couldn't create the playlist: {e}");
                state.dialog = Dialog::Error(format!("Couldn't create the playlist: {e}"));
            } else {
                state.dialog = Dialog::None;
            }
        }
        Message::ImportPlaylist(name, handle) => {
            if let Err(e) =
                chilen_backend::music_lib::import_playlist_from_m3u8(name, &handle.into())
            {
                error!("Could not import the playlist: {e}");
                state.dialog = Dialog::Error(format!("Couldn't import the playlist: {e}"));
            } else {
                state.dialog = Dialog::None;
            }
        }
        Message::RenamePlaylist { playlist, name } => {
            if let Err(e) = chilen_backend::music_lib::rename_playlist(&playlist.name, &name) {
                error!("Could not rename the playlist: {e}");
                state.dialog = Dialog::Error(format!("Couldn't rename the playlist: {e}"));
            } else {
                state.dialog = Dialog::None;
            }
        }
        Message::DeletePlaylist(playlist) => {
            if let Err(e) = chilen_backend::music_lib::delete_playlists(vec![&playlist.name]) {
                error!("Couldn't delete playlist: {e}");
                state.dialog = Dialog::Error(format!("Couldn't delete playlist: {e}"))
            } else {
                state.dialog = Dialog::None;
            }
        }
        Message::AddTracksToPlaylist { tracks, playlist } => {
            if let Err(e) = chilen_backend::music_lib::add_tracks(
                &playlist.name,
                &tracks.iter().map(|t| t.hash_self()).collect::<Vec<_>>(),
            ) {
                state.dialog = Dialog::Error(format!(
                    "Couldn't add the track to playlist {}: {e}",
                    playlist.name
                ));
            } else {
                state.dialog = Dialog::None;
            }
        }
        Message::ExportPlaylist(playlist) => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_directory(home_dir().unwrap_or(PathBuf::from(".")))
                    .set_file_name(format!("{}.m3u8", playlist.name))
                    .save_file(),
                |maybe_handle| Message::SaveExportedPlaylist(playlist, maybe_handle),
            )
            .map(event_loop::Message::Library);
        }
        Message::SaveExportedPlaylist(playlist, file_handle) => match file_handle {
            Some(handle) => {
                if let Err(e) = chilen_backend::music_lib::export_playlist_to_m3u8(
                    &playlist.name,
                    handle.path(),
                ) {
                    let msg = format!(
                        "Couldn't export playlist {} to {:?}: {e}",
                        playlist.name,
                        handle.path()
                    );
                    error!("{msg}");
                    state.dialog = Dialog::Error(msg);
                }
            }
            None => debug!("User cancelled the export operation"),
        },
        Message::ToggleFavorite(favorite_item) => {
            let _ = toggle_favorite(favorite_item);
        }
        Message::RemoveTrackFromPlaylist { playlist, index } => {
            if let Err(e) =
                chilen_backend::music_lib::remove_tracks_by_indices(&playlist.name, vec![index])
            {
                let msg = format!("Couldn't remove track from playlist: {e}");
                error!("{msg}");
                state.dialog = crate::gui::dialog::Dialog::Error(msg);
            }
        }
    }
    Task::none()
}
