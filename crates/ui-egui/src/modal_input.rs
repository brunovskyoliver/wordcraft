//! Modal ownership for the application chrome, document and nested dialogs.

use egui::{Context, Id, InputState, LayerId, Order, Ui, Window};

use crate::WordApp;

/// Modeless Find/Go To and Mark Index Entry leave document and ribbon interaction available.
pub(crate) fn background_blocked(app: &WordApp) -> bool {
    app.dialog.as_ref().is_some_and(|dialog| !dialog.is_modeless()) || above_dialog(app)
}

/// Child dialogs are painted after the ordinary document dialog and own its input too.
pub(crate) fn above_dialog(app: &WordApp) -> bool {
    app.colors.is_some() || app.file_dialog_open() || app.zotero.alert_open()
}

/// Keep floating panes and other independent Areas below modal windows. The disabled
/// background scope supplies immediate ownership before egui's next-frame modal state.
pub(crate) fn window<'a>(ctx: &Context, id: Id, window: Window<'a>) -> Window<'a> {
    ctx.memory_mut(|memory| memory.set_modal_layer(LayerId::new(Order::Foreground, id)));
    window.order(Order::Foreground)
}

/// Raw dialog shortcuts need the same ownership check as egui's enabled widgets.
pub(crate) fn keys_enabled(ui: &Ui) -> bool {
    ui.is_enabled() && ui.ctx().memory(|memory| memory.is_above_modal_layer(ui.layer_id()))
}

pub(crate) fn key_input(ui: &Ui, read: impl FnOnce(&InputState) -> bool) -> bool {
    keys_enabled(ui) && ui.input(read)
}

#[cfg(test)]
mod tests;
