//! Selection Pane: a live list from the engine; row activation uses its selection command.

use egui::{Response, Ui, vec2};
use serde_json::json;

use crate::WordApp;
use crate::theme::regular;

pub(super) fn show(app: &mut WordApp, ui: &mut Ui) {
    let objects = wordcraft_engine::cmd::objects::selection_objects(&app.session);
    if objects.is_empty() {
        ui.add(egui::Label::new(tl!("No objects in the document body.")).wrap());
        return;
    }
    ui.add(egui::Label::new(egui::RichText::new(tl!("Objects in the document body")).small().weak()).wrap());
    ui.separator();
    let selected = wordcraft_engine::cmd::objects::picked_objects(&app.session);
    let mut picked = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for object in objects {
            let label = if object.hidden { crate::i18n::fmt(tl!("{name} (hidden)"), &[("name", &object.name)]) } else { object.name };
            ui.push_id(&object.pos, |ui| {
                if object_row(ui, &label, selected.contains(&object.pos)).clicked() {
                    picked = Some(object.index);
                }
            });
        }
    });
    if let Some(index) = picked {
        let _ = app.run("select.objects", json!({"index": index}));
        app.canvas.want_focus = true;
    }
}

fn object_row(ui: &mut Ui, name: &str, selected: bool) -> Response {
    let label = egui::RichText::new(name).font(regular(12.5));
    ui.add(egui::Button::selectable(selected, (label, egui::Atom::grow())).wrap().min_size(vec2(ui.available_width(), ui.spacing().interact_size.y)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_object_names_wrap_inside_the_pane() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0));
        let mut short = egui::Rect::NOTHING;
        let mut long = egui::Rect::NOTHING;
        let mut pane = egui::Rect::NOTHING;
        for _ in 0..3 {
            ctx.run_ui(egui::RawInput { screen_rect: Some(screen), ..Default::default() }, |ui| {
                let result = egui::Panel::right("selection_test").exact_size(180.0).show(ui, |ui| {
                    short = object_row(ui, "Rectangle 1", false).rect;
                    long = object_row(ui, &"Long object name ".repeat(20), true).rect;
                });
                pane = result.response.rect;
            })
            .drop_without_applying_deltas();
        }
        assert!(long.height() > short.height(), "long names wrap instead of truncating");
        assert!(long.left() >= pane.left() && long.right() <= pane.right(), "the row stays inside its pane");
    }
}
