# UI audit and regression backlog

> Last reviewed: 2026-10-29 · Last updated: 2026-10-29 · Status: first repair cycle verified; broader audit and major defects remain open. No completeness or readiness claim.

Task UI-AUDIT-001 resumes the comprehensive Word parity mission from baseline `aadb077e`.
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
| Title bar / Quick Access Toolbar / search | None | None | Pending |
| Ribbon tab overflow / folded groups / launchers | contextual/ menus and overflow inspected | Review/References chevrons and folded menus exercised; individual commands pending | Partial |
| Status bar / language / view / zoom | None | None | Pending |
| Canvas / pages / margins / rulers / scrollbars | None | None | Pending |
| Selection / caret / IME / clipboard | after/editing captures | Modal input/selection, bold undo/redo and synthetic IME guard tested; native IME/clipboard pending | Partial |
| Tables / cell selection / handles | None | None | Pending |
| Pictures / shapes / crop / rotation / wrapping | None | None | Pending |
| Headers / footers / notes / fields | None | None | Pending |
| Navigation / Styles / Inspector / Clipboard panes | extended/ Navigation+Comments light/dark inspected | Pane combinations shown; individual controls pending | Partial |
| Comments / Reviewing / Restrict Editing panes | None | None | Pending |
| Accessibility / Alt Text / Format Shape / Selection panes | None | None | Pending |
| Backstage Home / New / Open / recent / recovery | backstage/ compact light and selected normal dark inspected | Page commands preserve clean state; card/actions pending | Partial |
| Backstage Info / Save As / Export / Share / Print | backstage/ compact light and selected normal dark inspected | Page commands preserve clean state; native services absent; Share pending | Partial |
| Options / customization / keyboard / language | All eight Options categories at compact width; customization dialogs inspected | Major compact clipping; settings semantics/persistence pending | Partial |
| Text / object / table / field context menus | None | None | Pending |
| Color / font / style / table / symbol galleries | None | None | Pending |
| Focus / Full Screen / Read / Draft / Web / Outline views | Read compact light/dark and Outlining inspected | Read exit, column/size/spacing/color and Navigation actions tested; other modes pending | Partial |
| Native file dialogs / multiwindow / close / recovery | None | None | Pending |
| Keyboard traversal / KeyTips / hover / focus / disabled states | None | None | Pending |
| Screen reader / contrast / reduced motion | None | None | Pending |
| Colors modal / Standard and Custom tabs | Custom inspected, compact light and normal dark | Actions and Standard tab pending | Partial |
| Read Aloud player / update window / Zotero alerts | None | None | Pending |
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
| Customize Ribbon | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
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
| Font | before/ and final after/ short-window captures inspected | Title/footer geometry checked; validation and command coverage partial |
| Footnote and Endnote | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
| Formula | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
| Hyphenation | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| Icons | dialog-families/ compact initial state inspected | Open/close keeps document clean; nested states/actions pending |
| If…Then…Else | Pending | Pending |
| Index | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Insert Chart | dialog-families/ compact initial state inspected | Open/close checked; detailed validation/apply paths pending |
| Insert Hyperlink | next-dialogs/ initial state inspected | Programmatic open/close leaves document clean; actual control coverage pending |
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
| Modify Table Style | dialog-families/ compact table state inspected | Open/close preserves inspected document; detailed apply/validation pending |
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
| AUD-006 | Major behavior | Actual Font UI accepts size 0 into document properties; malformed size abc is ignored while other changes apply and dialog closes. | Runtime reproduced; open |
| AUD-007 | Critical interaction | Font open on selected text still permits canvas typing; Escape cancels the dialog and collapses the document selection. Baseline control replays confirm both. | Fixed; modal/modeless keyboard, selection and synthetic IME regressions pass |
| AUD-008 | Minor visual | Zoom dialog slider rail is white on white; only thumb/value visible. | Confirmed by visual subagent; open |
| AUD-009 | Major behavior | Selecting Subscript on superscript text leaves both boxes checked; OK keeps superscript. | Runtime reproduced; open |
| AUD-010 | Major interaction | Actual Selection Pane and Thesaurus clicks show no result, while pure command probes return objects and synonyms. Evidence: control-results/README.md. | Reproduced; open |
| AUD-011 | Accessibility | New tall-dialog regression found six controls without accessible names in Define New Multilevel List. | Fixed; labels bound in List, Insert Table, Zoom and Manage Styles; accessibility regressions pass |
| AUD-012 | Major visual | Quick Access Toolbar customization at 800×600 clips both sides, including Close and reorder buttons. | Runtime reproduced; open |
| AUD-013 | Visual regression | Manage Styles expands vertically with a centered Close row and excess blank space. | Fixed; intrinsic footer regression passed and final capture inspected |
| AUD-014 | Major behavior | Empty Hyperlink Address closes the dialog with an error and loses the draft display text. | Runtime reproduced; open |
| AUD-015 | Minor visual | Folded Table Cell Size fields lack visible input boundaries. | Observed; cause and fix pending |
| AUD-016 | Major visual | Compact Print preview footer overlaps page navigation and zoom controls. | backstage/ compact/normal comparison reproduced; open |
| AUD-017 | Major visual | Compact Options clips fields, Browse buttons, dropdowns and explanatory text across seven categories. | All eight categories inspected; responsive allocation repair pending |
| AUD-018 | Major visual | Compact Home/New crop template cards beyond the right edge without horizontal scrolling. | Captures and source confirm layout problem; geometry/interaction regression pending |
| AUD-019 | Behavior gap | Undo restores original inspected content but retains dirty:true, causing unnecessary save prompts. Session::undo always calls touch(). | Reproduced after Chart/SmartArt insertion; savepoint tracking repair pending |
| AUD-020 | Major regression caught in review | Direct Go To leaves its initial-focus flag set, blocking document keyboard selection; Page also repeatedly steals focus. | Runtime reproduced; one-shot focus correction added; Verified: Page/Bookmark and IME regressions, inspected after replay, full CI pass |
| AUD-021 | Major visual | At 1024×480 File sidebar clips Options/About and overlaps Close with the community link. | backstage/short-home.png inspected; scrolling repair staged |
| AUD-022 | Parity gap | Table Properties is a stacked form; Word documents Table/Row/Column/Cell/Alt Text tabs and additional options. | Public reference comparison recorded; current controls fit but hierarchy/depth parity remains open |
| AUD-023 | Major behavior | Modify Table Style for a style without a base has a blank preview and rejects OK because UI sends basedOn as an empty string. | Actual UI reproduced; narrow parameter fix/regressions being staged |
| AUD-024 | Parity gap | Font puts spacing/kerning in one compact form; Word documents an Advanced tab and additional typography controls. | Public reference supports missing hierarchy; full Font layout comparison remains pending |
| AUD-025 | Major interaction | With Font open, a settled actual ribbon Bold click still changes selected document text formatting behind the dialog. | Reproduced with explicit 8-frame settling; shared pointer/modal ownership repair being staged |
| AUD-003 | Audit gap | ui-parity.md duplicated modal-dialog, keyboard and pane rows and falsely claimed Thesaurus presentation. | Those rows consolidated; Linux registry remeasured at 163 commands/192 shortcuts; old aggregate estimate marked historical. Other historical rows still require full state review |

## Verification record

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
| registry-measurement.json; registry.log | Linux command/shortcut counts; no claim that every keyboard path works |
| reference-notes.md | Primary public Word behavior references; no live comparison or copied assets |
| ci.log; ci.exit | Current full gate run and eventual exit code; completion must be checked |

Reusable strict JSONL scenarios live in `crates/ui-egui/tests/scenarios/`. The harness now fails
on unexpected command errors, missing responses and rendering errors, supports state assertions,
and bounds dimensions, DPI, waits and frame counts.

The runtime catalog reports 505/520 registered feature entries (97.1%). This measures command
registration, not complete behavior or visual parity. The inert pane controls demonstrate why
that metric cannot replace this state and interaction audit.

## Exact continuation

1. Preserve the verified first-cycle commit. Keep build inputs pinned to craft-fonts
   `b076417b5880bd4154bd22f00695b1d28c6bea83` and craft-i18n
   `9205108cfdced5177706264038848b68b2422d2b` in ignored `target/audit-inputs/`.
2. Apply and verify staged QAT width, Font/Hyperlink validation, pane and modal-pointer repairs.
   These patches are prepared on copies while the first source revision runs through CI.
3. Repair compact Backstage Print, Options, template-gallery and sidebar allocation (AUD-016–018/021).
   Implement Thesaurus presentation, investigate Zoom rail contrast and folded Cell Size inputs.
4. Continue the uninspected matrix entries, including remaining contextual tabs, Backstage,
   validation/apply/cancel paths, long/complex documents, localization and release-font captures.
5. Obtain direct black-box Word evidence and native platform checks when an environment exists.
   Existing reference documents and public Microsoft Read Mode documentation are secondary
   evidence; they do not establish exhaustive visual or behavioral parity.

Implementation, verification, commit and deployment status are separate. Keyboard isolation does not yet block background pointer commands (AUD-025). The whole mission
remains incomplete while major findings and inventory gaps remain.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-29 | Initial and first-cycle review | Inventory, strict screenshot harness, responsive Read Mode/dialog repairs, input isolation, accessibility labels, Zotero race regression, wider audit and open defect backlog; all eight CI gates passed; broader mission remains open |
