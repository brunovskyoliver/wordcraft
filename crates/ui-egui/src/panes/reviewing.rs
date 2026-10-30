//! Reviewing Pane: the engine's tracked changes as a list; a row selects its change.

use egui::{Ui, vec2};
use serde_json::{Value, json};

use crate::WordApp;
use crate::theme::regular;

pub(super) fn show(app: &mut WordApp, ui: &mut Ui) {
    let list = match app.session.run("review.changes", &json!({})) {
        Ok(Value::Array(list)) => list,
        _ => Vec::new(),
    };
    if list.is_empty() {
        ui.add(egui::Label::new(tl!("No tracked changes.")).wrap());
        return;
    }
    ui.add(
        egui::Label::new(egui::RichText::new(crate::i18n::fmt(tl!("Revisions: {count}"), &[("count", &list.len().to_string())])).small().weak())
            .wrap(),
    );
    let mut command = None;
    ui.horizontal(|ui| {
        if ui.button(tl!("Accept")).clicked() {
            command = Some("review.accept");
        }
        if ui.button(tl!("Reject")).clicked() {
            command = Some("review.reject");
        }
    });
    ui.separator();
    let sel = app.session.sel.ordered();
    let mut picked = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, change) in list.iter().enumerate() {
            let Some((a, b)) = range(change) else { continue };
            let selected = sel == (a.clone(), b.clone());
            ui.push_id(i, |ui| {
                let label = egui::RichText::new(row_text(change)).font(regular(12.5));
                let row = egui::Button::selectable(selected, (label, egui::Atom::grow()))
                    .wrap()
                    .min_size(vec2(ui.available_width(), ui.spacing().interact_size.y));
                if ui.add(row).clicked() {
                    picked = Some((a, b));
                }
            });
        }
    });
    if let Some((anchor, focus)) = picked {
        app.session.sel = wordcraft_engine::Selection { anchor, focus };
        app.session.push_ui_request(json!({"scrollTo": "caret"}));
    }
    if let Some(id) = command {
        let _ = app.run(id, json!({}));
    }
}

fn range(change: &Value) -> Option<(wordcraft_doc::Pos, wordcraft_doc::Pos)> {
    Some((serde_json::from_value(change.get("start")?.clone()).ok()?, serde_json::from_value(change.get("end")?.clone()).ok()?))
}

/// "Author — Inserted: text"; a change's own description when the engine gives one.
fn row_text(change: &Value) -> String {
    let field = |key: &str| change.get(key).and_then(Value::as_str).unwrap_or_default();
    let kind = match field("kind") {
        "insert" | "ins" | "insertion" | "moveTo" => tl!("Inserted"),
        "delete" | "del" | "deletion" | "moveFrom" => tl!("Deleted"),
        "format" => tl!("Formatted"),
        _ => tl!("Changed"),
    };
    let what = if field("description").is_empty() { field("text") } else { field("description") };
    let short: String = what.chars().take(120).collect();
    match field("author") {
        "" => format!("{kind}: {short}"),
        author => format!("{author} — {kind}: {short}"),
    }
}
