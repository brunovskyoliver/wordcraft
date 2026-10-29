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
        let kinds: Vec<_> = ["method", "shot", "steps", "wait"].into_iter().filter(|k| fields.contains_key(*k)).collect();
        let [kind] = kinds.as_slice() else {
            return Err(format!("step {} must have exactly one action", index + 1));
        };
        let allowed: &[&str] = if *kind == "method" { &["method", "params", "expect", "id"] } else { std::slice::from_ref(kind) };
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
}
