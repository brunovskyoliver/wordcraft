//! Ribbon widgets: large and small buttons, split buttons, groups, combo fields, colour grids.

use egui::{Align2, Color32, CornerRadius, Rect, Response, Sense, Stroke, Ui, pos2, vec2};
use serde_json::Value;

use crate::theme::{Tokens, regular};
use crate::{WordApp, icons};

mod buttons;
mod combo;
mod groups;
mod icon_buttons;
mod tooltip;

pub use buttons::{
    BIG_ICON_Y, BIG_LABEL_Y, ButtonAction, CONTENT_H, DROPDOWN, LABEL_H, LABEL_LINE_H, LABEL_PITCH, MenuButtonOptions, big, big_toggle, enabled,
    label_line, menu_button, menu_enabled, small, split,
};
pub(crate) use buttons::{bg, run_from_ribbon};
pub(crate) use combo::swatch;
pub use combo::{
    ADD_FONT, COMBO_PREVIEW_ROW_H, DrawnState, FontComboOptions, LOAD_SYSTEM_FONTS, color_grid, combo, combo_hint, combo_list, drawn_state,
    font_combo, font_combo_with_recent, font_combo_with_recent_hint, font_matches,
};
#[cfg(test)]
use combo::{Row, highlight_row, section_row, section_rows};
pub use groups::{command_icon, command_menu, customize_menu_items, group, group_titled, row};
pub use icon_buttons::{
    dialog_tabs, icon_button, icon_menu_item, icon_only_button, icon_text_button, icon_text_button_width, icon_text_menu_button, primary_button,
};
#[cfg(test)]
use tooltip::show_hint;
#[cfg(test)]
pub(crate) use tooltip::tooltip_lines;
pub use tooltip::{key_text, shortcut_text};

/// Gives a text field without a visible label its accessible name; a hint isn't a name. Only the
/// name is set, so the field's role and value stay as the field reported them.
pub fn named_input(response: Response, name: &str) -> Response {
    response.ctx.accesskit_node_builder(response.id, |node| node.set_label(name.to_string()));
    response
}

/// Gives a drop-down its accessible name, keeping its current choice readable.
pub fn named_combo(response: &Response, name: &str, shown: &str) {
    let enabled = response.enabled();
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::Role::ComboBox, enabled, name);
        info.current_text_value = Some(shown.to_string());
        info
    });
}

/// A slider with a visible rail. egui paints the rail in the input fill, which has no contrast
/// with the dialog or status bar behind it (AUD-008).
pub fn slider(ui: &mut Ui, slider: egui::Slider<'_>) -> Response {
    let rail = Tokens::get(ui.ctx()).border_strong;
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = rail;
        ui.add(slider)
    })
    .inner
}
// #653: a two-line label ends inside the content, above the group title row.
const _: () = assert!(BIG_LABEL_Y + LABEL_PITCH + LABEL_LINE_H <= CONTENT_H);
const _: () = assert!(BIG_ICON_Y + 16.0 <= BIG_LABEL_Y);

#[cfg(test)]
thread_local! {
    /// Groups drawn taller than [`CONTENT_H`] (title and height): their content runs into the
    /// group label below, as View › Show did with four check boxes in one column.
    pub(crate) static TALL_GROUPS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
mod tests;
