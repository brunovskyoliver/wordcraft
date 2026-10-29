//! Thesaurus presentation. The engine owns lookup, word targeting and replacement validation.

use egui::Ui;
use serde_json::{Value, json};
use wordcraft_engine::cmd::review::{ThesaurusLookup, thesaurus_lookup};

use crate::WordApp;

#[derive(Default)]
pub(crate) struct ThesaurusState {
    query: String,
    result: Option<ThesaurusLookup>,
    message: String,
    focus: bool,
}

impl ThesaurusState {
    pub(crate) fn focus_requested(&self) -> bool {
        self.focus
    }

    pub(crate) fn from_result(value: &Value) -> Result<Self, String> {
        let result: ThesaurusLookup = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        Ok(Self { query: result.word.clone(), result: Some(result), message: String::new(), focus: true })
    }
}

enum Action {
    Lookup(String),
    Insert(String),
}

pub(super) fn show(app: &mut WordApp, ui: &mut Ui) {
    let mut state = std::mem::take(&mut app.thesaurus);
    if state.result.is_none() {
        let result = thesaurus_lookup(&app.session, None);
        state.query = result.word.clone();
        state.result = Some(result);
    }
    let mut action = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        let label = ui.label(tl!("Search for"));
        let field_id = app.window_id("thesaurus_query");
        let request_focus = state.focus && crate::modal_input::keys_enabled(ui);
        if request_focus {
            // Let input arriving in the opening frame reach this editor.
            ui.memory_mut(|m| m.request_focus(field_id));
        }
        let field = ui
            .add(egui::TextEdit::singleline(&mut state.query).id(field_id).desired_width(ui.available_width()).char_limit(256))
            .labelled_by(label.id);
        if request_focus {
            // egui's click-away handling runs while the field is registered. The ribbon
            // click that opened this pane is outside the field, so claim focus after that
            // handling too before consuming the one-shot request.
            field.request_focus();
            state.focus = false;
        }
        if field.has_focus() && crate::modal_input::keys_enabled(ui) {
            app.canvas.focused = false;
            app.canvas.want_focus = false;
        }
        let enter = field.lost_focus() && crate::modal_input::key_input(ui, |i| i.key_pressed(egui::Key::Enter));
        if enter {
            ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            });
        }
        if ui.button(tl!("Look Up")).clicked() || enter {
            action = Some(Action::Lookup(state.query.clone()));
        }
        ui.add(egui::Label::new(egui::RichText::new(tl!("Limited built-in English dictionary.")).small().weak()).wrap());
        ui.separator();
        if !state.message.is_empty() {
            ui.add(egui::Label::new(egui::RichText::new(&state.message).color(ui.visuals().error_fg_color)).wrap());
        }
        if let Some(result) = &state.result {
            let current = result.target.as_ref().is_some_and(|target| target.is_current(&app.session));
            if result.target.is_some() && !current {
                ui.add(egui::Label::new(tl!("The document or selection changed. Look up the word again before inserting.")).wrap());
            } else if let Some(target) = &result.target {
                ui.add(egui::Label::new(crate::i18n::fmt(tl!("Replace: {word}"), &[("word", target.text())])).wrap());
            } else {
                ui.add(egui::Label::new(tl!("Select a single word in the document to replace it.")).wrap());
            }
            if result.word.is_empty() {
                ui.add(egui::Label::new(tl!("Enter a word to look up synonyms.")).wrap());
            } else if result.synonyms.is_empty() {
                ui.add(egui::Label::new(tl!("No synonyms found in the built-in dictionary. Try another word.")).wrap());
            } else {
                ui.add(egui::Label::new(egui::RichText::new(&result.word).strong()).wrap());
                for word in &result.synonyms {
                    ui.push_id(word, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let response = ui.add(egui::Button::new(word).wrap());
                            if response.clicked() {
                                action = Some(Action::Lookup(word.clone()));
                            }
                            response.on_hover_text(tl!("Look up related words"));
                            let insert =
                                ui.add_enabled(current && crate::widgets::enabled(app, "review.replaceSynonym"), egui::Button::new(tl!("Insert")));
                            if insert.clicked() {
                                action = Some(Action::Insert(word.clone()));
                            }
                            if let Some(reason) = app.session.why_disabled("review.replaceSynonym") {
                                insert.on_disabled_hover_text(tl!(reason));
                            }
                        });
                    });
                }
            }
        }
    });
    app.thesaurus = state;
    match action {
        Some(Action::Lookup(word)) => {
            if let Err(message) = app.run("review.thesaurus", json!({"word": word})) {
                app.thesaurus.message = message;
                app.thesaurus.focus = true;
            }
        }
        Some(Action::Insert(text)) => {
            let params = app.thesaurus.result.as_ref().map(|r| json!({"target": r.target, "word": r.word, "text": text}));
            if let Some(params) = params {
                match app.run("review.replaceSynonym", params) {
                    Ok(_) => {
                        app.thesaurus = Default::default();
                        app.canvas.want_focus = true;
                    }
                    Err(message) => app.thesaurus.message = message,
                }
            }
        }
        None => {}
    }
}

#[cfg(test)]
mod tests;
