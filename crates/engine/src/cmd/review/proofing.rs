use super::{CmdError, CmdResult, Pos, Selection, Session, StoryRef, Value, json, p, pos_json};

pub(super) fn word_count(s: &mut Session, v: &Value) -> CmdResult {
    if let Some(b) = p::bool(v, "includeTextBoxes") {
        s.prefs.count_notes = b;
    }
    let stories = if s.prefs.count_notes { s.doc.counted_stories() } else { vec![StoryRef::Body] };
    // A selected picture, shape or text box isn't a text selection: count the document.
    let text = if s.sel.is_collapsed() || crate::cmd::objects::object_selection(s).is_some() {
        stories.iter().map(|st| s.doc.plain_text(*st)).collect::<Vec<_>>().join("\n")
    } else {
        s.selected_text()
    };
    let words = if s.sel.is_collapsed() || crate::cmd::objects::object_selection(s).is_some() {
        stories.iter().map(|st| s.doc.story_word_count(*st)).sum()
    } else {
        wordcraft_doc::count_words(&text)
    };
    let chars = text.chars().filter(|c| *c != '\n' && *c != '\r').count();
    let chars_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();
    let paragraphs = text.split('\n').filter(|l| !l.trim().is_empty()).count();
    let lines: usize = {
        let l = s.layout();
        l.pages
            .iter()
            .flat_map(|p| p.items.iter())
            .map(|it| {
                if let wordcraft_layout::Placed::Lines { l0, l1, story, .. } = it
                    && stories.contains(story)
                {
                    l1 - l0
                } else {
                    0
                }
            })
            .sum()
    };
    let pages = s.layout().pages.len();
    Ok(json!({
        "pages": pages, "words": words, "characters": chars_no_spaces, "charactersWithSpaces": chars, "paragraphs": paragraphs, "lines": lines,
        "includeTextBoxes": s.prefs.count_notes,
    }))
}

/// Issues (spelling + grammar) in one paragraph as positions, each in its run's language and
/// skipping text left unchecked — the same issues the layout underlines.
fn para_issues(s: &Session, story: StoryRef, path: &wordcraft_doc::Path) -> Vec<(Pos, Pos, wordcraft_proof::Issue)> {
    let Some(p) = s.doc.para(story, path) else { return Vec::new() };
    let runs = wordcraft_layout::para::proof_runs(&s.doc, p, None);
    wordcraft_layout::para::paragraph_issues(p, &runs, &s.prefs.spelling(), true, wordcraft_layout::para::is_list_item(&s.doc, p))
        .into_iter()
        .map(|i| (Pos { story, path: path.clone(), off: i.start }, Pos { story, path: path.clone(), off: i.end }, i))
        .collect()
}

/// The issue under a position.
pub(super) fn issue_at(s: &Session, at: &Pos) -> Option<(Pos, Pos, wordcraft_proof::Issue)> {
    para_issues(s, at.story, &at.path).into_iter().find(|(a, b, _)| a.off <= at.off && at.off <= b.off)
}

fn issue_json(s: &Session, a: &Pos, b: &Pos, i: &wordcraft_proof::Issue) -> Value {
    let word = s.doc.para_at(a).and_then(|p| p.text.get(a.off..b.off)).unwrap_or("").to_string();
    let sugg = if i.kind == wordcraft_proof::IssueKind::Spelling { wordcraft_proof::suggest_in(&word, 6, i.lang) } else { i.suggestions.clone() };
    json!({
        "start": pos_json(a), "end": pos_json(b), "text": word,
        "kind": if i.kind == wordcraft_proof::IssueKind::Spelling { "spelling" } else { "grammar" },
        "message": i.message, "suggestions": sugg, "lang": i.lang.code(),
    })
}

/// F7: select the next issue after the caret and return it with suggestions.
pub(super) fn next_issue(s: &mut Session, _: &Value) -> CmdResult {
    let caret = s.sel.ordered().1;
    let story = caret.story;
    let paths = s.doc.para_paths(story);
    let start = paths.iter().position(|p| *p == caret.path).unwrap_or(0);
    for k in 0..paths.len() {
        let Some(path) = paths.get((start + k) % paths.len().max(1)) else { break };
        for (a, b, i) in para_issues(s, story, path) {
            if k == 0 && a.off < caret.off && start + k < paths.len() && paths.len() > 1 {
                continue;
            }
            let r = issue_json(s, &a, &b, &i);
            s.sel = Selection { anchor: a, focus: b };
            return Ok(r);
        }
    }
    s.status = "The spelling and grammar check is complete.".into();
    Ok(json!({"done": true}))
}

pub(super) fn all_issues(s: &mut Session, _: &Value) -> CmdResult {
    let mut out = Vec::new();
    for path in s.doc.para_paths(StoryRef::Body) {
        for (a, b, i) in para_issues(s, StoryRef::Body, &path) {
            out.push(issue_json(s, &a, &b, &i));
            if out.len() >= 500 {
                return Ok(Value::Array(out));
            }
        }
    }
    Ok(Value::Array(out))
}

pub(super) fn suggestions(s: &mut Session, v: &Value) -> CmdResult {
    let at = v.get("pos").and_then(crate::cmd::parse_pos).unwrap_or_else(|| s.sel.focus.clone());
    match issue_at(s, &at) {
        Some((a, b, i)) => Ok(issue_json(s, &a, &b, &i)),
        None => Ok(Value::Null),
    }
}

/// A lookup result; a target remembers the document word independently of the query.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThesaurusLookup {
    pub word: String,
    pub synonyms: Vec<String>,
    pub target: Option<ThesaurusTarget>,
}

/// A replacement receipt. Re-derive it before editing so stale or forged ranges cannot replace
/// another word. The body epoch also rejects receipts from before Undo/Redo, which can restore a revision.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThesaurusTarget {
    document: u64,
    revision: u64,
    body_epoch: u64,
    selection: Selection,
    start: Pos,
    end: Pos,
    text: String,
}

impl ThesaurusTarget {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_current(&self, s: &Session) -> bool {
        word_target(s).as_ref() == Some(self)
    }
}

fn word_target(s: &Session) -> Option<ThesaurusTarget> {
    if s.column_segments().is_some() || s.cell_range().is_some() || s.math.is_some() || crate::cmd::objects::object_selection(s).is_some() {
        return None;
    }
    let (mut a, mut b) = s.sel.ordered();
    if a.story != b.story || a.path != b.path {
        return None;
    }
    let para = s.doc.para_at(&a)?;
    if s.sel.is_collapsed() {
        (a.off, b.off) = para.word_at(a.off);
    }
    let raw = para.text.get(a.off..b.off)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 256 || !trimmed.chars().all(wordcraft_doc::bidi::is_word_char) {
        return None;
    }
    a.off = a.off.checked_add(raw.len() - raw.trim_start().len())?;
    b.off = a.off.checked_add(trimmed.len())?;
    Some(ThesaurusTarget {
        document: s.document_id(),
        revision: s.rev(),
        body_epoch: s.ime_body_epoch(),
        selection: s.sel.clone(),
        start: a,
        end: b,
        text: trimmed.into(),
    })
}

/// A tiny built-in English thesaurus of common words (our own list).
fn synonyms(word: &str) -> Vec<String> {
    const T: &[(&str, &[&str])] = &[
        ("good", &["fine", "excellent", "decent", "solid", "worthy"]),
        ("bad", &["poor", "inferior", "faulty", "unpleasant"]),
        ("big", &["large", "huge", "sizable", "vast", "major"]),
        ("small", &["little", "tiny", "compact", "minor", "modest"]),
        ("fast", &["quick", "rapid", "swift", "speedy"]),
        ("slow", &["unhurried", "gradual", "leisurely", "sluggish"]),
        ("happy", &["glad", "cheerful", "content", "joyful", "pleased"]),
        ("sad", &["unhappy", "sorrowful", "downcast", "gloomy"]),
        ("important", &["significant", "key", "vital", "essential", "major"]),
        ("show", &["display", "reveal", "present", "demonstrate"]),
        ("make", &["create", "build", "produce", "craft", "form"]),
        ("use", &["employ", "apply", "utilize", "operate"]),
        ("help", &["assist", "aid", "support", "serve"]),
        ("start", &["begin", "launch", "open", "initiate"]),
        ("end", &["finish", "close", "conclude", "stop"]),
        ("say", &["state", "tell", "remark", "mention"]),
        ("think", &["believe", "consider", "reckon", "suppose"]),
        ("new", &["fresh", "novel", "recent", "modern"]),
        ("old", &["aged", "former", "ancient", "vintage"]),
        ("beautiful", &["lovely", "attractive", "gorgeous", "stunning"]),
        ("easy", &["simple", "effortless", "straightforward"]),
        ("hard", &["difficult", "tough", "demanding", "firm"]),
        ("work", &["labor", "effort", "job", "task", "operate"]),
        ("idea", &["concept", "notion", "thought", "plan"]),
        ("change", &["alter", "modify", "adjust", "shift"]),
        ("great", &["excellent", "superb", "terrific", "considerable"]),
        ("very", &["extremely", "highly", "truly", "really"]),
        ("shared", &["common", "joint", "communal", "mutual"]),
        ("community", &["group", "society", "collective", "circle"]),
    ];
    T.iter().find(|(w, _)| *w == word).map(|(_, list)| list.iter().map(|w| (*w).into()).collect()).unwrap_or_default()
}

/// Query without dispatch: drawing a pane never consumes undo-join or IME command state.
pub fn thesaurus_lookup(s: &Session, requested: Option<&str>) -> ThesaurusLookup {
    let target = word_target(s);
    // Bound direct callers too; a long query is an ordinary no-result lookup, not an allocation proportional to a document.
    let source = requested.or_else(|| target.as_ref().map(|t| t.text.as_str())).unwrap_or("").trim();
    let word = source.chars().take(256).collect::<String>().to_lowercase();
    let synonyms = if source.len() <= 256 { synonyms(&word) } else { Vec::new() };
    ThesaurusLookup { word, synonyms, target }
}

pub(super) fn thesaurus(s: &mut Session, v: &Value) -> CmdResult {
    if v.get("word").is_some_and(|v| !v.is_string()) {
        return Err(CmdError::Params("`word` must be text".into()));
    }
    if p::str(v, "word").is_some_and(|w| w.len() > 256) {
        return Err(CmdError::Params("the lookup word must be at most 256 bytes".into()));
    }
    Ok(json!(thesaurus_lookup(s, p::str(v, "word"))))
}

pub(super) fn synonym_replacement_enabled(s: &Session) -> Option<&'static str> {
    word_target(s).is_none().then_some("select a word in ordinary text")
}

pub(super) fn replace_synonym(s: &mut Session, v: &Value) -> CmdResult {
    let supplied = v.get("target").ok_or_else(|| CmdError::Params("`target` is required".into()))?;
    // Compare with a freshly derived bounded receipt instead of deserializing caller-supplied
    // paths or text. No caller-provided range becomes an editing target.
    let target = word_target(s).ok_or_else(|| CmdError::Params("select a single word to replace".into()))?;
    if supplied != &json!(&target) {
        return Err(CmdError::Params("the document or selection changed; look up the word again".into()));
    }
    apply_synonym(s, target, synonym_text(v)?)
}

/// Relative replay is a separate command: a stale receipt is never silently retargeted.
pub(super) fn replace_synonym_at_selection(s: &mut Session, v: &Value) -> CmdResult {
    if v.get("target").is_some() {
        return Err(CmdError::Params("use review.replaceSynonym to validate an explicit target".into()));
    }
    let target = word_target(s).ok_or_else(|| CmdError::Params("select a single word to replace".into()))?;
    apply_synonym(s, target, synonym_text(v)?)
}

fn synonym_text(v: &Value) -> Result<&str, CmdError> {
    let replacement = p::req_str(v, "text")?;
    let word = p::req_str(v, "word")?;
    if word.len() > 256 || !synonyms(&word.to_lowercase()).iter().any(|word| word == replacement) {
        return Err(CmdError::Params("`text` must be a synonym returned for this word".into()));
    }
    Ok(replacement)
}

fn apply_synonym(s: &mut Session, target: ThesaurusTarget, replacement: &str) -> CmdResult {
    s.sel = Selection { anchor: target.start, focus: target.end };
    crate::cmd::type_text(s, replacement)?;
    crate::cmd::sel_result(s)
}
