use super::*;
use egui::{accesskit::Role, vec2};
use egui_kittest::kittest::{NodeT, Queryable};
use wordcraft_doc::{Document, StoryRef};
use wordcraft_engine::Session;

fn app() -> WordApp {
    let mut app = WordApp::new(Session::new(Document::from_text("The small house.")), crate::Services::default());
    app.run("select.text", json!({"text": "small"})).unwrap();
    app.ui.tab = "Review".into();
    app
}

fn harness(app: WordApp) -> egui_kittest::Harness<'static, WordApp> {
    let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
        |ui, app: &mut WordApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h
}

/// The ribbon has an Insert tab too. Scope result actions below this pane's query field.
fn pane_inserts<'a>(h: &'a egui_kittest::Harness<'_, WordApp>) -> Vec<egui_kittest::Node<'a>> {
    let query = h.get_by_role_and_label(Role::TextInput, "Search for").rect();
    h.query_all_by_role_and_label(Role::Button, "Insert")
        .filter(|node| node.rect().top() >= query.bottom() && node.rect().left() >= query.left())
        .collect()
}

fn lookup(h: &mut egui_kittest::Harness<'_, WordApp>, text: &str) {
    h.get_by_role_and_label(Role::TextInput, "Search for").click();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.key_press(egui::Key::Backspace);
    h.run_steps(1);
    h.get_by_role_and_label(Role::TextInput, "Search for").type_text(text);
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Look Up").click();
    h.run_steps(4);
}

#[test]
fn thesaurus_ribbon_and_shortcut_open_results_without_changing_selection() {
    for shortcut in [false, true] {
        let mut h = harness(app());
        let sel = h.state().session.sel.clone();
        let doc = serde_json::to_value(&h.state().session.doc).unwrap();
        if shortcut {
            h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F7);
        } else {
            h.get_by_role_and_label(Role::Button, "Thesaurus").click();
        }
        h.run_steps(4);
        assert!(h.state().session.view.thesaurus_pane, "shortcut: {shortcut}");
        assert!(h.get_by_role_and_label(Role::TextInput, "Search for").is_focused());
        assert!(h.query_by_role_and_label(Role::Button, "tiny").is_some());
        assert_eq!(h.state().session.sel, sel);
        assert_eq!(serde_json::to_value(&h.state().session.doc).unwrap(), doc);
        assert_eq!(h.state().session.undo_depth(), 0);
        // Opening must also route the next text event to the field without another click.
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.run_steps(2);
        h.event(egui::Event::Text("good".into()));
        h.run_steps(4);
        assert_eq!(h.state().thesaurus.query, "good");
        assert_eq!(h.state().session.sel, sel);
        assert_eq!(serde_json::to_value(&h.state().session.doc).unwrap(), doc);
        assert_eq!(h.state().session.undo_depth(), 0);
    }
}

#[test]
fn thesaurus_lookup_and_insert_use_the_original_word_and_undo_once() {
    let mut app = app();
    let sel = app.session.sel.clone();
    app.run("review.thesaurus", json!({})).unwrap();
    let mut h = harness(app);
    lookup(&mut h, "good");
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body).trim_end(), "The small house.");
    assert_eq!(h.state().session.sel, sel);
    assert!(h.query_by_role_and_label(Role::Button, "fine").is_some());
    assert_eq!(h.state().thesaurus.result.as_ref().unwrap().target.as_ref().unwrap().text(), "small");
    pane_inserts(&h).first().expect("the first synonym has an Insert action").click();
    h.run_steps(4);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body).trim_end(), "The fine house.");
    assert_eq!(h.state().session.undo_depth(), 1);
    assert!(h.state().canvas.focused, "Insert returns the keyboard to the document");
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    h.run_steps(4);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body).trim_end(), "The small house.");
    assert_eq!(h.state().session.sel, sel);
}

#[test]
fn thesaurus_empty_results_explain_dictionary_limits_and_related_lookup_is_pure() {
    let mut app = app();
    app.run("review.thesaurus", json!({"word": "great"})).unwrap();
    let mut h = harness(app);
    let sel = h.state().session.sel.clone();
    h.get_by_role_and_label(Role::Button, "excellent").click();
    h.run_steps(4);
    assert_eq!(h.state().thesaurus.query, "excellent");
    assert!(h.query_by_label("No synonyms found in the built-in dictionary. Try another word.").is_some());
    assert!(h.query_by_label("Limited built-in English dictionary.").is_some());
    lookup(&mut h, "zyzzyva");
    assert!(pane_inserts(&h).is_empty());
    lookup(&mut h, "");
    assert!(h.query_by_label("Enter a word to look up synonyms.").is_some());
    assert_eq!(h.state().session.sel, sel);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body).trim_end(), "The small house.");
    assert_eq!(h.state().session.undo_depth(), 0);
}

#[test]
fn thesaurus_disables_stale_insert_and_refreshes_only_after_lookup() {
    for edit in [false, true] {
        let mut app = app();
        app.run("review.thesaurus", json!({})).unwrap();
        let mut h = harness(app);
        if edit {
            h.state_mut().execute("format.bold", json!({})).unwrap();
        } else {
            h.state_mut().execute("select.text", json!({"text": "house"})).unwrap();
        }
        h.run_steps(4);
        let doc = serde_json::to_value(&h.state().session.doc).unwrap();
        let sel = h.state().session.sel.clone();
        assert!(h.query_by_label("The document or selection changed. Look up the word again before inserting.").is_some());
        let inserts = pane_inserts(&h);
        assert!(!inserts.is_empty());
        assert!(inserts.iter().all(|n| n.accesskit_node().is_disabled()));
        inserts[0].click();
        h.run_steps(4);
        assert_eq!(serde_json::to_value(&h.state().session.doc).unwrap(), doc);
        assert_eq!(h.state().session.sel, sel);
        h.get_by_role_and_label(Role::Button, "Look Up").click();
        h.run_steps(4);
        assert!(h.state().thesaurus.result.as_ref().unwrap().target.as_ref().unwrap().is_current(&h.state().session));
    }
}

#[test]
fn thesaurus_agent_queries_do_not_open_panes_and_window_drafts_stay_separate() {
    let mut app = app();
    let result = app.execute("review.thesaurus", json!({})).unwrap();
    assert_eq!(result["word"], "small");
    assert!(!app.session.view.thesaurus_pane && app.thesaurus.result.is_none());
    app.run("review.thesaurus", json!({})).unwrap();
    app.execute("review.thesaurus", json!({"word": "good"})).unwrap();
    assert!(app.session.view.thesaurus_pane);
    assert_eq!(app.thesaurus.query, "small");
    let mut other = crate::window_ui::WindowUi::new(app.session.document_id());
    app.swap_window(&mut other);
    assert!(app.thesaurus.result.is_none());
    app.swap_window(&mut other);
    assert_eq!(app.thesaurus.query, "small");
    let sel = app.session.sel.clone();
    app.run("view.thesaurusPane", json!({"value": false})).unwrap();
    assert_eq!(app.session.sel, sel);
    assert_eq!(app.session.undo_depth(), 0);
}

#[test]
fn thesaurus_fields_rows_and_empty_state_fit_a_narrow_pane() {
    let long = "x".repeat(256);
    for word in ["small", long.as_str()] {
        let mut app = app();
        app.run("review.thesaurus", json!({"word": word})).unwrap();
        let bounds = std::rc::Rc::new(std::cell::Cell::new((egui::Rect::NOTHING, egui::Rect::NOTHING)));
        let seen = bounds.clone();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(400.0, 600.0)).build_ui_state(
            move |ui, app: &mut WordApp| {
                let response = egui::Panel::right("thesaurus_geometry").exact_size(200.0).show(ui, |ui| {
                    egui::Frame::NONE.inner_margin(10).show(ui, |ui| {
                        let available = ui.available_rect_before_wrap();
                        show(app, ui);
                        seen.set((available, ui.min_rect()));
                    });
                });
                assert!(response.response.rect.width() <= 201.0);
            },
            app,
        );
        h.run_steps(4);
        let (available, body) = bounds.get();
        assert!(body.right() <= available.right() + 0.5, "{word}: {body:?} / {available:?}");
        let field = h.get_by_role_and_label(Role::TextInput, "Search for");
        assert!(field.rect().right() <= available.right() + 0.5);
        for row in h.query_all_by_role(Role::Button) {
            assert!(row.rect().right() <= available.right() + 0.5, "button escaped the pane: {:?}", row.rect());
        }
        if word == "small" {
            assert!(h.query_by_role_and_label(Role::Button, "modest").is_some(), "the last result remains reachable");
        }
    }
}

#[test]
fn thesaurus_opening_batch_does_not_type_behind_the_pane() {
    let mut h = harness(app());
    let sel = h.state().session.sel.clone();
    h.input_mut().events.extend([
        egui::Event::Key { key: egui::Key::F7, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::SHIFT },
        egui::Event::Text("must not reach the page".into()),
    ]);
    h.run_steps(4);
    assert!(h.state().session.view.thesaurus_pane);
    assert_eq!(h.state().session.sel, sel);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body).trim_end(), "The small house.");
    assert_eq!(h.state().session.undo_depth(), 0);
    assert!(h.get_by_role_and_label(Role::TextInput, "Search for").is_focused());
}

#[test]
fn thesaurus_context_menu_replacement_uses_the_same_undoable_command() {
    let mut h = harness(app());
    let sel = h.state().session.sel.clone();
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    h.run_steps(4);
    h.get_by_label("Synonyms").click();
    h.run_steps(4);
    h.get_by_label("tiny").click();
    h.run_steps(4);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body).trim_end(), "The tiny house.");
    assert_eq!(h.state().session.undo_depth(), 1);
    h.state_mut().execute("edit.undo", json!({})).unwrap();
    assert_eq!(h.state().session.sel, sel);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body).trim_end(), "The small house.");
}

#[test]
fn thesaurus_pane_draws_do_not_consume_pending_undo_joins() {
    let mut app = app();
    app.run("format.bold", json!({})).unwrap();
    let sel = app.session.sel.clone();
    app.session.join_next_undo();
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(300.0, 500.0));
    for _ in 0..4 {
        ctx.run_ui(egui::RawInput { screen_rect: Some(screen), ..Default::default() }, |ui| show(&mut app, ui)).drop_without_applying_deltas();
    }
    assert_eq!(app.session.sel, sel);
    app.run("format.italic", json!({})).unwrap();
    assert_eq!(app.session.undo_depth(), 1);
    app.run("edit.undo", json!({})).unwrap();
    let state = app.execute("format.state", json!({})).unwrap();
    assert_eq!(state["bold"], false);
    assert_eq!(state["italic"], false);
}

#[test]
fn thesaurus_opens_from_focus_and_full_screen_without_trapping_the_keyboard() {
    for mode in ["view.focus", "view.fullScreenView"] {
        let mut app = app();
        let sel = app.session.sel.clone();
        app.run(mode, json!({"value": true})).unwrap();
        let mut h = harness(app);
        h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F7);
        h.run_steps(4);
        assert!(!h.state().session.view.focus_mode && !h.state().session.view.full_screen_view, "{mode}");
        assert!(h.state().session.view.thesaurus_pane);
        assert!(h.get_by_role_and_label(Role::TextInput, "Search for").is_focused());
        assert_eq!(h.state().session.sel, sel);
        assert_eq!(h.state().session.doc.plain_text(StoryRef::Body), "The small house.");
        assert_eq!(h.state().session.undo_depth(), 0);
        lookup(&mut h, "good");
        assert_eq!(h.state().session.sel, sel);
        for _ in 0..3 {
            if h.state().canvas.focused {
                break;
            }
            h.key_press(egui::Key::F6);
            h.run_steps(4);
        }
        assert!(h.state().canvas.focused, "F6 returns to the document: {mode}");
        h.key_press(egui::Key::ArrowRight);
        h.run_steps(4);
        assert!(h.state().session.sel.is_collapsed(), "navigation works after leaving the search field: {mode}");
        h.state_mut().run(mode, json!({"value": true})).unwrap();
        h.run_steps(4);
        h.key_press(egui::Key::Escape);
        h.run_steps(4);
        assert!(!h.state().session.view.focus_mode && !h.state().session.view.full_screen_view, "Escape works after lookup: {mode}");
    }
}

#[test]
fn thesaurus_pending_focus_does_not_block_escape_if_a_later_command_hides_panes() {
    for mode in ["view.focus", "view.fullScreenView"] {
        let mut app = app();
        app.run("review.thesaurus", json!({})).unwrap();
        app.execute(mode, json!({"value": true})).unwrap();
        let mut h = harness(app);
        assert!(h.state().thesaurus.focus_requested());
        assert!(!crate::panes::overlay_open(h.state(), &h.ctx), "a hidden owner cannot lock input");
        // Escape collapses a selection before it leaves a screen mode.
        for _ in 0..2 {
            h.key_press(egui::Key::Escape);
            h.run_steps(4);
        }
        assert!(!h.state().session.view.focus_mode && !h.state().session.view.full_screen_view, "{mode}");
        assert!(h.get_by_role_and_label(Role::TextInput, "Search for").is_focused());
        assert!(!h.state().thesaurus.focus_requested());
        assert_eq!(h.state().session.doc.plain_text(StoryRef::Body), "The small house.");
    }
    let mut app = app();
    app.ui.backstage = true;
    app.run("review.thesaurus", json!({})).unwrap();
    assert!(!app.ui.backstage, "a user lookup returns to the editing window before claiming focus");
    let h = harness(app);
    assert!(h.get_by_role_and_label(Role::TextInput, "Search for").is_focused());
}

#[test]
fn thesaurus_insert_can_be_repeated_with_f4_on_another_word() {
    let mut app = app();
    app.run("review.thesaurus", json!({})).unwrap();
    let mut h = harness(app);
    pane_inserts(&h).first().expect("the first synonym has an Insert action").click();
    h.run_steps(4);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body), "The little house.");
    h.state_mut().execute("select.text", json!({"text": "house"})).unwrap();
    let sel = h.state().session.sel.clone();
    h.key_press(egui::Key::F4);
    h.run_steps(4);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body), "The little little.");
    assert_eq!(h.state().session.undo_depth(), 2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    h.run_steps(4);
    assert_eq!(h.state().session.doc.plain_text(StoryRef::Body), "The little house.");
    assert_eq!(h.state().session.sel, sel);
}
