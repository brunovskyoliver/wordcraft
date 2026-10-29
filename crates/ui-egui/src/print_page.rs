//! File › Print (#641): the print settings and a preview of the pages.
//!
//! Page range and parity shape the PDF. Native printer settings and copy collation go to
//! the shared spooler; browser printing and saved print PDFs retain copies in the PDF.

use std::sync::Arc;

use egui::{Rect, Sense, Stroke, TextureHandle, Ui, pos2, vec2};
use serde_json::{Value, json};
use wordcraft_layout::DocLayout;

use crate::theme::{Tokens, medium, regular};
use crate::{WordApp, icons};

/// The Print page's settings. Not saved with the document or the UI state.
#[derive(Clone, Debug, PartialEq)]
pub struct PrintSettings {
    /// `all`, `current`, `selection` or `custom`.
    pub range: String,
    /// The custom range text, like `1-3, 5`.
    pub pages: String,
    /// `all`, `odd` or `even`.
    pub parity: String,
    pub copies: u32,
    pub collate: bool,
    /// The page shown in the preview (0-based).
    pub preview: usize,
    /// `auto`, `portrait` or `landscape`.
    pub orientation: String,
    /// The preview's zoom (1.0 is the page's real size), or `None` to fit the whole page.
    pub zoom: Option<f32>,
    /// Scale to paper size (#2108): `none` prints the document's page sizes, a Layout › Size
    /// name scales every page onto that paper. The document itself is unchanged.
    pub paper: String,
}

impl Default for PrintSettings {
    fn default() -> Self {
        PrintSettings {
            range: "all".into(),
            pages: String::new(),
            parity: "all".into(),
            copies: 1,
            collate: true,
            preview: 0,
            zoom: None,
            orientation: "auto".into(),
            paper: NO_SCALING.into(),
        }
    }
}

impl PrintSettings {
    /// The params of `ui.print` and `file.printPdf` for these settings.
    pub fn params(&self) -> Value {
        let mut v = json!({
            "range": self.range,
            "parity": self.parity,
            "copies": self.copies,
            "collate": self.collate,
            "orientation": self.orientation,
        });
        if self.range == "custom" {
            v["pages"] = json!(self.pages);
        }
        if self.paper != NO_SCALING {
            v["paper"] = json!(self.paper);
        }
        v
    }

    /// The paper pages are scaled to, as (width, height) in points, or `None` for their own size.
    pub fn paper_size(&self) -> Option<(f32, f32)> {
        wordcraft_geom::paper_by_name(&self.paper).map(|(_, w, h)| (w, h))
    }
}

const RANGES: [(&str, &str); 4] = [("all", "All pages"), ("current", "Current page"), ("selection", "Selected pages"), ("custom", "Custom range")];
const PARITIES: [(&str, &str); 3] = [("all", "All pages in the range"), ("odd", "Odd pages only"), ("even", "Even pages only")];
/// The Scale to paper size choice that keeps each page's own size.
const NO_SCALING: &str = "none";
const ORIENTATIONS: [(&str, &str); 3] = [("auto", "Auto orientation"), ("portrait", "Portrait"), ("landscape", "Landscape")];

/// A choice list over `(id, label)` pairs; true when the choice changed.
fn choice(ui: &mut Ui, salt: &str, current: &mut String, items: &[(&str, &str)], label: &egui::Response) -> bool {
    let shown = items.iter().find(|(id, _)| *id == current.as_str()).map_or("", |(_, l)| *l);
    let mut changed = false;
    let combo = craft_ui::ComboBox::from_id_salt(salt).width(ui.available_width().min(260.0)).selected_text(tl!(shown)).show_ui(ui, |ui| {
        for (id, item) in items {
            if ui.selectable_label(current == id, tl!(item)).clicked() && current != id {
                *current = (*id).to_string();
                changed = true;
            }
        }
    });
    combo.response.labelled_by(label.id);
    changed
}

/// A choice list named without a visible label beside it (its heading names a group of choices).
fn choice_named(ui: &mut Ui, salt: &str, current: &mut String, items: &[(&str, &str)], name: &str) -> bool {
    let shown = items.iter().find(|(id, _)| *id == current.as_str()).map_or("", |(_, l)| *l);
    let mut changed = false;
    let combo = craft_ui::ComboBox::from_id_salt(salt).width(ui.available_width().min(260.0)).selected_text(tl!(shown)).show_ui(ui, |ui| {
        for (id, item) in items {
            if ui.selectable_label(current == id, tl!(item)).clicked() && current != id {
                *current = (*id).to_string();
                changed = true;
            }
        }
    });
    combo.response.accessible_name(name);
    changed
}

/// The output layout, kept while the document is unchanged (laying out every frame would be slow).
fn layout_of(app: &WordApp, ctx: &egui::Context) -> Arc<DocLayout> {
    let id = egui::Id::unique("print_page_layout");
    let key = (app.session.document_id(), app.session.rev());
    if let Some((k, l)) = ctx.data(|d| d.get_temp::<((u64, u64), Arc<DocLayout>)>(id))
        && k == key
    {
        return l;
    }
    let l = app.session.export_layout();
    ctx.data_mut(|d| d.insert_temp(id, (key, l.clone())));
    l
}

/// The texture of one page, re-rendered only when the document, page or size changes.
fn page_texture(app: &WordApp, ctx: &egui::Context, lay: &DocLayout, page: usize, width: f32) -> Option<TextureHandle> {
    let id = egui::Id::unique("print_page_texture");
    let key = (app.session.document_id(), app.session.rev(), page, width as u32, (ctx.pixels_per_point() * 100.0) as u32);
    if let Some((k, t)) = ctx.data(|d| d.get_temp::<((u64, u64, usize, u32, u32), TextureHandle)>(id))
        && k == key
    {
        return Some(t);
    }
    let pg = lay.pages.get(page)?;
    // Zoomed in, the texture stays within what the GPU takes and is stretched beyond that.
    let max_side = ctx.input(|i| i.max_texture_side) as f32 / pg.w.max(pg.h).max(1.0);
    let scale = (width / pg.w.max(1.0) * ctx.pixels_per_point()).clamp(0.05, 4.0).min(max_side);
    let img = wordcraft_render::render_page(&app.session.doc, pg, scale, &Default::default());
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &img.to_straight());
    let tex = ctx.load_texture("print_page_preview", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, (key, tex.clone())));
    Some(tex)
}

/// The Print page uses columns when both fit, otherwise settings and preview stack.
pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let lay = layout_of(app, &ctx);
    let total = lay.pages.len();
    let current = app.session.current_pages(&lay);
    app.ui.print.preview = app.ui.print.preview.min(total.saturating_sub(1));
    let label = page_label(app.ui.print.preview, total);
    let (navigation, zoom) = footer_widths(ui, &label, "500%");
    let available = ui.available_size();
    let gap = ui.spacing().item_spacing.x;
    let minimum = print_content_minimum(available.x, navigation, zoom, gap);
    let mut draw = |ui: &mut Ui| {
        // A scroll bar can narrow the child further, so measure the minimum again there.
        let minimum = print_content_minimum(ui.available_width(), navigation, zoom, gap);
        let size = vec2(ui.available_width().max(minimum.x), available.y.max(minimum.y));
        let (area, _) = ui.allocate_exact_size(size, Sense::hover());
        show_regions(app, ui, &lay, total, current, area, navigation + zoom + gap);
    };
    if available.x < minimum.x || available.y < minimum.y {
        // At high interface scaling even two footer rows may exceed the viewport. Keep
        // their real sizes and let the whole Print content scroll to them, without overlap.
        egui::ScrollArea::both().id_salt("print_content").auto_shrink(false).min_scrolled_width(0.0).min_scrolled_height(0.0).show(ui, draw);
    } else {
        draw(ui);
    }
}

fn print_content_minimum(width: f32, navigation: f32, zoom: f32, gap: f32) -> egui::Vec2 {
    let width = width.max(navigation.max(zoom));
    let footer = if width < navigation + zoom + gap { 2.0 * PREVIEW_BAR + gap } else { PREVIEW_BAR };
    // Leave a small page viewport as well as the footer. The stacked preview receives
    // 60% of the height after the gap; settings retain their own scroll area above it.
    let preview = footer + gap + 2.0 * PREVIEW_MARGIN + PREVIEW_BUTTON;
    let height =
        if width < SETTINGS_WIDTH + COLUMN_GAP + navigation + zoom + gap { STACK_GAP + preview / (1.0 - STACK_SETTINGS_SHARE) } else { preview };
    vec2(navigation.max(zoom), height)
}

fn show_regions(app: &mut WordApp, ui: &mut Ui, lay: &DocLayout, total: usize, current: (usize, usize), area: Rect, preview_min: f32) {
    let (settings_area, preview_area) = print_regions(area, preview_min);
    ui.scope_builder(egui::UiBuilder::new().id_salt("print_settings_region").max_rect(settings_area), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("print_settings")
            .auto_shrink(false)
            .min_scrolled_height(0.0)
            .max_height(settings_area.height())
            .show(ui, |ui| settings(app, ui, total, current));
    });
    ui.scope_builder(egui::UiBuilder::new().id_salt("print_preview_region").max_rect(preview_area), |ui| preview(app, ui, lay, total));
}

const SETTINGS_WIDTH: f32 = 380.0;
const COLUMN_GAP: f32 = 24.0;
const STACK_GAP: f32 = 12.0;
const STACK_SETTINGS_SHARE: f32 = 0.4;

fn print_regions(area: Rect, preview_min: f32) -> (Rect, Rect) {
    if area.width() >= SETTINGS_WIDTH + COLUMN_GAP + preview_min {
        (
            Rect::from_min_max(area.min, pos2(area.left() + SETTINGS_WIDTH, area.bottom())),
            Rect::from_min_max(pos2(area.left() + SETTINGS_WIDTH + COLUMN_GAP, area.top()), area.max),
        )
    } else {
        let gap = STACK_GAP.min(area.height().max(0.0));
        let settings_height = ((area.height() - gap).max(0.0) * STACK_SETTINGS_SHARE).min(260.0);
        (
            Rect::from_min_max(area.min, pos2(area.right(), area.top() + settings_height)),
            Rect::from_min_max(pos2(area.left(), area.top() + settings_height + gap), area.max),
        )
    }
}

fn settings(app: &mut WordApp, ui: &mut Ui, total: usize, current: (usize, usize)) {
    let t = Tokens::get(ui.ctx());
    ui.spacing_mut().item_spacing.y = 8.0;
    let native = app.services.print_native.is_some();
    let hook = native || app.services.print.is_some();
    // #2958: as in Word, the page's action leads the settings, in the accent the app
    // gives a dialog's primary button: the first thing seen, and plainly a button
    // that prints now rather than one more setting in the list below.
    if hook
        && ui
            .add_enabled(
                !native || app.printing.ready(),
                crate::widgets::primary_button(ui, tl!("Print")).min_size(vec2(ui.available_width().min(320.0), 34.0)).wrap(),
            )
            .clicked()
    {
        let params = app.ui.print.params();
        let _ = app.run("ui.print", params);
    }
    if native {
        if let Some(discover) = &app.services.printers {
            crate::printing::show(&mut app.printing, ui, discover, |label| crate::i18n::t(label).to_owned());
        }
        app.ui.print.copies = app.printing.settings.copies;
    } else if hook {
        ui.label(tl!("Send the document straight to the system print dialog, or save a copy in another format below."));
        ui.label(egui::RichText::new(tl!("The printer, two-sided printing and pages per sheet are chosen in that dialog.")).color(t.text_dim));
    } else {
        ui.label(tl!("Save a copy in another format. Printing goes through a PDF you can print from any viewer."));
    }
    if ui
        .add(
            egui::Button::new(egui::RichText::new(tl!("Save these pages as PDF…")).font(medium(13.5)))
                .min_size(vec2(ui.available_width().min(320.0), 34.0))
                .wrap(),
        )
        .clicked()
    {
        app.print_pdf_dialog();
    }
    ui.add_space(6.0);

    if native {
        if app.ui.print.copies > 1 {
            ui.add_enabled(!app.printing.busy(), egui::Checkbox::new(&mut app.ui.print.collate, tl!("Collate copies")));
        }
    } else {
        let copies_label = ui.label(egui::RichText::new(tl!("Copies")).font(medium(13.0)));
        ui.horizontal(|ui| {
            let mut n = app.ui.print.copies;
            ui.add(egui::DragValue::new(&mut n).range(1..=wordcraft_engine::print::MAX_COPIES).speed(0.2)).labelled_by(copies_label.id);
            app.ui.print.copies = n.clamp(1, wordcraft_engine::print::MAX_COPIES);
            if app.ui.print.copies > 1 {
                ui.checkbox(&mut app.ui.print.collate, tl!("Collate copies"));
            }
        });
    }

    let pages_label = ui.label(egui::RichText::new(tl!("Pages")).font(medium(13.0)));
    if choice(ui, "print_range", &mut app.ui.print.range, &RANGES, &pages_label) {
        // Show the first page that will print.
        app.ui.print.preview = if app.ui.print.range == "all" { 0 } else { current.0.min(total.saturating_sub(1)) };
    }
    if app.ui.print.range == "custom" {
        ui.add(
            egui::TextEdit::singleline(&mut app.ui.print.pages)
                .hint_text(tl!("Pages like 1-3, 5"))
                .desired_width(ui.available_width().min(260.0))
                .char_limit(wordcraft_engine::print::MAX_SPEC_CHARS),
        )
        .labelled_by(pages_label.id);
    }
    choice_named(ui, "print_parity", &mut app.ui.print.parity, &PARITIES, tl!("Pages in the range"));
    let orientation_label = ui.label(egui::RichText::new(tl!("Orientation")).font(medium(13.0)));
    choice(ui, "print_orientation", &mut app.ui.print.orientation, &ORIENTATIONS, &orientation_label);
    // Word: Pages per sheet › Scale to paper size. Only the printout changes.
    let paper_label = ui.label(egui::RichText::new(tl!("Scale to paper size")).font(medium(13.0)));
    let papers: Vec<(&str, &str)> =
        std::iter::once((NO_SCALING, "No Scaling")).chain(wordcraft_geom::PAPER_SIZES.iter().map(|(n, _, _)| (*n, *n))).collect();
    choice(ui, "print_paper", &mut app.ui.print.paper, &papers, &paper_label);

    match wordcraft_engine::print::plan(&app.ui.print.params(), total, current) {
        Ok(plan) => {
            let msg =
                crate::i18n::fmt(tl!("{count} of {total} pages will print"), &[("count", &plan.chosen.to_string()), ("total", &total.to_string())]);
            ui.label(egui::RichText::new(msg).color(t.text_dim));
        }
        Err(e) => {
            ui.label(egui::RichText::new(e.to_string()).color(t.red));
        }
    }
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);
    if hook {
        ui.label(tl!("Or save a copy in another format:"));
    }
    export_formats(app, ui);
}

/// Room around the page inside the preview pane.
const PREVIEW_MARGIN: f32 = 10.0;
/// The preview's bottom bar: the page indicator on the left, the zoom on the right.
const PREVIEW_BAR: f32 = 34.0;

/// The next preview zoom one step (10%) in or out, from 10% to 500% as in the status bar.
fn zoom_step(z: f32, up: bool) -> f32 {
    let z = (z * 10.0).round() / 10.0;
    (if up { z + 0.1 } else { z - 0.1 }).clamp(0.1, 5.0)
}

/// The preview (#1156): the page fills the pane, a bar below pages through the document and
/// zooms; zoomed in, the page scrolls both ways.
fn preview(app: &mut WordApp, ui: &mut Ui, lay: &DocLayout, total: usize) {
    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let page = app.ui.print.preview;
    let (page_w, page_h) = lay.pages.get(page).map_or((612.0, 792.0), |p| (p.w.max(1.0), p.h.max(1.0)));
    // Scale to paper size: the sheet is the chosen paper with the page scaled and centred on it.
    let sheet = app.ui.print.paper_size().map(|(w, h)| wordcraft_geom::fit_to_paper(page_w, page_h, w, h));
    let (pw, ph) = sheet.map_or((page_w, page_h), |f| (f.paper_w, f.paper_h));
    let label = page_label(page, total);
    let (navigation_width, zoom_width) = footer_widths(ui, &label, "500%");
    let (area, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let regions = preview_regions(area, navigation_width, zoom_width, ui.spacing().item_spacing.x);
    // Screen points per page point, confined to the allocated preview rather than a forced minimum.
    let fit = ((regions.page.width() - 2.0 * PREVIEW_MARGIN) / pw).min((regions.page.height() - 2.0 * PREVIEW_MARGIN) / ph).max(0.01);
    let scale = app.ui.print.zoom.map_or(fit, |z| z * crate::canvas::PX_PER_PT);
    let size = vec2(pw * scale, ph * scale);
    ui.scope_builder(egui::UiBuilder::new().id_salt("page").max_rect(regions.page), |ui| {
        egui::ScrollArea::both()
            .id_salt("print_preview")
            .auto_shrink(false)
            .min_scrolled_width(0.0)
            .min_scrolled_height(0.0)
            .max_width(regions.page.width())
            .max_height(regions.page.height())
            .show(ui, |ui| {
                let room =
                    vec2((size.x + 2.0 * PREVIEW_MARGIN).max(ui.available_width()), (size.y + 2.0 * PREVIEW_MARGIN).max(ui.available_height()));
                let (area, _) = ui.allocate_exact_size(room, Sense::hover());
                let r = Rect::from_center_size(area.center(), size);
                let resp = ui.interact(r, ui.scope_id().with("print_preview_page"), Sense::hover());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, &label));
                ui.painter().rect_filled(r.translate(vec2(2.0, 3.0)), 0.0, t.page_shadow);
                ui.painter().rect(r, 0.0, egui::Color32::WHITE, Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
                let on_sheet = sheet.map_or(r, |f| Rect::from_min_size(r.min + vec2(f.dx, f.dy) * scale, vec2(page_w, page_h) * f.k * scale));
                if let Some(tex) = page_texture(app, &ctx, lay, page, on_sheet.width()) {
                    ui.painter().image(tex.id(), on_sheet, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
                }
            });
    });
    ui.scope_builder(
        egui::UiBuilder::new().id_salt("navigation").max_rect(regions.navigation).layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            if preview_button(ui, "chevronLeft", tl!("Previous Page"), page > 0) {
                app.ui.print.preview = page - 1;
            }
            ui.label(egui::RichText::new(&label).font(regular(13.0)));
            if preview_button(ui, "chevronRight", tl!("Next Page"), page + 1 < total) {
                app.ui.print.preview = page + 1;
            }
        },
    );
    ui.scope_builder(egui::UiBuilder::new().id_salt("zoom").max_rect(regions.zoom).layout(egui::Layout::right_to_left(egui::Align::Center)), |ui| {
        let z = scale / crate::canvas::PX_PER_PT;
        if preview_button(ui, "onePage", tl!("One Page"), true) {
            app.ui.print.zoom = None;
        }
        ui.label(egui::RichText::new(format!("{}%", (z * 100.0).round())).font(regular(13.0)));
        if preview_button(ui, "plus", tl!("Zoom In"), z < 5.0) {
            app.ui.print.zoom = Some(zoom_step(z, true));
        }
        let room = ui.available_width() - PREVIEW_BUTTON - ui.spacing().item_spacing.x;
        if room >= 40.0 {
            let mut v = z;
            ui.spacing_mut().slider_width = room.min(110.0);
            let response = ui.add(egui::Slider::new(&mut v, 0.1..=5.0).logarithmic(true).show_value(false)).on_hover_text(tl!("Zoom"));
            response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Slider, ui.is_enabled(), tl!("Zoom")));
            if response.changed() {
                app.ui.print.zoom = Some(v.clamp(0.1, 5.0));
            }
        }
        if preview_button(ui, "minus", tl!("Zoom Out"), z > 0.1) {
            app.ui.print.zoom = Some(zoom_step(z, false));
        }
    });
}

const PREVIEW_BUTTON: f32 = 26.0;

fn page_label(page: usize, total: usize) -> String {
    crate::i18n::fmt(tl!("Page {page} of {pages}"), &[("page", &page.saturating_add(1).to_string()), ("pages", &total.to_string())])
}

/// Minimum widths without the optional slider, measured with the same fonts as the controls.
fn footer_widths(ui: &Ui, page: &str, percent: &str) -> (f32, f32) {
    let text = |value: &str| ui.painter().layout_no_wrap(value.to_owned(), regular(13.0), ui.visuals().text_color()).size().x;
    let gap = ui.spacing().item_spacing.x;
    (2.0 * PREVIEW_BUTTON + text(page) + 2.0 * gap, 3.0 * PREVIEW_BUTTON + text(percent) + 3.0 * gap)
}

struct PreviewRegions {
    page: Rect,
    navigation: Rect,
    zoom: Rect,
}

fn preview_regions(area: Rect, navigation_width: f32, zoom_width: f32, gap: f32) -> PreviewRegions {
    let stacked = area.width() < navigation_width + zoom_width + gap;
    let footer_height = if stacked { 2.0 * PREVIEW_BAR + gap } else { PREVIEW_BAR };
    let footer_top = area.bottom() - footer_height;
    let page = Rect::from_min_max(area.min, pos2(area.right(), (footer_top - gap).max(area.top())));
    let (navigation, zoom) = if stacked {
        (
            Rect::from_min_size(pos2(area.left(), footer_top), vec2(area.width(), PREVIEW_BAR)),
            Rect::from_min_size(pos2(area.left(), area.bottom() - PREVIEW_BAR), vec2(area.width(), PREVIEW_BAR)),
        )
    } else {
        (
            Rect::from_min_size(pos2(area.left(), footer_top), vec2(navigation_width, PREVIEW_BAR)),
            Rect::from_min_max(pos2(area.left() + navigation_width + gap, footer_top), area.max),
        )
    };
    PreviewRegions { page, navigation, zoom }
}

fn preview_button(ui: &mut Ui, icon: &str, tip: &str, enabled: bool) -> bool {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(PREVIEW_BUTTON, PREVIEW_BUTTON), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, tip));
    let col = if enabled { t.icon } else { t.text_disabled };
    if enabled && resp.hovered() {
        ui.painter().rect_filled(r, 3.0, t.hover);
    }
    icons::paint(ui.painter(), r.shrink(4.0), icon, col, col);
    enabled && resp.on_hover_text(tip).clicked()
}

/// The export "format" of a Word 2003 XML Document: an `.xml` file saved with `format: "wordml2003"`.
pub(crate) const EXPORT_WORD_2003: &str = "wordml2003";

/// The "save a copy" formats (File › Export, and below the settings on the Print page).
pub fn export_formats(app: &mut WordApp, ui: &mut Ui) {
    for (label, ext) in [
        ("PDF document (*.pdf)", "pdf"),
        ("Word document (*.docx)", "docx"),
        ("Word XML Document (*.xml)", "xml"),
        ("Word 2003 XML Document (*.xml)", EXPORT_WORD_2003),
        ("OpenDocument Text (*.odt)", "odt"),
        ("Rich Text Format (*.rtf)", "rtf"),
        ("Web page (*.html)", "html"),
        ("Markdown (*.md)", "md"),
        ("LaTeX (*.tex)", "tex"),
        ("Plain text (*.txt)", "txt"),
        ("Page image (*.png)", "png"),
    ] {
        if ui
            .add(egui::Button::new(egui::RichText::new(tl!(label)).font(medium(13.5))).min_size(vec2(ui.available_width().min(320.0), 34.0)).wrap())
            .clicked()
        {
            app.export_dialog(ext);
        }
        ui.add_space(4.0);
    }
}

#[cfg(test)]
mod responsive_tests {
    use super::*;

    #[test]
    fn print_caches_follow_document_switches_at_the_same_revision() {
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("First document")), crate::Services::default());
        let first = app.session.document_id();
        let revision = app.session.rev();
        let second = app.session.add_document(wordcraft_doc::Document::from_text("Second document"));
        app.sync_document();
        assert_eq!(app.session.rev(), revision, "distinct fresh documents share revision numbers");
        let ctx = egui::Context::default();
        app.execute("view.switchWindows", json!({"id": first})).unwrap();
        let first_layout = layout_of(&app, &ctx);
        let first_texture = page_texture(&app, &ctx, &first_layout, 0, 160.0).unwrap();
        app.execute("view.switchWindows", json!({"id": second})).unwrap();
        assert_eq!(app.session.rev(), revision);
        let second_layout = layout_of(&app, &ctx);
        assert!(!Arc::ptr_eq(&first_layout, &second_layout), "a document switch must not return the preceding document's layout");
        let second_texture = page_texture(&app, &ctx, &second_layout, 0, 160.0).unwrap();
        assert_ne!(first_texture.id(), second_texture.id(), "same-size pages must not reuse another document's image");
        assert!(Arc::ptr_eq(&second_layout, &layout_of(&app, &ctx)), "unchanged documents still reuse their layout");
        assert_eq!(second_texture.id(), page_texture(&app, &ctx, &second_layout, 0, 160.0).unwrap().id());
        app.execute("view.switchWindows", json!({"id": first})).unwrap();
        let back = layout_of(&app, &ctx);
        assert!(!Arc::ptr_eq(&second_layout, &back));
        assert_ne!(second_texture.id(), page_texture(&app, &ctx, &back, 0, 160.0).unwrap().id());
    }

    #[test]
    fn short_print_content_scrolls_instead_of_overlapping_footer_and_settings() {
        for available in [vec2(245.0, 80.0), vec2(120.0, 100.0)] {
            let minimum = print_content_minimum(available.x, 150.0, 140.0, 8.0);
            assert!(minimum.y > available.y);
            let content = Rect::from_min_size(egui::Pos2::ZERO, available.max(minimum));
            let (settings, preview) = print_regions(content, 298.0);
            let footer = preview_regions(preview, 150.0, 140.0, 8.0);
            for region in [footer.page, footer.navigation, footer.zoom] {
                assert!(preview.contains_rect(region));
                assert!(!settings.intersects(region), "scrollable content must keep controls out of the settings region");
            }
            assert!(footer.page.height() >= 2.0 * PREVIEW_MARGIN + PREVIEW_BUTTON - 0.1);
        }
    }

    #[test]
    fn print_regions_keep_both_scroll_areas_and_footer_inside_the_content() {
        for (width, height) in [(344.0, 200.0), (504.0, 200.0), (504.0, 430.0), (1144.0, 320.0), (1144.0, 730.0)] {
            let area = Rect::from_min_size(pos2(248.0, 130.0), vec2(width, height));
            let (settings, preview) = print_regions(area, 320.0);
            assert!(area.contains_rect(settings) && area.contains_rect(preview));
            assert!(!settings.intersects(preview));
            assert!(settings.width() >= 320.0 && settings.height() > 0.0);
            let regions = preview_regions(preview, 170.0, 140.0, 8.0);
            for rect in [regions.page, regions.navigation, regions.zoom] {
                assert!(preview.contains_rect(rect), "{width}×{height}: {rect:?} outside {preview:?}");
            }
            assert!(!regions.page.intersects(regions.navigation));
            assert!(!regions.page.intersects(regions.zoom));
            assert!(!regions.navigation.intersects(regions.zoom));
            if width < 700.0 {
                assert!(settings.bottom() < preview.top(), "compact settings and preview stack");
            } else {
                assert!(settings.right() < preview.left(), "desktop keeps columns");
            }
        }
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(280.0, 240.0));
        let regions = preview_regions(area, 180.0, 150.0, 8.0);
        assert!(regions.navigation.bottom() < regions.zoom.top(), "long footer labels use separate rows");
        assert!(area.contains_rect(regions.zoom));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn harness(width: f32, height: f32) -> egui_kittest::Harness<'static, WordApp> {
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        app.run("text.insert", json!({"text": "First page"})).unwrap();
        app.run("insert.pageBreak", json!({})).unwrap();
        app.run("text.insert", json!({"text": "Second page"})).unwrap();
        app.run("file.print", json!({})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(width, height)).build_ui_state(
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

    #[cfg(not(target_arch = "wasm32"))]
    fn wheel(h: &mut egui_kittest::Harness<'_, WordApp>, at: egui::Pos2, delta: egui::Vec2) {
        h.input_mut().events.extend([
            egui::Event::PointerMoved(at),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta,
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::NONE,
                source: egui::MouseWheelSource::Unknown,
            },
        ]);
        h.run_steps(20);
    }

    /// AUD-016: actual preview controls must remain separately clickable inside compact windows.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn responsive_print_controls_navigate_zoom_and_fit_without_overlap() {
        use egui_kittest::kittest::{By, Queryable};

        for (width, height) in [(640.0, 360.0), (640.0, 480.0), (800.0, 360.0), (800.0, 600.0), (1440.0, 480.0), (1440.0, 900.0)] {
            let mut h = harness(width, height);
            let screen = Rect::from_min_size(egui::Pos2::ZERO, vec2(width, height));
            let original = h.state().session.doc.clone();
            let dirty = h.state().session.dirty;
            let controls: Vec<_> = ["Previous Page", "Next Page", "Zoom Out", "Zoom In", "One Page"]
                .into_iter()
                .map(|name| (name, h.get_by_label(name).rect()))
                .collect();
            for (i, (name, rect)) in controls.iter().enumerate() {
                assert!(screen.contains_rect(*rect), "{width}×{height}: {name} outside viewport: {rect:?}");
                for (other, other_rect) in controls.iter().skip(i + 1) {
                    assert!(!rect.intersects(*other_rect), "{width}×{height}: {name} overlaps {other}");
                }
            }
            let label = h.get(By::new().role(egui::accesskit::Role::Label).label("Page 1 of 2")).rect();
            for (name, rect) in &controls {
                assert!(!label.intersects(*rect), "{width}×{height}: page label overlaps {name}");
            }
            let image = |h: &egui_kittest::Harness<'_, WordApp>| h.get(By::new().role(egui::accesskit::Role::Image)).rect();
            let fit = image(&h);
            assert!(screen.contains_rect(fit), "One Page fits inside the viewport");
            h.get_by_label("Next Page").click();
            h.run_steps(3);
            assert_eq!(h.state().ui.print.preview, 1);
            h.get_by_label("Previous Page").click();
            h.run_steps(3);
            assert_eq!(h.state().ui.print.preview, 0);
            for _ in 0..5 {
                h.get_by_label("Zoom In").click();
                h.run_steps(2);
            }
            let zoomed = image(&h);
            assert!(zoomed.height() > fit.height(), "Zoom In changes the preview");
            let zoom = h.state().ui.print.zoom.unwrap();
            let percent = format!("{}%", (zoom * 100.0).round());
            let percent_rect = h.get(By::new().role(egui::accesskit::Role::Label).label(&percent)).rect();
            assert!(screen.contains_rect(percent_rect));
            for name in ["Previous Page", "Next Page", "Zoom Out", "Zoom In", "One Page"] {
                assert!(!percent_rect.intersects(h.get_by_label(name).rect()), "{width}×{height}: percent overlaps {name}");
            }
            h.get_by_label("Zoom Out").click();
            h.run_steps(2);
            assert!(h.state().ui.print.zoom.unwrap() < zoom, "Zoom Out is independently clickable");
            h.get_by_label("One Page").click();
            h.run_steps(4);
            assert!(h.state().ui.print.zoom.is_none());
            assert!((image(&h).height() - fit.height()).abs() < 1.0, "One Page restores the fit");
            assert!(h.state().session.doc.same_content(&original));
            assert_eq!(h.state().session.dirty, dirty);
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn responsive_print_settings_and_preview_scroll_independently() {
        use egui_kittest::kittest::{By, Queryable};

        for (width, height) in [(640.0, 480.0), (800.0, 600.0), (1440.0, 900.0)] {
            let mut h = harness(width, height);
            let image = |h: &egui_kittest::Harness<'_, WordApp>| h.get(By::new().role(egui::accesskit::Role::Image)).rect();
            let fit = image(&h);
            let settings_target = h.get_by_label("Save these pages as PDF…").rect().center();
            wheel(&mut h, settings_target, vec2(0.0, -10_000.0));
            let last_export = h.get_by_label("Page image (*.png)").rect();
            let screen = Rect::from_min_size(egui::Pos2::ZERO, vec2(width, height));
            assert!(screen.contains_rect(last_export), "the last settings action is reachable");
            if width < 1000.0 {
                assert!(last_export.bottom() < fit.top(), "settings scroll stays above the preview");
            } else {
                assert!(last_export.right() < fit.left(), "settings scroll stays left of the preview");
            }
            assert_eq!(image(&h), fit, "scrolling settings leaves preview geometry alone");
            for _ in 0..5 {
                h.get_by_label("Zoom In").click();
                h.run_steps(2);
            }
            let zoomed = image(&h);
            wheel(&mut h, fit.center(), vec2(-120.0, -120.0));
            let moved = image(&h);
            assert!(moved.top() < zoomed.top() - 20.0, "the preview scrolls vertically");
            assert!(moved.left() < zoomed.left() - 20.0, "the preview scrolls horizontally");
            assert_eq!(h.get_by_label("Page image (*.png)").rect(), last_export, "preview scrolling leaves settings alone");
            h.get_by_label("One Page").click();
            h.run_steps(4);
            assert!((image(&h).height() - fit.height()).abs() < 1.0);
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn responsive_print_high_scale_short_window_keeps_controls_reachable() {
        use egui_kittest::kittest::Queryable;

        let mut h = harness(800.0, 360.0);
        h.state_mut().run("ui.interfaceSize", json!({"percent": 150})).unwrap();
        h.run_steps(6);
        let original = h.state().session.doc.clone();
        let screen = h.ctx.input(|i| i.content_rect());
        assert!(screen.height() < 300.0, "the test exercises the short logical viewport");
        // This action sits at the bottom of the independently scrolling settings list.
        h.get_by_label("Page image (*.png)").scroll_to_me();
        h.run_steps(12);
        assert!(screen.contains_rect(h.get_by_label("Page image (*.png)").rect()));
        h.get_by_label("Next Page").scroll_to_me();
        h.run_steps(12);
        assert!(screen.contains_rect(h.get_by_label("Next Page").rect()));
        h.get_by_label("Next Page").click();
        h.run_steps(3);
        assert_eq!(h.state().ui.print.preview, 1);
        h.get_by_label("Zoom In").scroll_to_me();
        h.run_steps(12);
        assert!(screen.contains_rect(h.get_by_label("Zoom In").rect()));
        h.get_by_label("Zoom In").click();
        h.run_steps(3);
        assert!(h.state().ui.print.zoom.is_some());
        h.get_by_label("One Page").scroll_to_me();
        h.run_steps(12);
        assert!(screen.contains_rect(h.get_by_label("One Page").rect()));
        h.get_by_label("One Page").click();
        h.run_steps(3);
        assert!(h.state().ui.print.zoom.is_none());
        assert!(h.state().session.doc.same_content(&original));
    }
}
