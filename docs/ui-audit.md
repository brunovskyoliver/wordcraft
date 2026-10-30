# UI audit and regression backlog

> Last reviewed: 2026-10-31 · Last updated: 2026-10-31 · Status: three repair cycles; cycle 03 adds geometry and inert-control sweeps; broader audit and parity gaps remain open. No completeness or readiness claim.

Task UI-AUDIT-001 resumes the comprehensive Word parity mission from baseline `aadb077e`.
Cycle 02 is tracked as UI-AUDIT-002, based on verified commit `9785d1cb`.
Cycle 03 is tracked as UI-AUDIT-003, based on `2a6421a2`.
Evidence lives in `~/agent-runs/2026-10-29-wordcraft-ui-audit/`.

## Acceptance and method

Audit → compare → reproduce → fix → build → inspect screenshots → test → verify related states.
A generated screenshot is not an inspected state. A passing engine test is not a verified UI control.
Record each finding with severity, reproduction, before/after evidence, root cause and regression.
Do not mark complete until every inventory entry has visual and behavioral evidence, critical/major
findings are resolved, and repository gates pass. Retain gaps and reference limitations explicitly.

## Environment and reference limits

Linux x86_64, Rust 1.99.0. The ignored plan directory and shared craftrules checkout are missing;
the public craftrules URL returned HTTP 404. Committed architecture, roadmap and parity docs
provide the fallback instructions. No Word installation or linked reference machine is available.
Recorded observations in ui-parity.md and typing-parity.md are secondary reference evidence only.
Public Microsoft font-size, Selection Pane, Thesaurus and Read Mode behavior is linked in the
evidence reference-notes.md; it does not establish precise visual geometry.
No Microsoft assets will be added. Word captures, if later available, belong only in ignored
`plan/word/screenshots/`. Optional craft-fonts and craft-i18n were absent from the initial build; those captures
use system fonts and English. Pinned-font and translated samples are now inspected in localized-scaling/ and wide/. Native
platform checks and broader release-input coverage remain outstanding.

## Representative state matrix

| Dimension | Required samples | Current evidence |
|---|---|---|
| Window | 800×600 compact; 1024×480 short; 1440×900 normal; 1920×1080 maximized; 2560×1080 ultrawide | Samples at compact, short, normal, maximized and ultrawide sizes inspected; component coverage partial |
| Scaling | 1×, 1.25×, 2×; interface 100%, 150%, 200%, Auto | 1×, 1.25×, 2× display captures; representative Auto/100%/150%/200% interface captures inspected |
| Appearance | Light, dark/light page, dark/dark page, system changes | Light and dark/light-page samples inspected; dark page/system transitions pending |
| Documents | Empty, short, long, sample, complex synthetic; dirty and protected | Sample and short synthetic editing/table/shape documents exercised; long/complex/protected pending |
| Selection | Caret, text range, mixed formatting, cells, image, shape, equation, chart | Text/caret and table/shape contextual transitions tested; remaining selections pending |
| Transient UI | Every dialog/tab, menu, gallery, tooltip; open/closed and nested | Pending |
| Interaction | Pointer, keyboard, programmatic; apply/cancel/error; undo/redo; focus restore | Pending |
| Content | Long names/labels, locales, RTL, CJK, unsupported commands | German/Polish/Japanese samples inspected with pinned inputs; mixed labels remain; RTL/long-content coverage pending |
| Layout | Zoom 50/100/200%; print/web/draft/read; pane combinations | 50/100% print and Read Mode samples with panes inspected; 200% document zoom and other layouts pending |

## Component inventory

Each row starts uninspected. Split into named scenarios as evidence is collected. Contextual tabs
require their matching selection; forcing a tab name alone does not verify contextual behavior.

| Component | Visual evidence | Behavior / regression evidence | Status |
|---|---|---|---|
| Home | before/compact-home.png inspected | Active tab asserted; control actions pending | Partial |
| Insert | before/compact-insert.png inspected | Active tab asserted; control actions pending | Partial |
| Draw | before/compact-draw.png inspected | Active tab asserted; control actions pending | Partial |
| Design | before/compact-design.png inspected | Active tab asserted; control actions pending | Partial |
| Layout | before/compact-layout.png inspected | Active tab asserted; control actions pending | Partial |
| References | before/compact-references.png inspected | Active tab asserted; control actions pending | Partial |
| Mailings | before/compact-mailings.png inspected | Active tab asserted; control actions pending | Partial |
| Review | before/compact-review.png inspected | Active tab asserted; control actions pending | Partial |
| View | before/compact-view.png inspected | Active tab asserted; control actions pending | Partial |
| Developer | before/compact-developer.png inspected | Active tab asserted; control actions pending | Partial |
| Zotero | None | None | Pending |
| Help | None | None | Pending |
| Table Design | contextual/ compact and normal captures inspected | Appears on matching selection; body selection restores Home | Partial |
| Table Layout | contextual/ compact and normal captures inspected | Appears on matching selection; body selection restores Home | Partial |
| Picture Format | None | None | Pending |
| Shape Format | contextual/ compact and normal captures inspected | Appears on matching selection; body selection restores Home | Partial |
| Equation | None | None | Pending |
| Chart Design | dialog-families/ inserted-object compact capture inspected | Actual insertion, undo and redo tested; enclosed controls pending | Partial |
| SmartArt Design | dialog-families/ inserted-object compact capture inspected | Actual insertion, undo and redo tested; enclosed controls pending | Partial |
| Title bar / Quick Access Toolbar / search | Cycle-02 Search and compact customization captures inspected | Canonical Search dispatch/focus and hidden helper filtering verified; QAT command actions have regressions; remaining title-bar/menu states pending | Partial |
| Ribbon tab overflow / folded groups / launchers | contextual/ menus and overflow inspected | Review/References chevrons and folded menus exercised; individual commands pending | Partial |
| Status bar / language / view / zoom | None | None | Pending |
| Canvas / pages / margins / rulers / scrollbars | None | None | Pending |
| Selection / caret / IME / clipboard | after/editing captures | Modal input/selection, bold undo/redo and synthetic IME guard tested; native IME/clipboard pending | Partial |
| Tables / cell selection / handles | None | None | Pending |
| Pictures / shapes / crop / rotation / wrapping | None | None | Pending |
| Headers / footers / notes / fields | None | None | Pending |
| Navigation / Styles / Inspector / Clipboard panes | extended/ Navigation+Comments light/dark inspected | Pane combinations shown; individual controls pending | Partial |
| Comments / Reviewing / Restrict Editing panes | None | None | Pending |
| Accessibility / Alt Text / Format Shape panes | None | None | Pending |
| Selection Pane | Cycle-02 empty/object/deletion captures at four profiles inspected | Query-only behavior, selection, deletion/reindex, undo and close replayed; body objects only; other stories/hide/rename gaps remain | Partial |
| Thesaurus | Cycle-02 results/no-result/stale-target and Focus/Full Screen restoration captures inspected | Lookup focus, insertion, repeat/undo and selection preservation replayed; small English dictionary, language/depth parity open | Partial |
| Backstage Home / New / Open / recent / recovery | Built-in/personal gallery wrapping, long-name tooltips, gutters and short sidebar inspected in cycle 02 | Personal card activation/reachability regressions; page commands preserve clean state; Open/recent/recovery actions remain partial | Partial |
| Backstage Info / Save As / Export / Share / Print | Cycle-02 Print compact/short/dark/scaled/localized captures inspected | Footer geometry, preview cache document switching and settings actions have regressions; native printing/services and Share remain pending | Partial |
| Options / customization / keyboard / language | All eight categories plus General/Advanced/Accessibility boundaries and lower customization controls inspected | Responsive settings/actions, accessibility names and command filtering regressions; full customization operations/native key delivery remain partial | Partial |
| Text / object / table / field context menus | None | None | Pending |
| Color / font / style / table / symbol galleries | None | None | Pending |
| Focus / Full Screen / Read / Draft / Web / Outline views | Read compact light/dark and Outlining inspected | Read exit, column/size/spacing/color and Navigation actions tested; other modes pending | Partial |
| Native file dialogs / multiwindow / close / recovery | None | None | Pending |
| Keyboard traversal / KeyTips / hover / focus / disabled states | Cycle-02 validation, pane focus, Read Aloud and disabled stale-target captures inspected | Modal pointer/keyboard/wheel/focus and transport keyboard regressions; exhaustive KeyTips/hover/menu coverage pending | Partial |
| Screen reader / contrast / reduced motion | Validation colors and named control states inspected | Error color contrast and Read Aloud/QAT accessible-name regressions; native screen readers and whole-interface contrast/reduced-motion review pending | Partial |
| Colors modal / Standard and Custom tabs | Custom inspected, compact light and normal dark | Actions and Standard tab pending | Partial |
| Read Aloud player | Fresh cycle-02 compact/light and normal/dark focus, pause, next and stop captures inspected | Named transport controls, keyboard activation and disabled-state regressions; silent backend, audible/native output pending | Partial |
| Update window / Zotero alerts | None | Modal input ownership covered in code; full window states pending | Partial |
| Content-control popups / split view / Outlining | None | None | Pending |

## Dialog inventory

Enumerated from the dialog hosts, including nested insert, reference, review and layout forms.
Individual tabs, validation and cancel/apply states must be split during execution.

| Dialog title | Visual | Apply / cancel / focus / validation |
|---|---|---|
| About WordCraft | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| AutoCorrect | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Bookmark | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Borders and Shading | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Caption | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Caption Numbering | Pending | Pending |
| Cell Margins | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Change Chart Type | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Check for Errors | Pending | Pending |
| Columns | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Combine Documents | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Compare Documents | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Content Control Properties | Pending | Pending |
| Convert Text to Table | Pending | Pending |
| Convert to Text | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Create New Style | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Create Source | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Cross-reference | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Custom Document Properties | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Custom Watermark | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Customize Keyboard | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Customize Ribbon | Cycle-02 compact/150% stacked layout and lower controls inspected | Visibility names and customization actions have regressions; lower controls reachable; full operation/localization coverage partial |
| Define New Multilevel List | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Discard recovery copy | Pending | Pending |
| Drop Cap | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Edit Data | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Edit Field Code | Pending | Pending |
| Edit Source | Pending | Pending |
| Enclose Characters | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Encrypt with Password | Pending | Pending |
| Envelopes | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Field | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Find Recipient | Pending | Pending |
| Find and Replace | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Finish & Merge | Pending | Pending |
| Font | Cycle-02 normal/compact/wide/light/dark/scaled invalid/corrected captures inspected | Invalid drafts retain values/focus; half-point sizes, mutual exclusion and undo replayed; Advanced hierarchy and remaining controls partial |
| Footnote and Endnote | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Formula | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Hyphenation | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Icons | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| If…Then…Else | Pending | Pending |
| Index | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Insert Chart | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Insert Hyperlink | Cycle-02 invalid/corrected captures at four profiles inspected | Draft retention, URL apply, selection and undo replayed; stale status and selected-label/current-address editing remain open |
| Insert Merge Field | Pending | Pending |
| Insert SmartArt | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Insert Table | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Labels | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Language | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Line Numbers | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Manage Styles | after/manage-styles-final.png inspected | Intrinsic footer geometry regression passed; style actions pending |
| Manual Hyphenation | Pending | Pending |
| Mark Index Entry | dialog-families/ compact initial state inspected | Modeless selection and Escape behavior tested; Apply pending |
| Match Fields | Pending | Pending |
| Modify Style | Pending | Pending |
| Modify Table Style | Fresh compact/light and normal/dark preview/apply/reopen/undo captures inspected | Styles without a base apply correctly; bold preview/persistence/undo verified; other style parts/fields remain partial |
| New Label | Pending | Pending |
| New Table Style | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Organizer | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Page Number Format | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Page Setup | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Paragraph | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Password | Pending | Pending |
| Paste Special | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Phonetic Guide | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Recipient List | Pending | Pending |
| Rename Document | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Search Commands | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Select Table | Pending | Pending |
| Size | Pending | Pending |
| Skip Record If | Pending | Pending |
| Sort | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Source Manager | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Split Cells | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Symbol | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Table Properties | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Table of Contents | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Table of Contents Options | Pending | Pending |
| Table of Contents Styles | Pending | Pending |
| Table of Figures | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Tabs | Pending | Pending |
| Templates and Add-ins | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Track Changes Options | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Word Count | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| WordCraft | Pending | Pending |
| Zoom | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |

## Defect backlog

| ID | Severity | Finding / reproduction | Status |
|---|---|---|---|
| AUD-001 | Major verification gap | ui_shot prints command errors or rendering failures but can exit successfully; scripts cannot assert resulting UI state. A failed tab selection may leave captures of the previous tab. | Verified: 5 harness tests and full CI pass |
| AUD-002 | Verification gap | ui_shot hardcodes 2× scaling, preventing native/fractional DPI scenarios. | Verified: 5 harness tests and full CI pass |
| AUD-004 | Major | At 800 pt / 1.25× the Read Mode top exit overlaps Navigation and is clipped. Evidence: before/read-compact.png. Root cause: all settings in a non-wrapping row. Bottom exit still works. | Fixed in tested widths/themes; exit geometry and color action regressions pass |
| AUD-005 | Major | Define New Multilevel List at 1024×480 / 2× loses title and footer beyond viewport. Evidence: before/1024x480-dialog-defineList.png. Shared window has no vertical scroll area. | Fixed; final titles/footers inspected, scrolling and Cancel regressions pass |
| AUD-006 | Major behavior | Actual Font UI accepts size 0 into document properties; malformed size abc is ignored while other changes apply and dialog closes. | Invalid → corrected → undo scenarios pass at four profiles; all Font captures inspected. Verified within the tested scope; all eight CI gates pass. |
| AUD-007 | Critical interaction | Font open on selected text still permits canvas typing; Escape cancels the dialog and collapses the document selection. Baseline control replays confirm both. | Fixed; modal/modeless keyboard, selection and synthetic IME regressions pass |
| AUD-008 | Minor visual | Zoom dialog slider rail is white on white; only thumb/value visible. The same low-contrast rail appears in Accessibility Read Aloud speed. | Fixed in cycle 03: shared slider helper paints the rail in the strong border color at all seven sites; status bar captures inspected |
| AUD-009 | Major behavior | Selecting Subscript on superscript text leaves both boxes checked; OK keeps superscript. | Mutual-exclusion/undo UI test and screenshot replay pass. Verified within the tested scope; all eight CI gates pass. |
| AUD-010 | Major interaction | Actual Selection Pane and Thesaurus clicks show no result, while pure command probes return objects and synonyms. Evidence: control-results/README.md. | Selection and Thesaurus panes implemented. Twenty-one behavior scenarios include query-only, focus, insert, delete, repeat, undo, stale targets and empty states; their 60 images inspected. Verified within the tested scope; all eight CI gates pass. |
| AUD-011 | Accessibility | New tall-dialog regression found six controls without accessible names in Define New Multilevel List. | Fixed; labels bound in List, Insert Table, Zoom and Manage Styles; accessibility regressions pass |
| AUD-012 | Major visual | Quick Access Toolbar customization at 800×600 clips both sides, including Close and reorder buttons. | Responsive layouts and QAT actions repaired; compact/scaled captures and lower-control scrolling inspected. Verified within the tested scope; all eight CI gates pass. |
| AUD-013 | Visual regression | Manage Styles expands vertically with a centered Close row and excess blank space. | Fixed; intrinsic footer regression passed and final capture inspected |
| AUD-014 | Major behavior | Empty Hyperlink Address closes the dialog with an error and loses the draft display text. | Invalid Address draft/selection retained; correction and undo pass at four profiles. AUD-032/034 remain separate gaps. Verified within the tested scope; all eight CI gates pass. |
| AUD-015 | Minor visual | Folded Table Cell Size fields lack visible input boundaries. | Fixed in cycle 03: a folded group restores ribbon widget visuals instead of the borderless menu style |
| AUD-016 | Major visual | Compact Print preview footer overlaps page navigation and zoom controls. | Responsive compact/dark/localized Print captures inspected with separate navigation and zoom rows. Verified within the tested scope; all eight CI gates pass. |
| AUD-017 | Major visual | Compact Options clips fields, Browse buttons, dropdowns and explanatory text across seven categories. | Responsive fields/categories and Advanced boundary repaired; General and Accessibility follow-ups AUD-036/037 now pass regressions and final-source visual review. Verified within the tested scope; all eight CI gates pass. |
| AUD-018 | Major visual | Compact Home/New crop template cards beyond the right edge without horizontal scrolling. | Built-in and personal cards wrap with bounded captions/tooltips. Compact, dark, scaled and fractional-input captures inspected; card-opening and tooltip geometry regressions included. Verified within the tested scope; all eight CI gates pass. |
| AUD-019 | Behavior gap | Undo restores original inspected content but retains dirty:true, causing unnecessary save prompts. Session::undo always calls touch(). | Fixed in cycle 03: Undo/Redo compare with the saved content; saves move the savepoint and password/recovery/failed-save paths invalidate it; engine regressions pass |
| AUD-020 | Major regression caught in review | Direct Go To leaves its initial-focus flag set, blocking document keyboard selection; Page also repeatedly steals focus. | Runtime reproduced; one-shot focus correction added; Verified: Page/Bookmark and IME regressions, inspected after replay, full CI pass |
| AUD-021 | Major visual | At 1024×480 File sidebar clips Options/About and overlaps Close with the community link. | Short sidebar captures expose Options, community and About on separate rows; reachability regression included. Verified within the tested scope; all eight CI gates pass. |
| AUD-022 | Parity gap | Table Properties is a stacked form; Word documents Table/Row/Column/Cell/Alt Text tabs and additional options. | Public reference comparison recorded; current controls fit but hierarchy/depth parity remains open |
| AUD-023 | Major behavior | Modify Table Style for a style without a base has a blank preview and rejects OK because UI sends basedOn as an empty string. | Fresh compact/light and normal/dark preview, apply, reopen and undo replays inspected; empty base omitted from command parameters. Verified within the tested scope; all eight CI gates pass. |
| AUD-024 | Parity gap | Font puts spacing/kerning in one compact form; Word documents an Advanced tab and additional typography controls. | Public reference supports missing hierarchy; full Font layout comparison remains pending |
| AUD-025 | Major interaction | With Font open, a settled actual ribbon Bold click still changes selected document text formatting behind the dialog. | Shared modal input ownership implemented; actual pointer, wheel, drag, nested-window and pane focus regressions included. Verified within the tested scope; all eight CI gates pass. |
| AUD-026 | Major correctness | Print layout and page-texture cache keys omit document identity, so equal revisions in different documents can reuse stale preview data. | Document identity added to layout and texture cache keys; real document-switch regression included. Verified within the tested scope; all eight CI gates pass. |
| AUD-027 | Major layout | Very short Print content at enlarged interface scale allocates less height than its two-row footer, overlapping the settings region. | Bounded inner scroll areas and outer scrolling expose the short high-scale Print footer without overlap in inspected captures. Verified within the tested scope; all eight CI gates pass. |
| AUD-028 | Major accessibility | Read Aloud transport icons draw and respond to pointer input but expose no named button widget information to the accessibility tree. | Named transport buttons and keyboard/disabled-state regressions implemented; fresh keyboard focus, pause, next and stop replays inspected in both profiles. Audible output untested. Verified within the tested scope; all eight CI gates pass. |
| AUD-029 | Major accessibility | Customize Ribbon visibility checkboxes have no accessible name; initial integration tests identify 12 unnamed controls. | Each Ribbon visibility checkbox now has its row label as an accessible name; regressions included. Verified within the tested scope; all eight CI gates pass. |
| AUD-030 | Accessibility | Font/Hyperlink validation text inherits default pure red, below the 4.5 contrast target on light and dark dialog backgrounds. | Theme error color applied with contrast regression; invalid Font/Hyperlink captures inspected in light/dark profiles. Verified within the tested scope; all eight CI gates pass. |
| AUD-031 | Minor visual | Invalid Font size 0 shrinks the sample to its 6-point minimum while the draft error remains open. | Fixed in cycle 03: an invalid draft keeps the last valid sample size |
| AUD-032 | Minor interaction | A corrected successful Hyperlink still shows its earlier error on the status bar until the existing 6-second expiry. | Fixed in cycle 03 for Hyperlink: the dialog alone shows a refusal; other dialogs not audited for this pattern |
| AUD-033 | Minor visual | Floating Backstage scrollbar covers the rightmost template edge; different caption heights vertically offset card tops. | Reserved scrollbar gutter and top-aligned gallery rows inspected across built-in/personal galleries. Verified within the tested scope; all eight CI gates pass. |
| AUD-034 | Major behavior | Hyperlink ignores an edited display label when text is selected; reopening initializes Address to https:// instead of the current link. | Fixed in cycle 03: staged patch reviewed, merged and compiled; label replacement, current-address preload, stale-target refusal, protection, tracking and replay regressions pass (11 engine, 2 UI) |
| AUD-035 | Major interaction | Search Commands and customization offer receipt/replay synonym helpers as standalone actions; Enter invokes them without required parameters and reports an error. | Automation-only metadata hides receipt/replay and raw pane-visibility helpers from Search/customization. Final-source Search shows only canonical Thesaurus and opens its focused query; registry/picker regression included. Verified within the tested scope; all eight CI gates pass. |
| AUD-036 | Minor visual | General Options wraps User name and Initials into fragments at 800 and 1440 widths despite available space. | Measured label/field rows replace the default Grid sizing. Regression passes; final compact/normal/scaled/localized captures show whole labels. Verified within the tested scope; all eight CI gates pass. |
| AUD-037 | Major layout | Accessibility Options speed slider and numeric value widen the 640/150% column, clipping the following Skip citations checkbox too. | Slider reserves measured numeric width and stacks its label when needed. Boundary geometry/checkbox action regression passes; final 640/150% captures fit. Verified within the tested scope; all eight CI gates pass. |
| AUD-038 | Major behavior | Inert-control sweep: Reviewing Pane and Show Notes ran data queries with no visible result. Next/Previous Comment and Change, Next Footnote and the Update Table family silently did nothing when there was nothing to act on. | Fixed: real Reviewing Pane (list, row selects, Accept/Reject); Show Notes moves to the first note or reports none; Update Table reports when it finds no table of contents; the others are disabled with a reason. Regressions and capture inspected |
| AUD-039 | Major behavior | View › Split and Ctrl+Alt+S sent a UI request no front end handles, so they did nothing. | Disabled with a reason so it no longer deceives. A real split window remains an open parity gap |
| AUD-040 | Major accessibility | Geometry sweep and the test harness found about 60 fields, drop-downs and spin buttons without accessible names across dialogs, panes and File › New. | Fixed: labels bound or names added; 0 unnamed in all 220 sweep runs; regression covers 13 dialogs |
| AUD-041 | Minor interaction | About, Word Count, Check for Errors and Match Fields offered Close and Cancel doing the same thing; Manual Hyphenation left-aligned its Close. | Fixed: one shared right-aligned Close; regression passes |
| AUD-042 | Minor visual | Compact Review/References/Layout folded groups all drew the same dots icon. | Fixed: each mapped group draws its own original icon; compact captures inspected; unmapped groups still use the fallback |
| AUD-043 | Minor visual | Shape Format size used points while Picture Format used the document unit. | Fixed: both follow the unit in Options; inches shown in capture |
| AUD-044 | Minor visual | Outlining: Show Level touches its combo, and the two checkboxes touch. | Spacing added; final capture not separately re-inspected |
| AUD-045 | Minor visual | Find and Replace action buttons are left-aligned; Word's sit on the right. Read Mode shows Edit Document twice. Recipient List has no visible horizontal scroll indicator at 800 wide. | Observed in cycle 03 sweep; open |
| AUD-003 | Audit gap | ui-parity.md duplicated modal-dialog, keyboard and pane rows and falsely claimed Thesaurus presentation. | Those rows consolidated; Linux registry remeasured at 163 commands/192 shortcuts; old aggregate estimate marked historical. Other historical rows still require full state review |

## Verification record

### Cycle 03

`cargo xtask ci` passed all eight gates: 4,718 tests passed, none failed, four existing ignores.
`cargo xtask parity` is unchanged at 505/520 registered features, which does not measure behavior.

`ui_shot` gained `clickLabel` (click a control by accessible name) and `audit` (controls leaving
the window, overlapping or unnamed, with an optional strict failure). A generated sweep ran 110
scenes in two profiles (1280×800 light, 800×600 dark): 220 runs, about 15,000 audited controls.
All ribbon tabs including Zotero, Help and contextual tabs, view modes, zoom 50/200/500%, panes,
context menus, 57 dialogs and 10 File pages ran. Contact sheets were reviewed by eye: about 40
normal-profile scenes and 33 dark compact scenes, not all 222 individually. The audit cannot see
scroll clipping, so its offWindow and overlap lists include scrolled-list false positives,
triaged by hand; those counts are not a defect count.

A second sweep clicked each enabled control once in a fresh app (841 listed; chrome and repeats
deduplicated) and compared before and after pixels. Unchanged controls were triaged by hand. This
detects no visible effect only: it does not prove a control's effect is correct.

CI results and the commit are recorded in `cycle-03/final-report.md` in the evidence directory.

### Cycle 02

The final source replay passed 36 scenarios and 442 reply assertions; all 148 final captures
were individually inspected. Earlier responsive evidence covers 23 scenarios and 114 captures,
including Print, galleries and personal tooltips. Reports identify each source/binary snapshot;
they must not be combined into a claim that every state ran on one binary. The final source
changes did not alter those Print/gallery implementations. The later native Print correction
changed only an integration test: scroll to Grayscale, prove the whole control is inside its
paint clip, then click and retain the original setting/readiness assertions. The diagnostic
confirmed that the previous test clicked below the settings viewport. Production source and
the screenshot runner remained unchanged.

`cargo xtask ci` passed all eight gates: formatting, Clippy with warnings denied, workspace
tests, the separate translation audit, doctest build, asset attribution, layering and WASM.
Workspace tests: **4,695 passed, zero failed, four ignored**. The UI library contributed
1,060 passing tests, two ignores and one translation-audit filter; the engine passed all 1,087.
The filtered translation audit then passed separately. Missing translations are informational
in that gate and remain a parity gap. The doctest gate found no doctest cases. WASM checking
passed for all 16 selected crates. `cargo xtask parity` also passed and regenerated the
unchanged 505/520 registration checklist.

The four existing ignores cover a recovery-size measurement, a locally licensed KaiU font,
the mini-toolbar anchor conflict and a large-table benchmark. They are not new passing coverage.
The replay used a silent speech backend; audible output and native GUI/platform integration
remain untested. The repository contains 46 portable cycle-02 scenarios with 493 assertions
and 184 screenshot checkpoints; personal-template fixture scenarios remain in external evidence.
The exported scripts match their evidence copies apart from relative screenshot destinations.


The initial build passed in 25m12s. Ten compact ribbon captures and ten short-dialog captures
were inspected. A first scrolling implementation fixed clipping but introduced excessive blank
space; after-image review caught it. The final layout uses a bounded, content-sized scroll area.
Six scrolled footer images and three actual pointer Cancel checks verified reachability and no
unintended document edits. Detailed reproduction and review notes remain in the evidence directory.

Final iteration tests passed: 995 UI tests, 5 ui_shot tests and 46 Zotero tests, with two existing
UI tests ignored and the translation audit reserved for CI. The subsequent Manage Styles repair
passed its focused geometry regression and final screenshot review. Editing and modal-input
JSONL replays preserve text, selection, formatting and undo/redo behavior.

Full normal-profile `cargo xtask ci` passed all eight gates with pinned optional inputs:
formatting, Clippy, workspace tests, translation audit, doctests, assets, layers and WASM.
The workspace run passed 4,617 tests; four existing tests were ignored and the translation audit
was run separately. The UI suite passed 996 tests. WASM checked all 16 supported crates.
`first-cycle-ci-summary.json` and `ci.log` retain the results. Translation gaps reported by the
audit are not failing gates and still require follow-up.

Two early attempts ended with SIGTERM. A third was stopped to apply the Go To review fix.
The fourth caught an older IME test expecting Go To to reclaim focus after Tab. That test now
checks an explicit Tab/Shift+Tab round trip while retaining composition-reset assertions; it
passed individually and in the fifth full CI run. No test was disabled to obtain the pass.
No push or deployment has occurred.

Tests exposed a separate Linux Zotero ownership race: before accept, the connected server socket
can have placeholder uid/inode zero. The repair resolves only that placeholder through the
matching verified listener, refusing missing, ambiguous or foreign ownership. Nineteen peer
checks and three UI integrations passed. See zotero-test-review.md and peer-accept evidence.

Independent first-cycle review found a Go To focus latch introduced by the new input guard.
A real Bookmark-target/document-click/Shift+Right replay confirmed it. Go To now consumes initial
focus once, requests it when the target/tab changes, and permits document editing while open.
The new Page/Bookmark regression passed against the normal-profile test binary. The after
replay also passed for both targets; both PNGs were inspected and Escape preserved the selected
text. The screenshot runner now includes the pinned optional inputs. Full CI passed.

## Evidence index

All paths below are relative to `~/agent-runs/2026-10-29-wordcraft-ui-audit/`.

| Evidence | Scope and limits |
|---|---|
| visual-review.md; before/; after/ | Initial/final ribbon, dialog, Read Mode captures; footer pointer checks |
| dialog-behavior-review.md | Modal keyboard leak, Escape selection and modeless exclusions |
| extended/README.md | Six inspected Colors Custom, Outlining and Navigation+Comments states |
| contextual/report.md | 25 inspected table/shape contextual and folded-menu captures; command execution partial |
| next-dialogs/README.md | 23 inspected additional dialog states; QAT clipping and Manage Styles finding |
| font-validation/README.md; results.json | Actual invalid Font and Hyperlink UI reproductions |
| control-results/README.md | Actual inert Selection Pane and Thesaurus clicks versus engine results |
| verification-final.log | Combined UI, harness and Zotero iteration tests |
| manage-styles-regression.log | Final Close-row geometry regression |
| backstage/README.md | 20 inspected Backstage/Options captures; compact Print, Options and template-gallery defects |
| dialog-families/README.md | Compact insertion/reference dialogs, Chart/SmartArt actual insertion and undo; dirty-state gap |
| read-mode-interactions/report.md | Actual menu settings and exit; light/dark, content/selection/focus preserved |
| first-cycle-review.md | Independent source review; Go To finding reproduced and corrected before commit |
| localized-scaling/report.md | 16 German/Polish/Japanese captures; 150% interface and fractional DPI; partial controls |
| wide/README.md | Maximized/ultrawide, Auto/200% interface, Navigation+Styles, 50/100% document zoom |
| registry-measurement.json; cycle-02/registry-measurement.json | Baseline and current Linux command/shortcut counts; no claim that every keyboard path works |
| reference-notes.md | Primary public Word behavior references; no live comparison or copied assets |
| first-cycle-ci.log; first-cycle-ci.exit | Archived successful first-cycle full gates |
| cycle-02/sol-final-behavior.md; sol-final-behavior-index.json | 21 scenarios / 324 assertions / 60 inspected images on the pre-Clippy-refactor runner; 18 prior interaction images also reviewed |
| cycle-02/sol-final-responsive.md; sol-responsive-inspected-index.json | 23 scenarios / 157 assertions / 114 inspected images; General/Accessibility follow-ups AUD-036/037 |
| cycle-02/final-source-replay.md; final-source-replay-index.json | Final-source replay: 36 scenarios, 442 assertions, 148 individually inspected PNGs; calibrated wheel probes archived |
| cycle-02/progress.md; final-source-inventory.json | Integration and verification history; source hashes include untracked Rust modules |
| cycle-02/ci.log; ci.exit; parity.log; parity.exit | Final cycle-02 gates; success must be established from completed results, not job start |

Reusable strict JSONL scenarios live in `crates/ui-egui/tests/scenarios/`, including 46 cycle-02
scenarios with 493 reply assertions and 184 capture checkpoints. These counts describe the
reusable set, not one final-source execution; personal-template fixtures stay in external evidence. The harness now fails
on unexpected command errors, missing responses and rendering errors, supports state assertions,
and bounds dimensions, DPI, waits and frame counts.

The cycle-02 Linux runtime lists 655 commands (including available print commands), of which
163 have 192 default shortcut combinations. The runtime catalog reports 505/520 registered
feature entries (97.1%). This measures command
registration, not complete behavior or visual parity. The previously inert pane controls demonstrated why
that metric cannot replace this state and interaction audit.

## Acceptance checkpoint

This checkpoint covers the user's full mission, not just the second repair cycle. A pause after
cycle 02 does not mean the application meets the definition of DONE.

| Requirement | Evidence and remaining work |
|---|---|
| Every inventoried component and meaningful state inspected | Not met: pending/partial rows remain in both inventories. |
| All discovered critical/major defects resolved | Not met: Hyperlink display-label/current-address editing remains open (AUD-034). |
| No accidental clipping/overlap in tested scenarios | Final-source replay inspected 148 captures without a new layout defect; earlier snapshots and untested states have separate coverage limits. |
| Proportions and hierarchy consistently match Word | Not established: no direct Word environment; Font/Table Properties hierarchy gaps remain. |
| All exposed controls have verified behavior | Not met: representative actions are tested; many dialog fields, menu contents and native paths remain unverified. |
| Core editing workflows pass end to end | Selected formatting, link, object, synonym and table-style workflows pass; comprehensive long/complex-document workflow coverage remains open. |
| Responsive layouts, themes, scaling and contextual states validated | Representative Linux headless profiles pass; complete combinations and native platforms remain open. |
| Regression tests and repository quality gates pass | All eight cycle-02 CI gates pass; 4,695 workspace tests pass with four existing ignores. This verifies the patch, not whole-app parity. |
| Remaining parity gaps documented with evidence | Known findings and uninspected areas remain in this backlog and the linked evidence reports. |

## Exact continuation

The mission remains incomplete. Nothing was pushed or deployed. Next:

1. Implement a real split window, or keep View › Split disabled (AUD-039).
2. Resolve AUD-022/024 (Table Properties and Font hierarchy) and AUD-045.
3. Extend the inert-control sweep into dropdown menu contents, galleries, Backstage and dialog
   controls; it currently clicks only controls visible without opening anything.
4. Add scroll clipping to the `audit` data so scrolled lists stop producing false positives.
5. Continue pending states: long, complex and protected documents, RTL/CJK, localized sweeps,
   hover and keyboard traversal, native accessibility and performance.
6. Obtain direct black-box Word and native platform evidence when an environment exists. None of
   the visual comparisons so far used live Word. Keep captures only in ignored
   `plan/word/screenshots/`.

Keep optional inputs pinned to craft-fonts `b076417b5880bd4154bd22f00695b1d28c6bea83` and
craft-i18n `9205108cfdced5177706264038848b68b2422d2b` under ignored `target/audit-inputs/`.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-29 | Initial and first-cycle review | Inventory, strict screenshot harness, responsive Read Mode/dialog repairs, input isolation, accessibility labels, Zotero race regression, wider audit and open defect backlog; all eight CI gates passed; broader mission remains open |
| 2026-10-29 | Second-cycle integration | Integrated staged repairs; initial compile passed; added source/geometry findings AUD-026/027; runtime and visual verification pending |
| 2026-10-29 | UI-AUDIT-002 closeout | Validation, task panes, modal ownership, responsive controls, accessibility and command filtering repaired; 36 final replay scenarios and all eight CI gates pass; remaining gaps retained; stop after cycle 02 at owner request |
| 2026-10-30 | UI-AUDIT-003 | Hyperlink editing, savepoint-aware dirty state, Reviewing Pane, disabled no-op commands, accessible names, minor visual fixes; geometry and inert-control sweeps added; remaining gaps retained |
