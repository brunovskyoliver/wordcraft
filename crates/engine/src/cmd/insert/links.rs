use super::{CharProps, CmdError, CmdResult, InlineObject, Pos, Selection, Session, StoryRef, Value, json, p, sel_result};

/// What a link can point to inside the document (Insert › Link › Place in This Document): the top
/// of the document, its headings (body block, level, text) and its bookmarks, without the hidden
/// ones (names starting with `_`).
pub(super) fn link_targets(s: &mut Session, _: &Value) -> CmdResult {
    let headings: Vec<Value> = crate::cmd::references::headings(s)
        .into_iter()
        .map(|(block, level, text, _)| json!({"block": block, "level": level, "text": text}))
        .collect();
    let mut bookmarks: Vec<String> = Vec::new();
    for (name, _) in s.doc.bookmarks() {
        if !name.starts_with('_') && !bookmarks.contains(&name) {
            bookmarks.push(name);
        }
    }
    Ok(json!({"top": true, "headings": headings, "bookmarks": bookmarks}))
}

/// The address and the default text of a link to a place in this document, from `top`, `bookmark`
/// or `heading` (a heading gets a hidden `_Toc` bookmark, as in Word); None when none is given.
fn place_target(s: &mut Session, v: &Value) -> Result<Option<(String, String)>, CmdError> {
    if p::bool(v, "top") == Some(true) {
        return Ok(Some(("#_top".into(), "Top of the Document".into())));
    }
    if let Some(name) = p::str(v, "bookmark") {
        if !s.doc.bookmarks().iter().any(|(n, _)| n == name) {
            return Err(CmdError::Params(format!("no bookmark `{name}`")));
        }
        return Ok(Some((format!("#{name}"), name.to_string())));
    }
    if let Some(block) = p::u64(v, "heading") {
        let Some((i, _, text, lead)) = crate::cmd::references::headings(s).into_iter().find(|h| h.0 as u64 == block) else {
            return Err(CmdError::Params(format!("block {block} is not a heading")));
        };
        let name = crate::cmd::references::toc_bookmark(s, i, lead)?;
        return Ok(Some((format!("#{name}"), text)));
    }
    Ok(None)
}

/// Bounded ordinary-text target shared by dialogs and programmatic edits. Positions supplied
/// by a caller are never trusted: edit_link compares this with a freshly derived context.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkContext {
    document: u64,
    revision: u64,
    body_epoch: u64,
    selection: Selection,
    start: Pos,
    end: Pos,
    pub text: String,
    pub url: Option<String>,
}

/// A caret strictly inside an ordinary-text hyperlink edits its entire contiguous URL run.
/// Link boundaries remain insertion points. Complex selections are deliberately unsupported.
pub fn link_context(s: &Session) -> Option<LinkContext> {
    if s.column_segments().is_some() || s.cell_range().is_some() || s.math.is_some() || crate::cmd::objects::object_selection(s).is_some() {
        return None;
    }
    let (mut a, mut b) = s.sel.ordered();
    if a.story != b.story || a.path != b.path {
        return None;
    }
    let para = s.doc.para_at(&a)?;
    if !para.text.is_char_boundary(a.off) || !para.text.is_char_boundary(b.off) {
        return None;
    }
    if s.sel.is_collapsed() {
        let mut cluster: Option<(usize, usize, &str)> = None;
        for (range, props) in para.run_ranges() {
            if let Some((lo, hi, url)) = cluster {
                if props.link.as_deref() == Some(url) && hi == range.start {
                    cluster = Some((lo, range.end, url));
                    continue;
                }
                if lo < a.off && a.off < hi {
                    a.off = lo;
                    b.off = hi;
                    break;
                }
            }
            cluster = props.link.as_deref().map(|url| (range.start, range.end, url));
        }
        if a == b
            && let Some((lo, hi, _)) = cluster
            && lo < a.off
            && a.off < hi
        {
            a.off = lo;
            b.off = hi;
        }
    }
    let raw = para.text.get(a.off..b.off)?;
    if raw.len() > 65536 || raw.chars().any(|c| c == wordcraft_doc::para::OBJ || c == '\u{2028}' || c == '\u{2029}' || c.is_control()) {
        return None;
    }
    // Never flatten tracked deletions/insertions or their formatting history into a label.
    if para.run_ranges().any(|(r, c)| r.start < b.off && a.off < r.end && (c.del.is_some() || c.ins.is_some() || c.fmt_change.is_some())) {
        return None;
    }
    let url = if a.off < b.off {
        let first = para.props_of_char(a.off).link.as_ref().filter(|url| url.len() <= 4096).cloned();
        first.filter(|url| para.run_ranges().filter(|(r, _)| r.start < b.off && a.off < r.end).all(|(_, c)| c.link.as_ref() == Some(url)))
    } else {
        None
    };
    Some(LinkContext {
        document: s.document_id(),
        revision: s.rev(),
        body_epoch: s.ime_body_epoch(),
        selection: s.sel.clone(),
        start: a,
        end: b,
        text: raw.into(),
        url,
    })
}

pub(super) fn link_enabled(s: &Session) -> Option<&'static str> {
    // Refuse before column dispatch can split a block into independent ordinary-text edits.
    (s.column_segments().is_some() || s.cell_range().is_some()).then_some("select ordinary text instead of a block or whole cells")
}

pub(super) fn edit_link(s: &mut Session, v: &Value) -> CmdResult {
    let target = link_context(s).ok_or_else(|| CmdError::Params("select ordinary text or a text insertion point".into()))?;
    if v.get("target") != Some(&json!(&target)) {
        return Err(CmdError::Params("the document or selection changed; reopen the link dialog".into()));
    }
    apply_link(s, v, Some(target))
}

pub(super) fn link(s: &mut Session, v: &Value) -> CmdResult {
    if v.get("target").is_some() {
        return Err(CmdError::Params("use insert.editLink to validate an explicit target".into()));
    }
    let target = link_context(s);
    apply_link(s, v, target)
}

fn apply_link(s: &mut Session, v: &Value, target: Option<LinkContext>) -> CmdResult {
    if v.get("text").is_some_and(|v| !v.is_string()) {
        return Err(CmdError::Params("`text` must be text".into()));
    }
    if let Some(text) = p::str(v, "text") {
        if text.is_empty()
            || text.len() > 65536
            || text.chars().any(|c| c.is_control() || c == wordcraft_doc::para::OBJ || c == '\u{2028}' || c == '\u{2029}')
        {
            return Err(CmdError::Params("the display text must be nonempty ordinary text, at most 65536 bytes".into()));
        }
        if target.is_none() {
            return Err(CmdError::Params("display text can only replace ordinary text in one paragraph".into()));
        }
    }
    let (url, label) = match p::str(v, "url") {
        Some(url) => {
            if url.len() > 4096 {
                return Err(CmdError::Params("the link address is too long".into()));
            }
            (url.to_string(), url.to_string())
        }
        None => match place_target(s, v)? {
            Some(t) => t,
            None => return Err(CmdError::Params("a link address or document place is required".into())),
        },
    };
    if url.is_empty() {
        return Err(CmdError::Params("a link address is required".into()));
    }
    if url.len() > 4096 {
        return Err(CmdError::Params("the link address is too long".into()));
    }
    let style = Some("Hyperlink".to_string());
    if let Some(target) = target {
        s.sel = Selection { anchor: target.start, focus: target.end };
        let text = p::str(v, "text").unwrap_or(if s.sel.is_collapsed() { &label } else { &target.text });
        if text != target.text || s.sel.is_collapsed() {
            if text.is_empty()
                || text.len() > 65536
                || text.chars().any(|c| c.is_control() || c == wordcraft_doc::para::OBJ || c == '\u{2028}' || c == '\u{2029}')
            {
                return Err(CmdError::Params("the display text must be nonempty ordinary text, at most 65536 bytes".into()));
            }
            // Shared insertion handles protection, tracking and inherited character formatting.
            // Put link properties on the inserted text only, never the retained tracked deletion.
            crate::cmd::type_text(s, text)?;
            let end = s.sel.focus.clone();
            let off = end.off.checked_sub(text.len()).ok_or_else(|| CmdError::Failed("the link label insertion range is invalid".into()))?;
            let start = Pos { off, ..end.clone() };
            s.doc.format_range(&start, &end, &|c| {
                c.link = Some(url.clone());
                c.style = style.clone();
            })?;
            s.pending = Some(CharProps::default());
            return sel_result(s);
        }
        let (a, b) = s.sel.ordered();
        crate::cmd::controls::check_edit(s, &a, &b)?;
        // Unchanged labels retain every rich run, and the user's original selection direction.
        s.doc.format_range(&s.sel.anchor, &s.sel.focus, &|c| {
            c.link = Some(url.clone());
            c.style = style.clone();
        })?;
        s.sel = target.selection;
    } else {
        let (a, b) = s.sel.ordered();
        crate::cmd::controls::check_edit(s, &a, &b)?;
        s.doc.format_range(&a, &b, &|c| {
            c.link = Some(url.clone());
            c.style = style.clone();
        })?;
    }
    sel_result(s)
}

/// Whether `name` can name a bookmark: starts with a letter or underscore, contains only alphanumeric characters
/// or underscores, and is at most 40 characters (#3491). Word allows leading underscores for hidden bookmarks (_Toc, _Ref).
pub fn bookmark_name_ok(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_') && chars.all(|c| c.is_alphanumeric() || c == '_') && name.chars().count() <= 40
}

/// Where bookmark `name` is: see [`bookmark_span`]. `None` when there is no such bookmark.
pub fn bookmark_range(doc: &wordcraft_doc::Document, name: &str) -> Option<(Pos, Pos)> {
    let (_, start) = doc.bookmarks().into_iter().find(|(n, _)| n == name)?;
    Some(bookmark_span(doc, name, &start))
}

/// The text of bookmark `name` whose start mark is at `start`: from just after that mark to its
/// end mark — in a later paragraph when it spans several (the start again when it has no end).
pub fn bookmark_span(doc: &wordcraft_doc::Document, name: &str, start: &Pos) -> (Pos, Pos) {
    let from = Pos { off: start.off.saturating_add(wordcraft_doc::para::OBJ.len_utf8()), ..start.clone() };
    let end = doc.para_paths(start.story).into_iter().skip_while(|p| *p != start.path).find_map(|path| {
        let para = doc.para(start.story, &path)?;
        let off = para.object_offsets().into_iter().find(|o| {
            (path != start.path || *o > start.off) && matches!(para.object_at(*o), Some(InlineObject::BookmarkEnd { name: n }) if n == name)
        })?;
        Some(Pos { story: start.story, path, off })
    });
    let end = end.unwrap_or_else(|| from.clone());
    (from, end)
}

/// Remove bookmark `name`'s start and end marks, in whichever paragraphs and stories they are,
/// keeping the selection on the same text. Returns whether the bookmark was there.
fn remove_bookmark(s: &mut Session, name: &str) -> Result<bool, CmdError> {
    let mark_len = wordcraft_doc::para::OBJ.len_utf8();
    let mut found = false;
    let stories: Vec<StoryRef> = std::iter::once(StoryRef::Body).chain(s.doc.parts.keys().map(|k| StoryRef::Part(*k))).collect();
    for story in stories {
        for path in s.doc.para_paths(story) {
            let Some(p) = s.doc.para(story, &path) else { continue };
            let mut marks = Vec::new();
            for off in p.object_offsets() {
                match p.object_at(off) {
                    Some(InlineObject::BookmarkStart { name: n }) if n == name => {
                        found = true;
                        marks.push(off);
                    }
                    Some(InlineObject::BookmarkEnd { name: n }) if n == name => marks.push(off),
                    _ => {}
                }
            }
            if marks.is_empty() {
                continue;
            }
            let para = s.doc.para_mut(story, &path)?;
            for off in marks.into_iter().rev() {
                para.delete(off, off + mark_len)?;
                for at in [&mut s.sel.anchor, &mut s.sel.focus] {
                    if at.story == story && at.path == path && at.off > off {
                        at.off = at.off.saturating_sub(mark_len);
                    }
                }
            }
        }
    }
    s.clamp_selection();
    Ok(found)
}

pub(super) fn delete_bookmark(s: &mut Session, v: &Value) -> CmdResult {
    let name = p::req_str(v, "name")?.trim();
    if !remove_bookmark(s, name)? {
        return Err(CmdError::Params(format!("no bookmark `{name}`")));
    }
    sel_result(s)
}

pub(super) fn bookmark(s: &mut Session, v: &Value) -> CmdResult {
    let name = p::req_str(v, "name")?.trim().to_string();
    if !bookmark_name_ok(&name) {
        return Err(CmdError::Params("bookmark names start with a letter, have no spaces and at most 40 characters".into()));
    }
    // Replace an existing bookmark of the same name.
    remove_bookmark(s, &name)?;
    let (a, b) = s.sel.ordered();
    let props = CharProps::default();
    let end = s.doc.insert_object(&b, InlineObject::BookmarkEnd { name: name.clone() }, &props)?;
    s.doc.insert_object(&a, InlineObject::BookmarkStart { name: name.clone() }, &props)?;
    let _ = end;
    sel_result(s)
}

#[cfg(test)]
mod hyperlink_tests {
    use super::*;
    use wordcraft_doc::{Document, para::OBJ};

    fn selected() -> Session {
        let mut s = Session::new(Document::from_text("Alpha Beta"));
        s.sel = Selection { anchor: Pos::body(0, 5), focus: Pos::body(0, 0) };
        s
    }
    fn edit(s: &mut Session, text: &str) -> CmdResult {
        let target = link_context(s);
        s.run("insert.editLink", &json!({"target": target, "url": "https://example.invalid/new", "text": text}))
    }
    fn plain(s: &Session) -> String {
        s.doc.plain_text(StoryRef::Body)
    }

    #[test]
    fn hyperlink_label_replacement_restores_rich_selection_in_one_undo() {
        let mut s = selected();
        s.doc.para_mut(StoryRef::Body, &wordcraft_doc::Path::top(0)).unwrap().format(0, 2, &|c| c.bold = Some(true)).unwrap();
        let original = s.doc.clone();
        let selection = s.sel.clone();
        edit(&mut s, "Revised").unwrap();
        assert_eq!(plain(&s), "Revised Beta");
        assert_eq!(s.undo_depth(), 1);
        let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
        assert_eq!(p.props_of_char(0).bold, Some(true));
        assert_eq!(p.props_of_char(0).link.as_deref(), Some("https://example.invalid/new"));
        s.run("edit.undo", &json!({})).unwrap();
        assert!(original.same_content(&s.doc));
        assert_eq!(s.sel, selection);
        s.run("edit.redo", &json!({})).unwrap();
        assert_eq!(plain(&s), "Revised Beta");
    }

    #[test]
    fn hyperlink_unchanged_label_preserves_rich_runs_and_direction() {
        let mut s = selected();
        s.doc.para_mut(StoryRef::Body, &wordcraft_doc::Path::top(0)).unwrap().format(0, 2, &|c| c.italic = Some(true)).unwrap();
        let selection = s.sel.clone();
        edit(&mut s, "Alpha").unwrap();
        let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
        assert_eq!(p.props_of_char(0).italic, Some(true));
        assert_eq!(p.props_of_char(3).italic, None);
        assert_eq!(s.sel, selection);
        assert_eq!(plain(&s), "Alpha Beta");
    }

    #[test]
    fn hyperlink_caret_context_preloads_full_rich_link_and_updates_address() {
        let mut s = selected();
        edit(&mut s, "Alpha").unwrap();
        s.doc.para_mut(StoryRef::Body, &wordcraft_doc::Path::top(0)).unwrap().format(2, 5, &|c| c.bold = Some(true)).unwrap();
        s.sel = Selection::caret(Pos::body(0, 3));
        let c = link_context(&s).unwrap();
        assert_eq!(c.text, "Alpha");
        assert_eq!(c.url.as_deref(), Some("https://example.invalid/new"));
        s.run("insert.editLink", &json!({"target": c, "url": "https://example.invalid/updated", "text": "Alpha"})).unwrap();
        assert_eq!(s.sel, Selection::caret(Pos::body(0, 3)));
        let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
        assert_eq!(p.props_of_char(0).link.as_deref(), Some("https://example.invalid/updated"));
        assert_eq!(p.props_of_char(4).link, p.props_of_char(0).link);
        assert_eq!(p.props_of_char(3).bold, Some(true));
        s.sel = Selection::caret(Pos::body(0, 5));
        assert_eq!(link_context(&s).unwrap().text, "");
    }

    #[test]
    fn hyperlink_stale_selection_document_and_undo_receipts_leave_no_edit_or_replay() {
        for change in 0..3 {
            let mut s = selected();
            let target = link_context(&s);
            match change {
                0 => s.sel = Selection::caret(Pos::body(0, 6)),
                1 => {
                    s.run("text.insert", &json!({"text": "Other"})).unwrap();
                    s.run("edit.undo", &json!({})).unwrap();
                }
                _ => {
                    s = selected();
                }
            }
            let before = s.doc.clone();
            let depth = s.undo_depth();
            let selection = s.sel.clone();
            let previous = s.last_command.clone();
            s.recording = Some(("test".into(), Vec::new()));
            assert!(s.run("insert.editLink", &json!({"target": target, "url": "https://example.invalid", "text": "Oops"})).is_err());
            assert!(before.same_content(&s.doc));
            assert_eq!(s.undo_depth(), depth);
            assert_eq!(s.sel, selection);
            assert_eq!(s.last_command, previous);
            assert!(s.recording.as_ref().unwrap().1.is_empty());
        }
    }

    #[test]
    fn hyperlink_protected_and_invalid_requests_preserve_document_and_history() {
        for mode in ["readOnly", "comments", "forms"] {
            let mut s = selected();
            s.run("review.restrict", &json!({"mode": mode})).unwrap();
            let before = s.doc.clone();
            let depth = s.undo_depth();
            let selection = s.sel.clone();
            assert!(edit(&mut s, "Revised").is_err());
            assert!(before.same_content(&s.doc));
            assert_eq!(s.undo_depth(), depth);
            assert_eq!(s.sel, selection);
        }
        let mut s = selected();
        for text in ["", "two\nlines", "\u{fffc}"] {
            assert!(edit(&mut s, text).is_err());
            assert_eq!(plain(&s), "Alpha Beta");
            assert_eq!(s.undo_depth(), 0);
        }
        assert!(s.run("insert.link", &json!({})).is_err());
        assert!(s.ui_requests.is_empty());
    }

    #[test]
    fn hyperlink_complex_selection_refuses_label_flattening() {
        let mut s = selected();
        s.doc.insert_object(&Pos::body(0, 2), InlineObject::BookmarkStart { name: "mark".into() }, &CharProps::default()).unwrap();
        s.sel = Selection { anchor: Pos::body(0, 0), focus: Pos::body(0, 5 + OBJ.len_utf8()) };
        assert!(link_context(&s).is_none());
        let before = s.doc.clone();
        assert!(edit(&mut s, "Oops").is_err());
        assert!(before.same_content(&s.doc));
        let mut s = Session::new(Document::from_text("Alpha\nBeta"));
        s.sel = Selection { anchor: Pos::body(0, 0), focus: Pos::body(1, 4) };
        assert!(link_context(&s).is_none());
        assert!(edit(&mut s, "Oops").is_err());
        assert_eq!(s.undo_depth(), 0);
    }

    #[test]
    fn hyperlink_repeat_and_macro_record_relative_operations_only_after_success() {
        let mut s = selected();
        s.recording = Some(("test".into(), Vec::new()));
        edit(&mut s, "Label").unwrap();
        let replay = s.recording.take().unwrap().1;
        assert_eq!(replay.len(), 1);
        assert_eq!(replay[0].0, "insert.link");
        assert!(replay[0].1.get("target").is_none());
        s.sel = Selection { anchor: Pos::body(0, 6), focus: Pos::body(0, 10) };
        s.run("edit.repeat", &json!({})).unwrap();
        assert_eq!(plain(&s), "Label Label");
        s.run("edit.undo", &json!({})).unwrap();
        s.sel = Selection { anchor: Pos::body(0, 6), focus: Pos::body(0, 10) };
        s.run(&replay[0].0, &replay[0].1).unwrap();
        assert_eq!(plain(&s), "Label Label");
    }

    #[test]
    fn hyperlink_label_replacement_uses_tracked_text_insertion() {
        let mut s = selected();
        s.doc.settings.track_changes = true;
        edit(&mut s, "Revised").unwrap();
        let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
        assert!(p.run_ranges().any(|(_, c)| c.ins.is_some() && c.link.as_deref() == Some("https://example.invalid/new")));
        assert!(p.run_ranges().any(|(_, c)| c.del.is_some()));
        assert_eq!(s.undo_depth(), 1);
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(plain(&s), "Alpha Beta");
    }
    #[test]
    fn hyperlink_block_selection_is_refused_before_column_dispatch() {
        let mut s = selected();
        s.column = Some(crate::ColumnBlock {
            sel: s.sel.clone(),
            anchor_x: 0.0,
            focus_x: 10.0,
            segments: vec![(Pos::body(0, 0), Pos::body(0, 5))],
            cells: None,
        });
        assert!(link_context(&s).is_none());
        assert!(s.run("insert.link", &json!({"url": "https://example.invalid", "text": "Oops"})).is_err());
        assert_eq!(plain(&s), "Alpha Beta");
        assert_eq!(s.undo_depth(), 0);
    }

    #[test]
    fn hyperlink_locked_control_refuses_changed_and_unchanged_labels() {
        let mut s = selected();
        s.run("developer.richText", &json!({"tag": "locked"})).unwrap();
        s.run("developer.properties", &json!({"control": "locked", "lockEdit": true})).unwrap();
        let range = s.doc.control_ranges(StoryRef::Body).into_iter().next().unwrap();
        s.sel = Selection { anchor: range.content_start(), focus: range.end };
        let before = s.doc.clone();
        let depth = s.undo_depth();
        let unchanged = s.selected_text();
        for label in [unchanged.as_str(), "Revised"] {
            assert!(edit(&mut s, label).is_err());
            assert!(before.same_content(&s.doc));
            assert_eq!(s.undo_depth(), depth);
        }
    }

    #[test]
    fn hyperlink_context_query_is_pure_and_receipts_do_not_become_relative_implicitly() {
        let mut s = selected();
        let context = s.run("link.context", &json!({})).unwrap();
        assert_eq!(context, json!(link_context(&s)));
        assert!(!s.can_undo());
        assert!(s.ui_requests.is_empty());
        assert!(s.run("insert.link", &json!({"target": context, "url": "https://example.invalid", "text": "Oops"})).is_err());
        assert_eq!(plain(&s), "Alpha Beta");
    }
}
