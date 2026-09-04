use chilen_backend::music_lib::Progress;
use iced::Element;
use iced_m3::{theme::ColorScheme, widget::dialog};
use iced_widget::{column, text};

use crate::gui::{
    Message, SPACING_REGULAR,
    font::{self},
    icons,
};

pub fn view<'a>(theme: &'a impl ColorScheme, progress: Option<Progress>) -> Element<'a, Message> {
    let status = progress
        .map(|p| {
            match p {
                Progress::FindingTracks => "Finding tracks",
                Progress::Indexing => "Indexing",
                Progress::RebuildingLibrary => "Rebuilding library",
                Progress::RestoringState => "Restoring state...",
                // Those two states should never actually be displayed
                Progress::Done => "Done.",
                Progress::Failed(_) => "Error!",
            }
        })
        .unwrap_or("Starting...");
    let tooltip = "The initial indexing might take up to a minute depending on your library size and hardware. After that, Chilen will boot up almost instantly!";

    let mut content = column![].spacing(SPACING_REGULAR);

    #[cfg(debug_assertions)]
    {
        use crate::gui::font::bold_text;

        let warning = "INDEXER RUNNING IN DEBUG MODE, image decoding will be EXTREMELY SLOW.\nConsider running Chilen in release mode for the initial indexing";
        content = content.push(
            bold_text(warning)
                .size(font::SIZE_REGULAR)
                .color(theme.error()),
        );
    }

    content = content.push(
        text(tooltip)
            .size(font::SIZE_SMALL)
            .color(theme.on_surface_variant()),
    );

    dialog(theme, content, Vec::new())
        .title_font(font::bold())
        .title(status)
        .icon_font(icons::filled())
        .icon(*icons::REFRESH)
        .width((dialog::MIN_WIDTH + dialog::MAX_WIDTH) / 2.0)
        .into()
}
