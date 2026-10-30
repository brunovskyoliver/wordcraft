//! Headless UI screenshots: renders the whole WordCraft window offscreen (wgpu), so it works
//! with a locked screen or a hidden window.
//!
//! `cargo run -p wordcraft-ui-egui --example ui_shot -- script.jsonl`
//!
//! The script is JSON lines: control-channel requests (`{"method": …, "params": …}`, see
//! `docs/control-protocol.md`), `{"shot": "/abs/out.png"}` to save the window as PNG, or
//! `{"steps": n}` to run extra frames, or `{"wait": ms}` to keep running frames for a while (for
//! background work such as a Zotero session). The window is 1440×900 pt at 2× and opens the sample
//! document unless the first line is `{"empty": true}`; `{"empty": true, "startScreen": true}`
//! starts the way the desktop app does with nothing to open (File › Home, #766).
//! The first line may set `width`, `height` (points) and `pixelsPerPoint` (default 2).
//! Requests fail the run on an error or missing reply. `expect` checks a recursive subset of
//! the reply; use `"expect": {"ok": false}` to test a deliberate command refusal.
//! `{"clickLabel": "OK"}` clicks the first enabled control with that accessible name, so a script
//! doesn't depend on coordinates. `{"audit": "name"}` prints the controls that leave the window,
//! overlap or have no accessible name; add `"strict": true` to fail the run on any of them.

use wordcraft_engine::Session;
use wordcraft_ui_egui::{ControlRequest, Services, WordApp};

static READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: ui_shot script.jsonl")?;
    let script = std::fs::read_to_string(&path)?;
    let (tx, rx) = std::sync::mpsc::channel::<ControlRequest>();
    let lines: Vec<serde_json::Value> = script.lines().filter(|l| !l.trim().is_empty()).map(serde_json::from_str).collect::<Result<_, _>>()?;
    validate_steps(&lines)?;
    let (w, h, scale) = dimensions(&lines)?;
    let doc = if lines.first().is_some_and(|l| l.get("empty").and_then(|v| v.as_bool()) == Some(true)) {
        wordcraft_doc::Document::new()
    } else {
        wordcraft_engine::sample::sample_document()
    };
    let mut app = WordApp::new(Session::new(doc), Services::default()).with_control(rx);
    app.integrated_titlebar = true;
    if lines.first().is_some_and(|l| l.get("startScreen").and_then(|v| v.as_bool()) == Some(true)) {
        app.start_screen();
    }
    let mut harness =
        egui_kittest::Harness::builder().with_size(egui::vec2(w, h)).with_pixels_per_point(scale).with_max_steps(1_000_000).wgpu().build_ui_state(
            |ui, app: &mut WordApp| {
                if !READY.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
    harness.input_mut().max_texture_side = Some(8192);
    READY.store(true, std::sync::atomic::Ordering::Relaxed);
    step_n(&mut harness, 6);
    for (index, l) in lines.into_iter().enumerate() {
        if let Some(p) = l.get("shot").and_then(|v| v.as_str()) {
            step_n(&mut harness, 4);
            let img = harness.render().map_err(|e| format!("step {}: render failed: {e}", index + 1))?;
            img.save(p)?;
            println!("{p}");
        } else if let Some(n) = l.get("steps").and_then(|v| v.as_u64()) {
            step_n(&mut harness, bounded_count(n, 10_000, "steps")?);
        } else if let Some(ms) = l.get("wait").and_then(|v| v.as_u64()) {
            bounded_count(ms, 60_000, "wait milliseconds")?;
            let end = std::time::Instant::now() + std::time::Duration::from_millis(ms);
            while std::time::Instant::now() < end {
                step(&mut harness);
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        } else if let Some(label) = l.get("clickLabel").and_then(|v| v.as_str()) {
            click_label(&harness, label).map_err(|e| format!("step {}: {e}", index + 1))?;
            step_n(&mut harness, 8);
        } else if let Some(name) = l.get("audit").and_then(|v| v.as_str()) {
            step_n(&mut harness, 2);
            let report = audit(&harness, w, h, scale);
            println!("audit {name}: {report}");
            // `UI_SHOT_FULL=1` also lists every control, for scripts that visit each one.
            if std::env::var_os("UI_SHOT_FULL").is_some() {
                use egui_kittest::kittest::{NodeT, Queryable};
                for node in harness.query_all(egui_kittest::kittest::by()) {
                    let node = node.accesskit_node();
                    if is_control(node.role()) {
                        println!("control {:?} {} {:?}", node.role(), node.is_disabled(), node.label().unwrap_or_default());
                    }
                }
            }
            if l.get("strict").and_then(|v| v.as_bool()) == Some(true) && !report_is_clean(&report) {
                return Err(format!("step {}: audit {name} found defects", index + 1).into());
            }
        } else if let Some(m) = l.get("method").and_then(|v| v.as_str()) {
            let (req, reply) = ControlRequest::new(m, l.get("params").cloned().unwrap_or_default());
            tx.send(req)?;
            let mut received = false;
            for _ in 0..60 {
                step(&mut harness);
                if let Ok(r) = reply.try_recv() {
                    check_reply(&r, l.get("expect")).map_err(|e| format!("step {} ({m}): {e}", index + 1))?;
                    received = true;
                    let s = r.to_string();
                    // `UI_SHOT_FULL=1` prints whole replies (e.g. `document.inspect` for scripted checks).
                    let cut = if std::env::var_os("UI_SHOT_FULL").is_some() { s.len() } else { s.len().min(300) };
                    println!("{m}: {}", s.get(..cut).unwrap_or(&s));
                    break;
                }
            }
            if !received {
                return Err(format!("step {} ({m}): no reply after 60 frames", index + 1).into());
            }
            step_n(&mut harness, 2);
        } else if index != 0 || !is_config(&l) {
            return Err(format!("step {}: expected a screenshot, request, frame count, or wait", index + 1).into());
        }
    }
    Ok(())
}

type Shot<'a> = egui_kittest::Harness<'a, WordApp>;

fn click_label(harness: &Shot<'_>, label: &str) -> Result<(), String> {
    use egui_kittest::kittest::{NodeT, Queryable};
    let node = harness
        .query_all_by_label(label)
        .find(|n| !n.accesskit_node().is_disabled() && is_control(n.accesskit_node().role()))
        .ok_or_else(|| format!("no enabled control named {label:?}"))?;
    node.click();
    Ok(())
}

fn is_control(role: egui::accesskit::Role) -> bool {
    use egui::accesskit::Role::*;
    matches!(
        role,
        Button
            | CheckBox
            | ComboBox
            | EditableComboBox
            | TextInput
            | MultilineTextInput
            | SearchInput
            | SpinButton
            | Slider
            | RadioButton
            | Tab
            | MenuItem
            | Link
            | Switch
    )
}

/// One control the audit looks at: its window (the top-level tree node), the innermost scroll area
/// around it, and its box in points.
#[derive(Clone, Debug, PartialEq)]
struct AuditNode {
    role: String,
    label: String,
    disabled: bool,
    layer: u64,
    scroller: Option<u64>,
    ancestors: Vec<u64>,
    id: u64,
    rect: [f32; 4],
}

fn audit(harness: &Shot<'_>, w: f32, h: f32, scale: f32) -> serde_json::Value {
    use egui_kittest::kittest::NodeT;
    fn walk(
        node: egui_kittest::kittest::AccessKitNode<'_>,
        scale: f32,
        layer: u64,
        scroller: Option<u64>,
        ancestors: &mut Vec<u64>,
        next: &mut u64,
        out: &mut Vec<AuditNode>,
    ) {
        for child in node.children() {
            if child.is_hidden() {
                continue;
            }
            *next += 1;
            let id = *next;
            let layer = if layer == 0 { id } else { layer };
            if is_control(child.role())
                && let Some(b) = child.bounding_box()
            {
                out.push(AuditNode {
                    role: format!("{:?}", child.role()),
                    label: child.label().unwrap_or_default(),
                    disabled: child.is_disabled(),
                    layer,
                    scroller,
                    ancestors: ancestors.clone(),
                    id,
                    rect: [b.x0 as f32 / scale, b.y0 as f32 / scale, b.x1 as f32 / scale, b.y1 as f32 / scale],
                });
            }
            let inner = if child.role() == egui::accesskit::Role::ScrollView || child.clips_children() { Some(id) } else { scroller };
            ancestors.push(id);
            walk(child, scale, layer, inner, ancestors, next, out);
            ancestors.pop();
        }
    }
    let mut nodes = Vec::new();
    walk(harness.root().accesskit_node(), scale, 0, None, &mut Vec::new(), &mut 0, &mut nodes);
    audit_report(&nodes, w, h)
}

/// The rules, apart from the tree they read. A control inside a scroll area may leave the window;
/// a modal dialog above the ribbon is a different layer, so only controls sharing a layer and a
/// scroll area can overlap.
fn audit_report(nodes: &[AuditNode], w: f32, h: f32) -> serde_json::Value {
    let name = |n: &AuditNode| format!("{} {:?} [{:.0},{:.0},{:.0},{:.0}]", n.role, n.label, n.rect[0], n.rect[1], n.rect[2], n.rect[3]);
    let mut off = Vec::new();
    let mut overlap = Vec::new();
    let mut unnamed = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        let [x0, y0, x1, y1] = n.rect;
        if n.scroller.is_none() && (x0 < -1.0 || y0 < -1.0 || x1 > w + 1.0 || y1 > h + 1.0) {
            off.push(name(n));
        }
        if n.label.trim().is_empty() {
            unnamed.push(name(n));
        }
        for m in nodes.iter().skip(i + 1) {
            if m.layer != n.layer || m.scroller != n.scroller || m.ancestors.contains(&n.id) || n.ancestors.contains(&m.id) {
                continue;
            }
            let cross = (x1.min(m.rect[2]) - x0.max(m.rect[0])).max(0.0) * (y1.min(m.rect[3]) - y0.max(m.rect[1])).max(0.0);
            let smaller = ((x1 - x0) * (y1 - y0)).min((m.rect[2] - m.rect[0]) * (m.rect[3] - m.rect[1]));
            if smaller > 0.0 && cross / smaller > 0.15 {
                overlap.push(format!("{} / {}", name(n), name(m)));
            }
        }
    }
    serde_json::json!({"controls": nodes.len(), "offWindow": off, "overlap": overlap, "unnamed": unnamed})
}

fn report_is_clean(report: &serde_json::Value) -> bool {
    ["offWindow", "overlap", "unnamed"].iter().all(|key| report.get(*key).and_then(|v| v.as_array()).is_none_or(Vec::is_empty))
}

fn is_config(value: &serde_json::Value) -> bool {
    value.as_object().is_some_and(|fields| {
        !fields.is_empty()
            && fields.iter().all(|(key, value)| match key.as_str() {
                "width" | "height" | "pixelsPerPoint" => value.is_number(),
                "empty" | "startScreen" => value.is_boolean(),
                _ => false,
            })
    })
}

fn validate_steps(lines: &[serde_json::Value]) -> Result<(), String> {
    let mut actions = 0;
    for (index, line) in lines.iter().enumerate() {
        if index == 0 && is_config(line) {
            continue;
        }
        let fields = line.as_object().ok_or_else(|| format!("step {} must be an object", index + 1))?;
        let kinds: Vec<_> = ["method", "shot", "steps", "wait", "clickLabel", "audit"].into_iter().filter(|k| fields.contains_key(*k)).collect();
        let [kind] = kinds.as_slice() else {
            return Err(format!("step {} must have exactly one action", index + 1));
        };
        let allowed: &[&str] = match *kind {
            "method" => &["method", "params", "expect", "id"],
            "audit" => &["audit", "strict"],
            _ => std::slice::from_ref(kind),
        };
        if fields.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err(format!("step {}: unsupported field; expect is only valid on a request", index + 1));
        }
        actions += 1;
    }
    if actions == 0 {
        return Err("a scenario must contain at least one action".into());
    }
    Ok(())
}

fn dimensions(lines: &[serde_json::Value]) -> Result<(f32, f32, f32), String> {
    let number = |key: &str, default: f64| -> Result<f32, String> {
        let n = match lines.first().and_then(|line| line.get(key)) {
            Some(v) => v.as_f64().ok_or_else(|| format!("{key} must be a number"))?,
            None => default,
        };
        let min = if key == "pixelsPerPoint" { 0.5 } else { 1.0 };
        if !n.is_finite() || n < min || n > 8192.0 {
            return Err(format!("{key} must be between {min} and 8192"));
        }
        Ok(n as f32)
    };
    let (w, h, scale) = (number("width", 1440.0)?, number("height", 900.0)?, number("pixelsPerPoint", 2.0)?);
    if !(0.5..=4.0).contains(&scale) || w * scale > 8192.0 || h * scale > 8192.0 {
        return Err("pixelsPerPoint must be 0.5–4 and the image must fit within 8192×8192 pixels".into());
    }
    Ok((w, h, scale))
}

fn bounded_count(value: u64, max: u64, name: &str) -> Result<usize, String> {
    if value > max {
        return Err(format!("{name} must be at most {max}"));
    }
    usize::try_from(value).map_err(|e| format!("{name}: {e}"))
}

fn check_reply(reply: &serde_json::Value, expected: Option<&serde_json::Value>) -> Result<(), String> {
    let permits_error = expected.and_then(|v| v.get("ok")).and_then(|v| v.as_bool()) == Some(false);
    if reply.get("ok").and_then(|v| v.as_bool()) != Some(!permits_error) {
        return Err(format!("unexpected command outcome: {reply}"));
    }
    if let Some(expected) = expected
        && !contains_reply(reply, expected)
    {
        return Err(format!("expected {expected}, received {reply}"));
    }
    Ok(())
}

fn contains_reply(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match expected {
        serde_json::Value::Object(fields) => fields.iter().all(|(k, v)| actual.get(k).is_some_and(|a| contains_reply(a, v))),
        _ => actual == expected,
    }
}

/// Applies the harness's pending raw input and advances one UI frame.
fn step(harness: &mut egui_kittest::Harness<'_, WordApp>) {
    let mut raw = std::mem::take(harness.input_mut());
    let ctx = harness.ctx.clone();
    harness.state_mut().raw_input_hook(&ctx, &mut raw);
    *harness.input_mut() = raw;
    harness.step();
}

fn step_n(harness: &mut egui_kittest::Harness<'_, WordApp>, n: usize) {
    for _ in 0..n {
        step(harness);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn screenshot_dimensions_validate_before_allocating() {
        assert_eq!(dimensions(&[]).unwrap(), (1440.0, 900.0, 2.0));
        assert_eq!(dimensions(&[json!({"width": 800, "height": 600, "pixelsPerPoint": 1.25})]).unwrap(), (800.0, 600.0, 1.25));
        for input in [
            json!({"width": -1}),
            json!({"height": 0}),
            json!({"width": 1e-100}),
            json!({"width": "800"}),
            json!({"pixelsPerPoint": 0.1}),
            json!({"width": 8192}),
        ] {
            assert!(dimensions(&[input]).is_err());
        }
    }

    #[test]
    fn excessive_frame_counts_and_waits_are_refused() {
        assert_eq!(bounded_count(0, 10_000, "steps").unwrap(), 0);
        assert_eq!(bounded_count(60_000, 60_000, "wait").unwrap(), 60_000);
        assert!(bounded_count(u64::MAX, 60_000, "wait").is_err());
        assert!(bounded_count(u64::MAX, 10_000, "steps").is_err());
    }

    #[test]
    fn malformed_actions_are_not_treated_as_configuration() {
        assert!(is_config(&json!({"empty": false, "width": 800, "pixelsPerPoint": 1.25})));
        for value in [json!({"method": 7}), json!({"shot": true}), json!({"empty": "false"}), json!({"steps": -1}), json!({}), json!(null)] {
            assert!(!is_config(&value));
        }
    }

    #[test]
    fn scenarios_cannot_silently_skip_actions_or_assertions() {
        for lines in [
            vec![],
            vec![json!({"width": 800})],
            vec![json!({"shot": "out.png", "method": "ui.inspect"})],
            vec![json!({"shot": "out.png", "expect": {"ok": true}})],
            vec![json!({"method": "ui.inspect", "width": 800})],
        ] {
            assert!(validate_steps(&lines).is_err());
        }
        assert!(validate_steps(&[json!({"empty": true}), json!({"method": "ui.inspect", "expect": {"ok": true}})]).is_ok());
    }

    #[test]
    fn scenarios_fail_on_refusal_or_wrong_state() {
        let success = json!({"ok": true, "result": {"tab": "Home", "extra": 1}});
        let refusal = json!({"ok": false, "error": "disabled"});
        assert!(check_reply(&success, Some(&json!({"result": {"tab": "Home"}}))).is_ok());
        assert!(check_reply(&success, Some(&json!({"result": {"tab": "Insert"}}))).is_err());
        assert!(check_reply(&refusal, None).is_err());
        assert!(check_reply(&refusal, Some(&json!({"ok": false, "error": "disabled"}))).is_ok());
        assert!(check_reply(&success, Some(&json!({"ok": false}))).is_err());
        assert!(check_reply(&json!({}), None).is_err());
    }

    fn node(label: &str, layer: u64, scroller: Option<u64>, id: u64, ancestors: Vec<u64>, rect: [f32; 4]) -> AuditNode {
        AuditNode { role: "Button".into(), label: label.into(), disabled: false, layer, scroller, ancestors, id, rect }
    }

    #[test]
    fn audit_reports_clipped_overlapping_and_unnamed_controls() {
        let report = audit_report(
            &[
                node("clipped", 1, None, 1, vec![], [90.0, 0.0, 110.0, 10.0]),
                node("scrolled away", 1, Some(9), 2, vec![], [0.0, 500.0, 10.0, 510.0]),
                node("a", 1, None, 3, vec![], [0.0, 20.0, 20.0, 40.0]),
                node("b", 1, None, 4, vec![], [5.0, 25.0, 25.0, 45.0]),
                node("", 1, None, 5, vec![], [40.0, 40.0, 50.0, 50.0]),
            ],
            100.0,
            100.0,
        );
        assert_eq!(report["offWindow"].as_array().unwrap().len(), 1);
        assert!(report["offWindow"][0].as_str().unwrap().contains("clipped"));
        assert_eq!(report["overlap"].as_array().unwrap().len(), 1);
        assert_eq!(report["unnamed"].as_array().unwrap().len(), 1);
        assert!(!report_is_clean(&report));
    }

    #[test]
    fn audit_allows_other_layers_nested_controls_and_touching_edges() {
        let report = audit_report(
            &[
                node("ribbon", 1, None, 1, vec![], [0.0, 0.0, 50.0, 50.0]),
                node("dialog", 2, None, 2, vec![], [0.0, 0.0, 50.0, 50.0]),
                node("combo", 1, None, 3, vec![], [60.0, 0.0, 90.0, 20.0]),
                node("arrow", 1, None, 4, vec![3], [70.0, 0.0, 90.0, 20.0]),
                node("next", 1, None, 5, vec![], [60.0, 20.0, 90.0, 40.0]),
            ],
            100.0,
            100.0,
        );
        assert!(report_is_clean(&report), "{report}");
    }

    #[test]
    fn label_clicks_and_audits_are_valid_steps() {
        assert!(validate_steps(&[json!({"clickLabel": "OK"}), json!({"audit": "home", "strict": true})]).is_ok());
        assert!(validate_steps(&[json!({"clickLabel": "OK", "strict": true})]).is_err());
    }
}
