# UI audit scenarios

Build `cargo build -p wordcraft-ui-egui --example ui_shot`. From an evidence directory
outside the checkout, run the resulting `target/debug/examples/ui_shot` with an absolute
path to one of these JSONL files. Relative screenshot paths are written to that evidence
directory. Use consistent `CRAFT_FONTS_DIR` and `CRAFT_I18N_DIR` inputs when comparing builds,
and record their revisions with the evidence. Keep captures in `~/agent-runs/YYYY-MM-DD-topic/`, never in build folders.

- `dialog-input.jsonl`: document text and selection stay unchanged while Font receives keyboard input.
- `editing.jsonl`: select, bold, undo/redo, open Font and cancel from the keyboard.
- `modeless-goto.jsonl`: Page/Bookmark targets permit document keyboard selection; Escape preserves it.
- `read-compact.jsonl`: compact Read Mode, fractional display scaling.
- `dialogs-short.jsonl`: six dialog families in a short viewport.

The runner exits unsuccessfully on render/save errors, command refusal or missing replies.
An `expect` object checks a recursive subset of a control reply; arrays and scalar values
must match exactly. Expected refusals require `"expect": {"ok": false}`. Screenshot generation
is not a visual pass: inspect each PNG and record defects or acceptance in `docs/ui-audit.md`.

The first line configures size in points and `pixelsPerPoint`; defaults are 1440×900 at 2×.
A step may advance at most 10,000 frames or wait at most 60,000 milliseconds.
Repeat wait steps when background work needs longer. Malformed steps fail the run.
For boundary checks, copy a scenario into the evidence directory and change that line.
Run helper regressions with `cargo test -p wordcraft-ui-egui --example ui_shot`.

Pointer input is queued across frames. Add a `{"steps": 8}` settling step after `ui.click`
before asserting its final effect. Drags, animations and background work may need more frames
or an explicit state check. An immediate query can observe the state before the queued click.
