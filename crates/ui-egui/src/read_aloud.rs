//! The Read Aloud player: a small floating window with previous / play-pause / next / stop,
//! a speed slider and "Skip citations & bibliography". Every control runs a `readAloud.*`
//! command; the player itself is [`wordcraft_engine::speech::ReadAloud`].
//!
//! While reading, the caret follows the sentence being spoken (so stopping leaves it where
//! reading stopped) and the sentence is highlighted on the page.

use egui::{Color32, Rect, Ui, vec2};
use serde_json::json;
use wordcraft_engine::speech::{MAX_RATE, MIN_RATE, State};

use crate::WordApp;
use crate::theme::Tokens;

/// Per-frame bookkeeping: follow the spoken sentence with the caret, report errors, and keep
/// frames coming while reading.
pub fn poll(app: &mut WordApp, ctx: &egui::Context) {
    let ra = &app.session.read_aloud;
    if !ra.open {
        app.read_aloud_at = None;
        return;
    }
    if ra.state() == State::Playing {
        ctx.request_repaint_after(std::time::Duration::from_millis(120));
    }
    if let Some(e) = app.session.read_aloud.error()
        && app.read_aloud_error.as_deref() != Some(e.as_str())
    {
        app.read_aloud_error = Some(e.clone());
        app.status(e);
    }
    // The caret follows the reading only in the document being read.
    let current = if reads_shown_document(app) { app.session.read_aloud.current().map(|u| u.start) } else { None };
    if current != app.read_aloud_at {
        if let Some(start) = &current {
            app.session.sel = wordcraft_engine::Selection::caret(app.session.doc.clamp(start));
            app.session.goal_x = None;
            app.canvas.scroll_to_caret = true;
        }
        app.read_aloud_at = current;
    }
}

/// The page rectangles (in page units) of the sentence being read, to highlight.
pub fn highlight(app: &mut WordApp) -> Option<(wordcraft_engine::doc::Pos, wordcraft_engine::doc::Pos)> {
    if !app.session.read_aloud.open || !reads_shown_document(app) {
        return None;
    }
    let u = app.session.read_aloud.current()?;
    Some((app.session.doc.clamp(&u.start), app.session.doc.clamp(&u.end)))
}

/// Whether the reading is of the document the window shows: its sentence positions are in it.
fn reads_shown_document(app: &WordApp) -> bool {
    app.session.read_aloud.document == Some(app.session.document_id())
}

/// The player window.
pub fn show(app: &mut WordApp, ctx: &egui::Context) {
    if !app.session.read_aloud.open {
        return;
    }
    // With a window per document, the player is in the window of the document being read.
    if app.windows.native()
        && let Some(d) = app.session.read_aloud.document
        && d != app.session.document_id()
        && app.session.is_open(d)
    {
        return;
    }
    let t = Tokens::get(ctx);
    let state = app.session.read_aloud.state();
    let (index, count) = app.session.read_aloud.progress();
    let mut run: Option<(&str, serde_json::Value)> = None;
    let right_top = app.canvas.canvas_rect.map_or(ctx.content_rect().right_top() + vec2(-24.0, 160.0), |r| r.right_top() + vec2(-24.0, 16.0));
    let window = egui::Window::new(tl!("Read Aloud"))
        .enabled(!crate::modal_input::background_blocked(app))
        .id(egui::Id::unique("read-aloud-player"))
        .collapsible(false)
        .resizable(false)
        .pivot(egui::Align2::RIGHT_TOP)
        .default_pos(right_top);
    let shown = crate::window_close::window(ctx, window).show(ctx, |ui| {
        ui.set_width(236.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if button(ui, &t, "mediaPrev", tl!("Previous sentence"), count > 0 && index > 0) {
                run = Some(("readAloud.previous", json!({})));
            }
            let (icon, tip) = if state == State::Playing { ("mediaPause", tl!("Pause")) } else { ("mediaPlay", tl!("Play")) };
            if button(ui, &t, icon, tip, true) {
                run = Some(("readAloud.playPause", json!({})));
            }
            if button(ui, &t, "mediaNext", tl!("Next sentence"), index + 1 < count) {
                run = Some(("readAloud.next", json!({})));
            }
            if button(ui, &t, "mediaStop", tl!("Stop"), true) {
                run = Some(("readAloud.stop", json!({})));
            }
            ui.add_space(4.0);
            let what = match state {
                State::Playing => format!("{} {} / {}", tl!("Sentence"), (index + 1).min(count), count),
                State::Paused => tl!("Paused").to_string(),
                State::Stopped if count > 0 && index >= count => tl!("Finished").to_string(),
                State::Stopped => tl!("Stopped").to_string(),
            };
            ui.label(egui::RichText::new(what).color(t.text_dim).small());
        });
        ui.add_space(6.0);
        // Speed: applied when the slider is let go, so dragging doesn't restart the sentence
        // on every frame.
        let id = egui::Id::unique("read-aloud-rate");
        let mut rate = ui.data(|d| d.get_temp::<f32>(id)).unwrap_or_else(|| app.session.read_aloud.rate());
        ui.horizontal(|ui| {
            let label = ui.label(tl!("Speed"));
            ui.spacing_mut().slider_width = 110.0;
            let r = ui.add(
                egui::Slider::new(&mut rate, MIN_RATE..=MAX_RATE)
                    .step_by(0.05)
                    .trailing_fill(true)
                    .custom_formatter(|v, _| format!("{v:.2}×"))
                    .custom_parser(|s| s.trim().trim_end_matches(['×', 'x']).trim().parse().ok()),
            );
            let r = r.labelled_by(label.id);
            if r.dragged() {
                ui.data_mut(|d| d.insert_temp(id, rate));
            } else {
                ui.data_mut(|d| d.remove::<f32>(id));
                if r.changed() || r.drag_stopped() {
                    run = Some(("readAloud.speed", json!({"value": rate})));
                }
            }
            if ui.small_button("1×").on_hover_text(tl!("Normal speed")).clicked() {
                run = Some(("readAloud.speed", json!({"value": 1.0})));
            }
        });
        let mut skip = app.session.read_aloud.skip_citations;
        if ui
            .checkbox(&mut skip, tl!("Skip citations & bibliography"))
            .on_hover_text(tl!("Don't read Zotero, Mendeley or EndNote citations and bibliographies"))
            .changed()
        {
            run = Some(("readAloud.skipCitations", json!({"value": skip})));
        }
        // The error `poll` saw this frame, not the worker's latest: the player thread can set
        // one between the two, and the window would show a message the status bar hasn't.
        if let Some(e) = app.read_aloud_error.as_ref().filter(|e| app.session.read_aloud.error().as_ref() == Some(*e)) {
            ui.label(egui::RichText::new(e.as_str()).color(t.red).small());
        }
    });
    if shown.is_some_and(|s| crate::window_close::show(ctx, &s.response, &t)) {
        run = Some(("readAloud.stop", json!({})));
    }
    if let Some((id, params)) = run {
        let _ = app.run(id, params);
        app.ui.read_aloud_rate = app.session.read_aloud.rate();
        app.ui.read_aloud_skip_citations = app.session.read_aloud.skip_citations;
        ctx.request_repaint();
    }
}

/// An icon button; returns whether it was clicked.
fn button(ui: &mut Ui, t: &Tokens, icon: &str, tip: &str, enabled: bool) -> bool {
    // A real egui button supplies keyboard activation, disabled focus behavior and a
    // visible focus indicator. The original icon is painted inside the same 30-point hit box.
    let on = enabled && ui.is_enabled();
    let resp = ui.add_enabled(enabled, egui::Button::new("").frame(true).frame_when_inactive(false).min_size(vec2(30.0, 30.0)));
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, on, tip));
    let resp = resp.on_hover_text(tip);
    let c = if on { t.icon } else { t.text_disabled };
    let r = Rect::from_center_size(resp.rect.center(), vec2(20.0, 20.0));
    crate::icons::paint(ui.painter(), r, icon, c, if on { t.accent } else { Color32::TRANSPARENT });
    on && resp.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::accesskit::Role;
    use egui_kittest::kittest::{NodeT, Queryable};
    use wordcraft_engine::speech::Backend;

    fn player() -> egui_kittest::Harness<'static, WordApp> {
        let mut app = WordApp::new(
            wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("First sentence. Second sentence. Third sentence.")),
            crate::Services::default(),
        );
        // Hold is the engine's silent backend: no speech program, service or network.
        app.session.read_aloud.backend = Backend::Hold;
        app.run("ui.language", json!({"value": "en"})).unwrap();
        app.run("ui.interfaceSize", json!({"percent": 100})).unwrap();
        app.run("review.readAloud", json!({})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(8);
        h
    }

    fn activate(h: &mut egui_kittest::Harness<'_, WordApp>, label: &str, key: egui::Key) {
        h.get_by_role_and_label(Role::Button, label).focus();
        h.run_steps(4);
        assert!(h.get_by_role_and_label(Role::Button, label).is_focused(), "{label} accepts keyboard focus");
        h.key_press(key);
        h.run_steps(8);
    }

    #[test]
    fn transport_buttons_expose_names_bounds_and_boundary_disabled_states() {
        let mut h = player();
        assert_eq!(h.state().session.read_aloud.progress(), (0, 3));
        for (name, disabled) in [("Previous sentence", true), ("Pause", false), ("Next sentence", false), ("Stop", false)] {
            let button = h.get_by_role_and_label(Role::Button, name);
            assert_eq!(button.accesskit_node().is_disabled(), disabled, "{name}");
            assert_eq!(button.rect().size(), vec2(30.0, 30.0), "the transport keeps its original hit box");
        }
        h.state_mut().execute("readAloud.next", json!({})).unwrap();
        h.state_mut().execute("readAloud.next", json!({})).unwrap();
        h.run_steps(4);
        assert_eq!(h.state().session.read_aloud.progress(), (2, 3));
        assert!(!h.get_by_role_and_label(Role::Button, "Previous sentence").accesskit_node().is_disabled());
        assert!(h.get_by_role_and_label(Role::Button, "Next sentence").accesskit_node().is_disabled());
        h.state_mut().execute("readAloud.playPause", json!({})).unwrap();
        h.run_steps(4);
        assert!(h.query_by_role_and_label(Role::Button, "Pause").is_none());
        assert!(h.query_by_role_and_label(Role::Button, "Play").is_some());
    }

    #[test]
    fn focused_transport_space_and_enter_dispatch_the_player_commands() {
        for key in [egui::Key::Space, egui::Key::Enter] {
            let mut h = player();
            let doc = serde_json::to_value(&h.state().session.doc).unwrap();
            activate(&mut h, "Pause", key);
            assert_eq!(h.state().session.read_aloud.state(), State::Paused);
            activate(&mut h, "Play", key);
            assert_eq!(h.state().session.read_aloud.state(), State::Playing);
            activate(&mut h, "Next sentence", key);
            assert_eq!(h.state().session.read_aloud.progress(), (1, 3));
            activate(&mut h, "Previous sentence", key);
            assert_eq!(h.state().session.read_aloud.progress(), (0, 3));
            activate(&mut h, "Stop", key);
            assert!(!h.state().session.read_aloud.open);
            assert_eq!(h.state().session.read_aloud.state(), State::Stopped);
            assert_eq!(serde_json::to_value(&h.state().session.doc).unwrap(), doc, "{key:?} must never type into the document");
            assert!(!h.state().session.dirty);
            assert_eq!(h.state().session.undo_depth(), 0);
        }
    }

    #[test]
    fn tab_navigation_moves_between_enabled_transport_buttons() {
        let mut h = player();
        h.get_by_role_and_label(Role::Button, "Pause").focus();
        h.run_steps(4);
        h.key_press(egui::Key::Tab);
        h.run_steps(4);
        assert!(h.get_by_role_and_label(Role::Button, "Next sentence").is_focused());
        h.key_press(egui::Key::Tab);
        h.run_steps(4);
        assert!(h.get_by_role_and_label(Role::Button, "Stop").is_focused());
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Tab);
        h.run_steps(4);
        assert!(h.get_by_role_and_label(Role::Button, "Next sentence").is_focused());
        h.state_mut().execute("readAloud.next", json!({})).unwrap();
        h.state_mut().execute("readAloud.next", json!({})).unwrap();
        h.run_steps(4);
        h.get_by_role_and_label(Role::Button, "Pause").focus();
        h.run_steps(4);
        h.key_press(egui::Key::Tab);
        h.run_steps(4);
        assert!(h.get_by_role_and_label(Role::Button, "Stop").is_focused(), "Tab skips Next at the final sentence");
    }

    #[test]
    fn a_modal_disables_every_named_transport_control() {
        let mut h = player();
        h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
        h.run_steps(8);
        for name in ["Previous sentence", "Pause", "Next sentence", "Stop"] {
            assert!(h.get_by_role_and_label(Role::Button, name).accesskit_node().is_disabled(), "{name} follows its disabled player window");
        }
        assert!(h.state().session.read_aloud.open);
        assert_eq!(h.state().session.read_aloud.state(), State::Playing);
    }
}
