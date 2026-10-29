use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2, pos2, vec2};
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::{Services, WordApp, dialogs::Dialog};

fn harness() -> egui_kittest::Harness<'static, WordApp> {
    let text = format!("A small editing example.\n{}", "A line for scrolling.\n".repeat(120));
    let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text(&text)), Services::default());
    app.run("ui.language", json!({"value": "en"})).unwrap();
    app.run("ui.interfaceSize", json!({"percent": 100})).unwrap();
    app.run("select.text", json!({"text": "small"})).unwrap();
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

fn pointer(h: &mut egui_kittest::Harness<'_, WordApp>, at: Pos2, pressed: bool) {
    h.hover_at(at);
    h.event(Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
    h.run_steps(1);
}

fn click(h: &mut egui_kittest::Harness<'_, WordApp>, at: Pos2) {
    pointer(h, at, true);
    pointer(h, at, false);
    // Control input and layout settle over several frames, as in ui_shot pointer scenarios.
    h.run_steps(8);
}

fn bold_rect(h: &egui_kittest::Harness<'_, WordApp>) -> Rect {
    h.query_all_by_role_and_label(egui::accesskit::Role::Button, "Bold").min_by(|a, b| a.rect().top().total_cmp(&b.rect().top())).unwrap().rect()
}

fn bold(h: &mut egui_kittest::Harness<'_, WordApp>) -> bool {
    h.state_mut().execute("format.state", json!({})).unwrap()["bold"].as_bool().unwrap()
}

#[test]
fn modal_font_blocks_ribbon_clicks_but_programmatic_commands_still_run() {
    let mut h = harness();
    let ribbon = bold_rect(&h);
    assert!(!bold(&mut h));
    let selected = h.state().session.sel.clone();
    h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
    // First press arrives before egui has a previous-frame modal layer.
    click(&mut h, ribbon.center());
    click(&mut h, ribbon.center());
    assert!(!bold(&mut h));
    assert_eq!(h.state().session.sel, selected);
    assert_eq!(h.state().session.undo_depth(), 0);
    assert!(matches!(h.state().dialog, Some(Dialog::Font { .. })));
    h.state_mut().execute("format.bold", json!({})).unwrap();
    assert!(bold(&mut h), "modal ownership applies to user interaction, not engine.execute");
    h.key_press(Key::Escape);
    h.run_steps(8);
    assert!(h.state().dialog.is_none());
    click(&mut h, ribbon.center());
    assert!(!bold(&mut h), "background interaction resumes after dismissal");
}

#[test]
fn modal_font_blocks_document_pointer_drag_wheel_and_zoom() {
    let mut h = harness();
    h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
    h.run_steps(8);
    let viewport = h.state().canvas.viewport.unwrap();
    let start = pos2(viewport.left() + 24.0, viewport.top() + 35.0);
    let end = pos2(start.x + 65.0, viewport.bottom() - 8.0);
    let selected = h.state().session.sel.clone();
    let offset = h.state().canvas.scroll_offset;
    let zoom = h.state().session.view.zoom;
    pointer(&mut h, start, true);
    h.hover_at(end);
    h.run_steps(8);
    pointer(&mut h, end, false);
    for modifiers in [Modifiers::NONE, Modifiers::CTRL] {
        h.hover_at(start);
        h.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: vec2(0.0, -8.0),
            phase: egui::TouchPhase::Move,
            modifiers,
            source: egui::MouseWheelSource::Unknown,
        });
        h.run_steps(8);
    }
    assert_eq!(h.state().session.sel, selected);
    assert_eq!(h.state().canvas.scroll_offset, offset);
    assert_eq!(h.state().session.view.zoom, zoom);
    assert!(!h.state().canvas.drag_selecting());
    assert_eq!(h.state().session.undo_depth(), 0);
}

#[test]
fn modal_ownership_cancels_a_document_drag_already_in_progress() {
    let mut h = harness();
    let viewport = h.state().canvas.viewport.unwrap();
    let page = h.state().canvas.page_rects[0];
    let start = pos2(page.left() + 110.0, page.top() + 105.0);
    pointer(&mut h, start, true);
    h.hover_at(start + vec2(50.0, 20.0));
    h.run_steps(4);
    assert!(h.state().canvas.drag_selecting(), "the regression must begin with a live document drag");
    h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
    let selection = h.state().session.sel.clone();
    h.hover_at(viewport.right_bottom() - Vec2::splat(8.0));
    h.run_steps(8);
    assert_eq!(h.state().session.sel, selection);
    assert!(!h.state().canvas.drag_selecting());
    pointer(&mut h, start, false);
    h.key_press(Key::Escape);
    h.run_steps(8);
    assert_eq!(h.state().session.sel, selection, "the blocked gesture cannot resume after closing");
}

#[test]
fn modal_font_blocks_docked_and_floating_pane_actions() {
    for floating in [false, true] {
        let mut h = harness();
        h.state_mut().run("edit.copy", json!({})).unwrap();
        h.state_mut().run("edit.clipboardPane", json!({"value": true})).unwrap();
        if floating {
            h.state_mut().run("ui.paneDock", json!({"pane": "clipboard", "dock": "float"})).unwrap();
            h.state_mut().ui.pane_places.entry("clipboard".into()).or_default().float = Some([1100.0, 250.0, 300.0, 400.0]);
        }
        h.run_steps(8);
        let delete = h.get_by_label("Delete").rect();
        let count = h.state().session.clip_history.items().len();
        h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
        h.run_steps(8);
        let modal = h.ctx.memory(|m| m.area_rect(egui::Id::unique(("dialog", "Font")))).unwrap();
        assert!(!modal.contains(delete.center()), "the pointer must hit the pane, not the dialog");
        click(&mut h, delete.center());
        assert_eq!(h.state().session.clip_history.items().len(), count);
        assert!(matches!(h.state().dialog, Some(Dialog::Font { .. })));
    }
}

#[test]
fn modeless_dialogs_leave_ribbon_and_document_pointer_input_available() {
    for name in ["find", "goto", "markEntry"] {
        let mut h = harness();
        let ribbon = bold_rect(&h);
        h.state_mut().run("ui.dialog", json!({"name": name})).unwrap();
        h.run_steps(8);
        assert!(!super::background_blocked(h.state()));
        click(&mut h, ribbon.center());
        assert!(bold(&mut h), "{name} leaves ribbon formatting available");
        let selection = h.state().session.sel.clone();
        let page = h.state().canvas.page_rects[0];
        let at = pos2(page.left() + 12.0, page.top() + 110.0);
        click(&mut h, at);
        assert_ne!(h.state().session.sel, selection, "{name} leaves document pointer selection available");
        assert!(h.state().dialog.is_some());
    }
}

#[test]
fn nested_colors_owns_parent_pointer_and_escape_until_it_closes() {
    let mut h = harness();
    h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
    h.run_steps(8);
    let parent = h.ctx.memory(|m| m.area_rect(egui::Id::unique(("dialog", "Font")))).unwrap();
    let close = pos2(parent.right() - 10.0, parent.top() + 12.0);
    h.state_mut().colors = Some(crate::colors::ColorsDialog::new(None, crate::colors::Target::Field(egui::Id::unique("test_nested_color"))));
    h.run_steps(8);
    click(&mut h, close);
    assert!(matches!(h.state().dialog, Some(Dialog::Font { .. })), "child blocks its parent's caption close");
    // Clicking the Colors backdrop may close the child. Reopen it for the keyboard assertion.
    h.state_mut().colors = Some(crate::colors::ColorsDialog::new(None, crate::colors::Target::Field(egui::Id::unique("test_nested_color"))));
    h.run_steps(8);
    h.key_press(Key::Escape);
    h.run_steps(8);
    assert!(h.state().colors.is_none());
    assert!(matches!(h.state().dialog, Some(Dialog::Font { .. })), "Escape belongs only to the child");
    h.key_press(Key::Escape);
    h.run_steps(8);
    assert!(h.state().dialog.is_none());
    assert_eq!(h.state().session.undo_depth(), 0);
}

#[test]
fn disabled_panes_keep_pending_focus_until_the_modal_closes() {
    for thesaurus in [false, true] {
        let mut h = harness();
        if thesaurus {
            h.state_mut().run("review.thesaurus", json!({})).unwrap();
        } else {
            h.state_mut().run("view.navigationPane", json!({"value": true})).unwrap();
            h.state_mut().nav_search_focus = true;
        }
        h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
        let selection = h.state().session.sel.clone();
        h.run_steps(8);
        if thesaurus {
            assert!(h.state().thesaurus.focus_requested(), "a disabled pane cannot consume pending focus");
        } else {
            assert!(h.state().nav_search_focus, "navigation must wait before searching or taking focus");
        }
        assert_eq!(h.state().session.sel, selection);
        h.key_press(Key::Escape);
        h.run_steps(8);
        assert!(h.state().dialog.is_none());
        if thesaurus {
            assert!(!h.state().thesaurus.focus_requested());
            assert!(h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Search for").is_focused());
        } else {
            assert!(!h.state().nav_search_focus);
            let search = h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Search document");
            assert!(search.is_focused());
            assert_eq!(search.value().as_deref(), Some("small"));
        }
    }
}

#[test]
fn disabled_thesaurus_does_not_consume_the_modal_enter_key() {
    for reopen in [false, true] {
        let mut h = harness();
        if reopen {
            // A fresh egui Window has an invisible, disabled sizing pass. Cache its geometry
            // first so the immediate-open Enter branch tests input ownership, not that pass.
            h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
            h.run_steps(8);
            h.key_press(Key::Escape);
            h.run_steps(8);
            assert!(h.state().dialog.is_none());
        }
        h.state_mut().run("review.thesaurus", json!({})).unwrap();
        h.run_steps(8);
        let input = h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Search for");
        input.click();
        h.run_steps(2);
        h.key_press_modifiers(Modifiers::COMMAND, Key::A);
        h.run_steps(2);
        h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Search for").type_text("good");
        h.run_steps(4);
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, "tiny").is_some(), "the draft query has not run");
        let selection = h.state().session.sel.clone();
        h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
        if !reopen {
            h.run_steps(8);
        }
        h.key_press(Key::Enter);
        h.run_steps(8);
        assert!(h.state().dialog.is_none(), "Enter reaches Font's OK action, reopening: {reopen}");
        assert_eq!(h.state().session.sel, selection);
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, "tiny").is_some());
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, "fine").is_none(), "the disabled pane did not perform its draft lookup");
    }
}

#[test]
fn command_palette_does_not_take_focus_from_nested_colors() {
    let mut h = harness();
    h.state_mut().run("ui.dialog", json!({"name": "commands"})).unwrap();
    h.run_steps(8);
    h.state_mut().colors = Some(crate::colors::ColorsDialog::new(None, crate::colors::Target::Field(egui::Id::unique("palette_child"))));
    h.run_steps(8);
    h.key_press(Key::Escape);
    h.run_steps(8);
    assert!(h.state().colors.is_none());
    assert!(matches!(h.state().dialog, Some(Dialog::Commands { .. })), "only the child consumes Escape");
    assert_eq!(h.state().session.undo_depth(), 0);
    h.key_press(Key::Escape);
    h.run_steps(8);
    assert!(h.state().dialog.is_none());
}

#[test]
fn a_new_modal_blocks_the_independent_read_aloud_window() {
    let mut h = harness();
    // Showing an idle player needs no speech service or external process.
    h.state_mut().session.read_aloud.open = true;
    h.run_steps(8);
    let player = h.ctx.memory(|memory| memory.area_rect(egui::Id::unique("read-aloud-player"))).unwrap();
    let close = h.query_all_by_label("Close").find(|node| player.contains_rect(node.rect())).unwrap().rect().center();
    pointer(&mut h, close, true);
    h.state_mut().run("ui.dialog", json!({"name": "font"})).unwrap();
    pointer(&mut h, close, false);
    h.run_steps(8);
    assert!(h.state().session.read_aloud.open, "the first modal frame must block the cached caption-close response");
    assert!(matches!(h.state().dialog, Some(Dialog::Font { .. })));
    h.key_press(Key::Escape);
    h.run_steps(8);
    click(&mut h, close);
    assert!(!h.state().session.read_aloud.open, "the same pointer action works after the modal closes");
}
