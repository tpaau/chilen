use iced::{Background, Border, Color, Shadow, widget::container};

use crate::gui::theme::Theme;

pub fn scrollable(
    status: iced::widget::scrollable::Status,
    theme: &Theme,
) -> iced::widget::scrollable::Style {
    let color = match status {
        iced::widget::scrollable::Status::Active {
            is_horizontal_scrollbar_disabled: _,
            is_vertical_scrollbar_disabled: _,
        } => Color::TRANSPARENT,
        iced::widget::scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered: _,
            is_vertical_scrollbar_hovered: _,
            is_horizontal_scrollbar_disabled: _,
            is_vertical_scrollbar_disabled: _,
        } => theme.on_surface_variant().scale_alpha(0.6),
        iced::widget::scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged: _,
            is_vertical_scrollbar_dragged: _,
            is_horizontal_scrollbar_disabled: _,
            is_vertical_scrollbar_disabled: _,
        } => theme.on_surface_variant(),
    };
    iced::widget::scrollable::Style {
        vertical_rail: iced::widget::scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: iced::widget::scrollable::Scroller {
                background: Background::Color(color),
                border: Border::default().rounded(u32::MAX),
            },
        },
        container: container::Style::default(),
        horizontal_rail: iced::widget::scrollable::Rail {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            scroller: iced::widget::scrollable::Scroller {
                background: Background::Color(color),
                border: Border::default().rounded(u32::MAX),
            },
        },
        gap: None,
        auto_scroll: iced::widget::scrollable::AutoScroll {
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: Color::TRANSPARENT,
        },
    }
}
