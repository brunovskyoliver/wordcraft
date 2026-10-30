//! Tools › Customize Ribbon (#379; File › Options, command search, right-click on the ribbon or
//! the Quick Access Toolbar): two pages. Ribbon: commands by category on the left, the ribbon's
//! tabs with their custom groups on the right; show or hide tabs, add custom tabs and groups,
//! put commands in custom groups, rename, reorder, remove, reset. Quick Access Toolbar: the same
//! command list next to the toolbar's commands. Every change runs `tools.customizeRibbon`, so
//! agents get the same result without the dialog; the layout persists with the preferences.

use egui::{Ui, vec2};
use serde::Serialize;
use serde_json::{Value, json};

use crate::WordApp;
use crate::dialogs::Dialog;
use crate::dialogs_keyboard::{categories, commands, list, matching_commands, select_in};
use crate::theme::semibold;

/// The dialog's two pages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Page {
    #[default]
    Ribbon,
    QuickAccess,
}

/// An item in the ribbon tree: a tab, one of its custom groups, or a command in a group.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TreeItem {
    pub tab: String,
    pub group: Option<String>,
    pub command: Option<String>,
}

/// The dialog's state.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RibbonForm {
    pub page: Page,
    /// The left list: a category (empty for All Commands) and the command picked in it.
    pub category: String,
    pub command: String,
    /// Filter-as-you-type query for the command list (#1845); UI-only dialog state, ignored
    /// while empty. Never persisted and never a command parameter.
    pub filter: String,
    /// The item picked in the ribbon tree.
    pub selected: TreeItem,
    /// The command picked in the Quick Access Toolbar list.
    pub qat_selected: String,
    /// Rename… asked: the name being typed.
    pub renaming: Option<String>,
    /// Reset asked; waiting for Yes / No.
    pub confirm_reset: bool,
    /// Why the last change didn't happen.
    pub message: String,
}

impl RibbonForm {
    pub fn read(app: &WordApp, page: Page) -> RibbonForm {
        let category = categories(app).into_iter().next().unwrap_or_default();
        let command = commands(app, &category).first().map(|(id, _)| (*id).to_string()).unwrap_or_default();
        let selected = TreeItem { tab: app.session.ribbon.tabs().first().map(|t| t.name.clone()).unwrap_or_default(), ..Default::default() };
        RibbonForm { page, category, command, selected, ..Default::default() }
    }
}

/// Open the dialog on a page.
pub fn open(app: &mut WordApp, page: Page) {
    app.dialog = Some(Dialog::CustomizeRibbon { form: Box::new(RibbonForm::read(app, page)) });
}

/// Run `tools.customizeRibbon`; the result, or why it failed (shown in the dialog).
fn change(app: &mut WordApp, f: &mut RibbonForm, params: Value) -> Option<Value> {
    match app.session.run("tools.customizeRibbon", &params) {
        Ok(v) => {
            f.message.clear();
            Some(v)
        }
        Err(e) => {
            f.message = e.to_string();
            None
        }
    }
}

fn label_of(app: &WordApp, id: &str) -> String {
    app.session.registry.get(id).map_or_else(|| id.to_string(), |s| tl!(s.label).to_string())
}

const PICKER_SECTION_GAP: f32 = 4.0;

/// Widths shared by the Ribbon and Quick Access Toolbar pages. Keep their usual columns when
/// they fit, then share the remaining room between the lists. Below a usable category width,
/// put the destination below the command picker; the dialog host supplies vertical scrolling.
struct CustomizationLayout {
    width: f32,
    category: f32,
    commands: f32,
    destination: f32,
    add_button: f32,
    move_button: f32,
    stacked: bool,
}

impl CustomizationLayout {
    fn for_ui(ui: &Ui) -> Self {
        let gap = ui.spacing().item_spacing.x;
        // `dialogs_keyboard::list` and `tree` use this frame around each list.
        let list_margin = egui::Frame::new().stroke(ui.visuals().widgets.noninteractive.bg_stroke).inner_margin(2.0).total_margin().sum().x;
        let window_margin = egui::Frame::window(ui.style()).total_margin().sum().x;
        let available = (ui.ctx().content_rect().width() - window_margin - ui.spacing().scroll.allocated_width()).max(1.0);
        let button_width = |labels: &[&str], minimum: f32| {
            let font = egui::TextStyle::Button.resolve(ui.style());
            labels.iter().fold(minimum, |width, label| {
                let text = ui.painter().layout_no_wrap(tl!(label).to_string(), font.clone(), ui.visuals().text_color());
                width.max(text.size().x + 2.0 * ui.spacing().button_padding.x)
            })
        };
        let add_button = button_width(&["Add >>", "<< Remove"], 96.0);
        let move_button = button_width(&["Up", "Down"], 56.0);
        let fixed = add_button + move_button + 3.0 * list_margin + 4.0 * gap + 2.0 * PICKER_SECTION_GAP;
        let width = available.min(150.0 + 230.0 + 250.0 + fixed);
        let scale = ((width - fixed) / (150.0 + 230.0 + 250.0)).clamp(0.0, 1.0);
        let stacked = 150.0 * scale < 100.0;
        let (category, commands, destination) = if stacked {
            let source = (width - 2.0 * list_margin - gap).max(1.0);
            let destination = (width - add_button - move_button - list_margin - 2.0 * gap - PICKER_SECTION_GAP).max(1.0);
            (source * 150.0 / 380.0, source * 230.0 / 380.0, destination)
        } else {
            (150.0 * scale, 230.0 * scale, 250.0 * scale)
        };
        Self { width, category, commands, destination, add_button, move_button, stacked }
    }
}

/// List rows keep their full accessible label, but a long command name must not widen its
/// assigned column or wrap past the virtual row height.
fn picker_list(ui: &mut Ui, id: &str, size: egui::Vec2, rows: &[(String, String)], selected: &str) -> Option<String> {
    ui.scope(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        list(ui, id, size, rows, selected)
    })
    .inner
}

fn command_picker(app: &WordApp, ui: &mut Ui, f: &mut RibbonForm, layout: &CustomizationLayout, h: f32) {
    let mut cats: Vec<(String, String)> = categories(app).into_iter().map(|c| (c.clone(), tl!(&c).to_string())).collect();
    cats.push((String::new(), tl!("All Commands").to_string()));
    let cmds: Vec<(String, String)> = matching_commands(app, &f.category, &f.filter);
    f.command = select_in(&cmds, &f.command);
    if cmds.is_empty() {
        ui.label(egui::RichText::new(tl!("No matching commands")).weak());
    }
    ui.label(egui::RichText::new(tl!("Choose a command")).font(semibold(12.5)));
    ui.horizontal_top(|ui| {
        if let Some(c) = picker_list(ui, "rb_categories", vec2(layout.category, h), &cats, &f.category) {
            f.category = c;
            f.filter.clear();
            f.command = matching_commands(app, &f.category, &f.filter).first().map(|(id, _)| id.clone()).unwrap_or_default();
        }
        if let Some(c) = picker_list(ui, "rb_commands", vec2(layout.commands, h), &cmds, &f.command) {
            f.command = c;
        }
    });
}

fn destination_page(app: &mut WordApp, ui: &mut Ui, f: &mut RibbonForm, layout: &CustomizationLayout, h: f32) {
    match f.page {
        Page::Ribbon => ribbon_page(app, ui, f, layout, h),
        Page::QuickAccess => qat_page(app, ui, f, layout, h),
    }
}

/// The dialog body; true closes it.
pub fn customize_ribbon(app: &mut WordApp, ui: &mut Ui, f: &mut RibbonForm) -> bool {
    let layout = CustomizationLayout::for_ui(ui);
    ui.set_width(layout.width);
    ui.horizontal_wrapped(|ui| {
        for (page, name) in [(Page::Ribbon, "Ribbon"), (Page::QuickAccess, "Quick Access Toolbar")] {
            if ui.selectable_label(f.page == page, tl!(name)).clicked() && f.page != page {
                f.page = page;
                f.renaming = None;
                f.confirm_reset = false;
                f.message.clear();
            }
        }
    });
    ui.separator();
    let h = 300.0;
    let search = ui.add(
        egui::TextEdit::singleline(&mut f.filter)
            .hint_text(tl!("Search"))
            .desired_width(layout.category + layout.commands + ui.spacing().item_spacing.x),
    );
    crate::widgets::named_input(search, tl!("Search"));
    if layout.stacked {
        command_picker(app, ui, f, &layout, h);
        ui.separator();
        ui.horizontal_top(|ui| destination_page(app, ui, f, &layout, h));
    } else {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| command_picker(app, ui, f, &layout, h));
            ui.add_space(PICKER_SECTION_GAP);
            destination_page(app, ui, f, &layout, h);
        });
    }
    if let Some(spec) = app.session.registry.get(&f.command) {
        ui.add(egui::Label::new(egui::RichText::new(format!("{} · {}", crate::i18n::location(spec.location), spec.id)).small().weak()).wrap());
    }
    if !f.message.is_empty() {
        ui.colored_label(ui.visuals().warn_fg_color, f.message.as_str());
    }
    ui.add_space(6.0);
    if f.confirm_reset {
        let ask = match f.page {
            Page::Ribbon => tl!("Remove every custom tab and group and show all tabs again?"),
            Page::QuickAccess => tl!("Put the default commands back on the Quick Access Toolbar?"),
        };
        ui.label(ask);
        ui.horizontal(|ui| {
            if ui.button(tl!("Yes")).clicked() {
                let what = if f.page == Page::Ribbon { "ribbon" } else { "qat" };
                change(app, f, json!({"reset": what}));
                f.selected = TreeItem { tab: app.session.ribbon.tabs().first().map(|t| t.name.clone()).unwrap_or_default(), ..Default::default() };
                f.confirm_reset = false;
            }
            if ui.button(tl!("No")).clicked() {
                f.confirm_reset = false;
            }
        });
        return false;
    }
    let mut close = false;
    ui.horizontal(|ui| {
        let custom = match f.page {
            Page::Ribbon => !app.session.ribbon.ribbon_is_default(),
            Page::QuickAccess => app.session.ribbon.qat.is_some(),
        };
        if ui.add_enabled(custom, egui::Button::new(tl!("Reset…"))).clicked() {
            f.confirm_reset = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(tl!("Close")).clicked() {
                close = true;
            }
        });
    });
    close
}

/// A row in the ribbon tree: the item, its indent level, its text, and (tabs) whether it shows.
struct Row {
    item: TreeItem,
    depth: u8,
    text: String,
    shown: Option<bool>,
}

fn tree_rows(app: &WordApp) -> Vec<Row> {
    let custom = tl!("(Custom)");
    let mut rows = Vec::new();
    for t in app.session.ribbon.tabs().iter() {
        let text = if t.custom { format!("{} {custom}", t.name) } else { tl!(&t.name).to_string() };
        rows.push(Row { item: TreeItem { tab: t.name.clone(), group: None, command: None }, depth: 0, text, shown: Some(!t.hidden) });
        for g in &t.groups {
            let item = TreeItem { tab: t.name.clone(), group: Some(g.name.clone()), command: None };
            rows.push(Row { item, depth: 1, text: format!("{} {custom}", g.name), shown: None });
            for id in &g.commands {
                let item = TreeItem { tab: t.name.clone(), group: Some(g.name.clone()), command: Some(id.clone()) };
                rows.push(Row { item, depth: 2, text: label_of(app, id), shown: None });
            }
        }
    }
    rows
}

/// The params naming a tree item (`{tab, group?, command?}`).
fn address(item: &TreeItem) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("tab".into(), json!(item.tab));
    if let Some(g) = &item.group {
        m.insert("group".into(), json!(g));
    }
    if let Some(c) = &item.command {
        m.insert("command".into(), json!(c));
    }
    m
}

fn ribbon_page(app: &mut WordApp, ui: &mut Ui, f: &mut RibbonForm, layout: &CustomizationLayout, h: f32) {
    let rows = tree_rows(app);
    if !rows.iter().any(|r| r.item == f.selected) {
        f.selected = rows.first().map(|r| r.item.clone()).unwrap_or_default();
    }
    let sel = f.selected.clone();
    let tab_custom = app.session.ribbon.tabs().iter().any(|t| t.name == sel.tab && t.custom);
    // Add >> / << Remove.
    ui.vertical(|ui| {
        ui.add_space(h / 2.0 - 10.0);
        let can_add = sel.group.is_some() && !f.command.is_empty();
        let add = ui.add_enabled(can_add, egui::Button::new(tl!("Add >>")).min_size(vec2(layout.add_button, 0.0)));
        if add.on_disabled_hover_text(tl!("Pick a custom group (New Group makes one)")).clicked() {
            let mut m = address(&sel);
            m.remove("command");
            m.insert("command".into(), json!(f.command));
            if let Some(after) = &sel.command {
                m.insert("after".into(), json!(after));
            }
            if change(app, f, json!({"add": m})).is_some() {
                f.selected = TreeItem { command: Some(f.command.clone()), ..sel.clone() };
            }
        }
        let can_remove = sel.group.is_some() || tab_custom;
        if ui.add_enabled(can_remove, egui::Button::new(tl!("<< Remove")).min_size(vec2(layout.add_button, 0.0))).clicked()
            && change(app, f, json!({"remove": address(&sel)})).is_some()
        {
            f.selected = match (&sel.group, &sel.command) {
                (Some(_), Some(_)) => TreeItem { command: None, ..sel.clone() },
                _ => TreeItem { tab: sel.tab.clone(), ..Default::default() },
            };
        }
    });
    ui.add_space(PICKER_SECTION_GAP);
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(tl!("Ribbon tabs")).font(semibold(12.5)));
        ui.horizontal_top(|ui| {
            tree(app, ui, f, &rows, vec2(layout.destination, h));
            ui.vertical(|ui| {
                ui.add_space(h / 2.0 - 10.0);
                for (text, by) in [("Up", -1), ("Down", 1)] {
                    if ui.add(egui::Button::new(tl!(text)).min_size(vec2(layout.move_button, 0.0))).clicked() {
                        let mut m = address(&sel);
                        m.insert("by".into(), json!(by));
                        change(app, f, json!({"move": m}));
                    }
                }
            });
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button(tl!("New Tab")).clicked()
                && let Some(v) = change(app, f, json!({"newTab": {"after": sel.tab}}))
                && let Some(tab) = v.get("tab").and_then(Value::as_str)
            {
                f.selected = TreeItem { tab: tab.to_string(), group: Some("New Group".into()), command: None };
            }
            if ui.button(tl!("New Group")).clicked()
                && let Some(v) = change(app, f, json!({"newGroup": {"tab": sel.tab}}))
                && let Some(g) = v.get("group").and_then(Value::as_str)
            {
                f.selected = TreeItem { tab: sel.tab.clone(), group: Some(g.to_string()), command: None };
            }
            let can_rename = sel.command.is_none() && (sel.group.is_some() || tab_custom);
            if ui.add_enabled(can_rename, egui::Button::new(tl!("Rename…"))).clicked() {
                f.renaming = Some(sel.group.clone().unwrap_or_else(|| sel.tab.clone()));
            }
        });
        let mut done = None;
        if let Some(name) = &mut f.renaming {
            ui.horizontal_wrapped(|ui| {
                ui.label(tl!("Name:"));
                let r = ui.add(egui::TextEdit::singleline(name).desired_width(140.0));
                let enter = r.lost_focus() && crate::modal_input::key_input(ui, |i| i.key_pressed(egui::Key::Enter));
                // macOS puts Cancel before OK.
                let mac = cfg!(target_os = "macos");
                if mac && ui.button(tl!("Cancel")).clicked() {
                    done = Some(None);
                }
                if ui.button(tl!("OK")).clicked() || enter {
                    done = Some(Some(name.clone()));
                }
                if !mac && ui.button(tl!("Cancel")).clicked() {
                    done = Some(None);
                }
            });
        }
        if let Some(answer) = done {
            f.renaming = None;
            if let Some(to) = answer {
                let mut m = address(&sel);
                m.insert("to".into(), json!(to));
                if let Some(v) = change(app, f, json!({"rename": m}))
                    && let Some(name) = v.get("name").and_then(Value::as_str)
                {
                    f.selected = match &sel.group {
                        Some(_) => TreeItem { group: Some(name.to_string()), ..sel.clone() },
                        None => TreeItem { tab: name.to_string(), ..Default::default() },
                    };
                }
            }
        }
    });
}

/// The ribbon tree: tabs with a box to show or hide them, their custom groups and commands.
fn tree(app: &mut WordApp, ui: &mut Ui, f: &mut RibbonForm, rows: &[Row], size: egui::Vec2) {
    egui::Frame::new().stroke(ui.visuals().widgets.noninteractive.bg_stroke).inner_margin(2.0).show(ui, |ui| {
        ui.set_width(size.x);
        ui.set_height(size.y);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        egui::ScrollArea::vertical().id_salt("rb_tree").auto_shrink([false, false]).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| tree_rows_ui(app, ui, f, rows));
        });
    });
}

fn tree_rows_ui(app: &mut WordApp, ui: &mut Ui, f: &mut RibbonForm, rows: &[Row]) {
    for row in rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            // Groups and commands line up under the tab names (past the tabs' boxes).
            ui.add_space(if row.depth == 0 { 0.0 } else { 24.0 + f32::from(row.depth.saturating_sub(1)) * 16.0 });
            if let Some(shown) = row.shown {
                let mut on = shown;
                if ui.checkbox(&mut on, "").accessible_name(&row.text).changed() {
                    let key = if on { "show" } else { "hide" };
                    change(app, f, json!({ key: row.item.tab }));
                }
            }
            let resp = ui.add(egui::Button::selectable(f.selected == row.item, row.text.as_str()));
            if resp.clicked() {
                f.selected = row.item.clone();
                f.renaming = None;
            }
        });
    }
}

fn qat_page(app: &mut WordApp, ui: &mut Ui, f: &mut RibbonForm, layout: &CustomizationLayout, h: f32) {
    let ids: Vec<String> = app.session.ribbon.qat().into_iter().map(str::to_string).collect();
    if !ids.contains(&f.qat_selected) {
        f.qat_selected = ids.first().cloned().unwrap_or_default();
    }
    ui.vertical(|ui| {
        ui.add_space(h / 2.0 - 10.0);
        let can_add = !f.command.is_empty() && !ids.contains(&f.command);
        if ui.add_enabled(can_add, egui::Button::new(tl!("Add >>")).min_size(vec2(layout.add_button, 0.0))).clicked()
            && change(app, f, json!({"qatAdd": f.command})).is_some()
        {
            f.qat_selected = f.command.clone();
        }
        if ui.add_enabled(!f.qat_selected.is_empty(), egui::Button::new(tl!("<< Remove")).min_size(vec2(layout.add_button, 0.0))).clicked() {
            change(app, f, json!({"qatRemove": f.qat_selected}));
        }
    });
    ui.add_space(PICKER_SECTION_GAP);
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(tl!("Quick Access Toolbar")).font(semibold(12.5)));
        ui.horizontal_top(|ui| {
            let rows: Vec<(String, String)> = ids.iter().map(|id| (id.clone(), label_of(app, id))).collect();
            if let Some(id) = picker_list(ui, "rb_qat", vec2(layout.destination, h), &rows, &f.qat_selected) {
                f.qat_selected = id;
            }
            ui.vertical(|ui| {
                ui.add_space(h / 2.0 - 10.0);
                for (text, by) in [("Up", -1), ("Down", 1)] {
                    if ui.add_enabled(!f.qat_selected.is_empty(), egui::Button::new(tl!(text)).min_size(vec2(layout.move_button, 0.0))).clicked() {
                        change(app, f, json!({"qatMove": {"command": f.qat_selected, "by": by}}));
                    }
                }
            });
        });
        // The same switch as the toolbar menu's Hide Quick Access Toolbar (#1945).
        ui.add_space(4.0);
        let mut shown = app.ui.quick_access_toolbar;
        if ui.checkbox(&mut shown, tl!("Show Quick Access Toolbar")).changed() {
            let _ = app.run("ui.quickAccessToolbar", json!({"value": shown}));
        }
    });
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wordcraft_engine::Session;

    use crate::dialogs::Dialog;
    use crate::{Services, WordApp};

    fn app() -> WordApp {
        WordApp::new(Session::new(wordcraft_doc::Document::new()), Services::default())
    }

    /// #379: Tools › Customize Ribbon opens the dialog; a custom tab with a group, a hidden tab and
    /// the Quick Access Toolbar survive a restart, and a damaged saved layout doesn't lose the
    /// other preferences.
    #[test]
    fn customize_ribbon_opens_and_the_layout_persists() {
        let mut a = app();
        a.run("tools.customizeRibbon", json!({})).unwrap();
        assert!(matches!(a.dialog, Some(Dialog::CustomizeRibbon { .. })));
        a.run("tools.customizeRibbon", json!({"newTab": {"name": "Mine"}, "hide": "Draw", "qatAdd": "format.bold"})).unwrap();
        a.run("tools.customizeRibbon", json!({"add": {"tab": "Mine", "group": "New Group", "command": "format.italic"}})).unwrap();
        let saved = serde_json::to_string(&a.prefs()).unwrap();
        let mut b = app();
        b.apply_prefs(serde_json::from_str(&saved).unwrap());
        assert_eq!(b.session.ribbon, a.session.ribbon);
        let tabs = b.session.ribbon.tabs();
        let mine = tabs.iter().find(|t| t.name == "Mine").unwrap();
        assert_eq!(mine.groups[0].commands, ["format.italic"]);
        assert!(tabs.iter().any(|t| t.name == "Draw" && t.hidden));
        assert_eq!(b.session.ribbon.qat(), ["file.save", "edit.undo", "edit.redo", "format.bold"]);

        let mut c = app();
        c.apply_prefs(serde_json::from_str(r#"{"ribbon": {"tabs": 3, "qat": ["gone.command", "edit.undo"]}}"#).unwrap());
        assert!(c.session.ribbon.ribbon_is_default());
        assert_eq!(c.session.ribbon.qat(), ["edit.undo"]);
    }

    /// A context with the interface fonts and theme (they take effect from the next frame).
    fn ctx() -> egui::Context {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::apply(&ctx, crate::theme::Appearance::Light);
        ctx.run_ui(egui::RawInput::default(), |_| {}).drop_without_applying_deltas();
        ctx
    }

    /// One frame at 60 fps with these input events.
    fn frame(ctx: &egui::Context, n: &mut u32, events: Vec<egui::Event>, draw: impl FnMut(&mut egui::Ui)) {
        *n += 1;
        let input = egui::RawInput { time: Some(f64::from(*n) / 60.0), predicted_dt: 1.0 / 60.0, events, ..Default::default() };
        ctx.run_ui(input, draw).drop_without_applying_deltas();
    }

    /// A click at `pos` over two frames.
    fn click(ctx: &egui::Context, n: &mut u32, pos: egui::Pos2, mut draw: impl FnMut(&mut egui::Ui)) {
        let button = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        frame(ctx, n, vec![egui::Event::PointerMoved(pos), button(true)], &mut draw);
        frame(ctx, n, vec![button(false)], &mut draw);
    }

    /// #379: a custom tab shows its group's commands as working buttons, and a hidden tab isn't
    /// shown (the ribbon falls back to a visible one).
    #[test]
    fn custom_tabs_draw_their_commands_and_hidden_tabs_dont_show() {
        let ctx = ctx();
        let mut n = 0;
        let mut a = app();
        a.run("tools.customizeRibbon", json!({"newTab": {"name": "Mine"}, "hide": "Home"})).unwrap();
        a.run("tools.customizeRibbon", json!({"add": {"tab": "Mine", "group": "New Group", "command": "view.ruler"}})).unwrap();
        a.ui.tab = "Mine".into();
        let ruler = a.session.view.ruler;
        frame(&ctx, &mut n, vec![], |ui| crate::ribbon::show(&mut a, ui));
        // The group's one command is a large button at the ribbon's left edge, under the tab strip.
        click(&ctx, &mut n, egui::pos2(24.0, 60.0), |ui| crate::ribbon::show(&mut a, ui));
        assert_eq!(a.ui.tab, "Mine");
        assert_ne!(a.session.view.ruler, ruler, "the custom group's button runs its command");

        a.ui.tab = "Home".into();
        frame(&ctx, &mut n, vec![], |ui| crate::ribbon::show(&mut a, ui));
        assert_eq!(a.ui.tab, "Insert", "a hidden tab isn't shown");
    }

    /// #1845: the shared command picker (Ribbon and Quick Access Toolbar pages) filters
    /// as you type, and Add works from a filtered selection.
    #[test]
    fn command_filter_narrows_the_picker_and_add_works() {
        use crate::dialogs_keyboard::{matching_commands, select_in};

        let mut a = app();
        a.run("tools.customizeRibbon", json!({})).unwrap();
        assert!(matches!(a.dialog, Some(Dialog::CustomizeRibbon { .. })));
        if let Some(Dialog::CustomizeRibbon { form }) = &mut a.dialog {
            form.filter = "bold".to_string();
        }
        let (category, filter, current) = match &a.dialog {
            Some(Dialog::CustomizeRibbon { form }) => (form.category.clone(), form.filter.clone(), form.command.clone()),
            _ => panic!("expected the Customize Ribbon dialog"),
        };
        // Empty filter keeps the category scope.
        let unfiltered = matching_commands(&a, &category, "");
        assert!(!unfiltered.is_empty());
        // Typing searches every command, ignoring the category.
        let cmds = matching_commands(&a, &category, &filter);
        assert!(cmds.iter().any(|(id, _)| id == "format.bold"));
        assert!(cmds.len() < a.session.registry.all().len());
        // The dialog's Add >> adds the filtered pick to the Quick Access Toolbar.
        let picked = select_in(&cmds, &current);
        assert!(!picked.is_empty());
        if let Some(Dialog::CustomizeRibbon { form }) = &mut a.dialog {
            form.command = picked.clone();
        }
        a.run("tools.customizeRibbon", json!({"qatAdd": picked})).unwrap();
        assert!(a.session.ribbon.qat().contains(&picked.as_str()));
        assert!(a.session.ribbon.qat().contains(&"format.bold"));
    }

    /// #376: the Quick Access Toolbar's … button opens its menu, and the toolbar shows the user's
    /// commands.
    #[test]
    fn quick_access_menu_opens_and_commands_come_and_go() {
        let ctx = ctx();
        let mut n = 0;
        let mut a = app();
        frame(&ctx, &mut n, vec![], |ui| crate::chrome::title_bar(&mut a, ui));
        let more = ctx.read_response(crate::chrome::qat_menu_button_id()).unwrap();
        let popup = egui::Popup::default_response_id(&more);
        assert!(!egui::Popup::is_id_open(&ctx, popup));
        click(&ctx, &mut n, more.rect.center(), |ui| crate::chrome::title_bar(&mut a, ui));
        assert!(egui::Popup::is_id_open(&ctx, popup), "the … button opens the Customize Quick Access Toolbar menu");

        a.run("tools.customizeRibbon", json!({"qatAdd": "file.new"})).unwrap();
        assert!(a.session.ribbon.qat().contains(&"file.new"));
        a.run("tools.customizeRibbon", json!({"qatRemove": "file.new"})).unwrap();
        assert_eq!(a.session.ribbon.qat(), ["file.save", "edit.undo", "edit.redo"]);
    }

    fn customization_harness(width: f32, page: super::Page) -> egui_kittest::Harness<'static, WordApp> {
        let mut a = app();
        a.run("ui.language", json!({"value": "en"})).unwrap();
        super::open(&mut a, page);
        if let Some(Dialog::CustomizeRibbon { form }) = &mut a.dialog {
            form.filter = "format.bold".into();
            form.command = "format.bold".into();
        }
        egui_kittest::Harness::builder().with_size(egui::vec2(width, 600.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        )
    }

    /// AUD-012: both customization pages keep their window and controls inside the viewport,
    /// including after resizing across the stacked layout boundary with the dialog open.
    #[test]
    fn customization_pages_stay_within_the_window_when_resized() {
        use egui_kittest::kittest::Queryable;
        for page in [super::Page::Ribbon, super::Page::QuickAccess] {
            let mut h = customization_harness(1440.0, page);
            for width in [1440.0, 800.0, 640.0, 520.0, 800.0, 1440.0] {
                h.set_size(egui::vec2(width, 600.0));
                h.run_steps(8);
                let window = h.ctx.memory(|m| m.area_rect(egui::Id::unique(("dialog", "Customize Ribbon")))).unwrap();
                assert!(window.left() >= -0.5 && window.right() <= width + 0.5, "{page:?} at {width}: {window:?}");
                for label in ["Add >>", "<< Remove", "Up", "Down", "Reset…"] {
                    let rect = h.get_by_label(label).rect();
                    assert!(rect.left() >= -0.5 && rect.right() <= width + 0.5, "{page:?} at {width}: {label} {rect:?}");
                }
                let close = h.query_all_by_label("Close").max_by(|a, b| a.rect().bottom().total_cmp(&b.rect().bottom())).unwrap();
                assert!(close.rect().left() >= -0.5 && close.rect().right() <= width + 0.5);
            }
        }
    }

    /// AUD-012: the compact controls still dispatch add/move/remove and visibility changes;
    /// a visible footer Close ends the dialog after using either arrangement.
    #[test]
    fn compact_quick_access_controls_edit_the_toolbar_and_close() {
        use egui_kittest::kittest::Queryable;
        for width in [520.0, 800.0] {
            let mut h = customization_harness(width, super::Page::QuickAccess);
            h.run_steps(8);
            h.get_by_label("Add >>").scroll_to_me();
            h.run_steps(8);
            h.get_by_label("Add >>").click();
            h.run_steps(4);
            assert_eq!(h.state().session.ribbon.qat(), ["file.save", "edit.undo", "edit.redo", "format.bold"]);
            h.get_by_label("Up").scroll_to_me();
            h.run_steps(8);
            h.get_by_label("Up").click();
            h.run_steps(4);
            assert_eq!(h.state().session.ribbon.qat(), ["file.save", "edit.undo", "format.bold", "edit.redo"]);
            h.get_by_label("Down").click();
            h.run_steps(4);
            assert_eq!(h.state().session.ribbon.qat(), ["file.save", "edit.undo", "edit.redo", "format.bold"]);
            h.get_by_label("<< Remove").scroll_to_me();
            h.run_steps(8);
            h.get_by_label("<< Remove").click();
            h.run_steps(4);
            assert_eq!(h.state().session.ribbon.qat(), ["file.save", "edit.undo", "edit.redo"]);
            h.get_by_label("Show Quick Access Toolbar").scroll_to_me();
            h.run_steps(8);
            h.get_by_label("Show Quick Access Toolbar").click();
            h.run_steps(4);
            assert!(!h.state().ui.quick_access_toolbar);
            h.query_all_by_label("Close").max_by(|a, b| a.rect().bottom().total_cmp(&b.rect().bottom())).unwrap().scroll_to_me();
            h.run_steps(8);
            let close = h.query_all_by_label("Close").max_by(|a, b| a.rect().bottom().total_cmp(&b.rect().bottom())).unwrap();
            assert!(close.rect().top() >= 0.0 && close.rect().bottom() <= 600.0, "footer Close is reachable at {width}");
            close.click();
            h.run_steps(4);
            assert!(h.state().dialog.is_none());
        }
    }

    #[test]
    fn compact_ribbon_customization_adds_a_command_to_a_new_group() {
        use egui_kittest::kittest::Queryable;
        let mut h = customization_harness(800.0, super::Page::Ribbon);
        h.run_steps(8);
        h.get_by_label("New Tab").click();
        h.run_steps(4);
        h.get_by_label("Add >>").click();
        h.run_steps(4);
        assert!(
            h.state()
                .session
                .ribbon
                .tabs()
                .iter()
                .any(|tab| { tab.custom && tab.groups.iter().any(|group| group.commands.iter().any(|command| command == "format.bold")) })
        );
    }

    /// #1945: the Quick Access Toolbar page's Show Quick Access Toolbar check box is the toolbar's
    /// visibility; the dialog stays open while it is switched.
    #[test]
    fn quick_access_page_has_a_show_toolbar_check_box() {
        use egui_kittest::kittest::{NodeT, Queryable};
        let mut a = app();
        super::open(&mut a, super::Page::QuickAccess);
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        let ticked = |h: &egui_kittest::Harness<'static, WordApp>| h.get_by_label("Show Quick Access Toolbar").accesskit_node().toggled();
        assert_eq!(ticked(&h), Some(egui::accesskit::Toggled::True), "ticked while the toolbar shows");
        h.get_by_label("Show Quick Access Toolbar").click();
        h.run_steps(2);
        assert!(!h.state().ui.quick_access_toolbar, "unticking hides the toolbar");
        assert!(matches!(h.state().dialog, Some(Dialog::CustomizeRibbon { .. })), "the dialog stays open");
        assert_eq!(ticked(&h), Some(egui::accesskit::Toggled::False));
        h.get_by_label("Show Quick Access Toolbar").click();
        h.run_steps(2);
        assert!(h.state().ui.quick_access_toolbar, "ticking shows it again");
    }
}
