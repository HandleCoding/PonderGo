# Yzy Toolbar Parity Implementation

Goal: implement the Yzy-inspired top command bar and related workflows with modern, clear UI and architecture-aligned backend support.

## Progress

### Phase 1: Command Bar Shell + Existing Actions
- [x] Replace/upgrade top toolbar into grouped command bar (`app/src/lib/toolbar/TopToolbar.svelte`).
- [x] Wire New/Open/Save actions through existing App handlers (`app/src/App.svelte`).
- [x] Add Paste SGF action using clipboard text + `loadSgf`.
- [x] Add board size / komi quick controls; board sizes call `newGame(size)`, komi calls `set_komi`.
- [x] Add live analysis start/stop controls and state chip.
- [x] Add visible entries for later feature groups as real controls, not hidden placeholders.
- [x] Verify frontend check/build.

### Phase 2: Board Markup + Stone Edit Modes
- [x] Add typed SGF markup model using BoardData properties (`BoardMarkup`, `BoardState.markup`).
- [x] Add backend commands for set/remove/clear markup (`set_markup`, `remove_markup`, `clear_markup`).
- [x] Serialize markup into BoardState from `LB/CR/SQ/TR/MA` SGF properties.
- [x] Render LB/CR/SQ/TR/MA-style markup on board (`board-renderer.ts`).
- [x] Add toolbar modes for A, 1, circle, square, triangle, cross, erase.
- [x] Add edit modes for play, black-only, white-only, alternating, delete.
- [x] Preserve setup/edit semantics without polluting normal move history by using setup properties (`AB/AW/AE`) and `add_stone`/`remove_stone` edit commands.
- [x] Add SGF roundtrip tests for markup/setup edits (`test_markup_properties_roundtrip`, `test_setup_properties_write_as_multi_values`).

### Phase 3: Point Constraints + Runtime Engine Params
- [x] Add AnalysisConstraint data model (`AnalysisConstraintRequest`, points, allow/avoid, applies_to).
- [x] Add board point-selection overlay/mode (`selectedPoints` on `BoardCanvas`).
- [x] Add allow/avoid engine command support using existing `GtpEngine::analyze_avoid` with capability gate.
- [x] Add cancel-last and all-cancel controls for selected point constraints.
- [x] Add runtime engine parameter popover.
- [x] Add commands for get/set/reset runtime analysis params (`get_engine_runtime_params`, `set_engine_runtime_params`, `reset_engine_runtime_params`).
- [x] Verify unsupported engines/no engine return clear errors or disabled UI for constraints/params.

### Phase 4: Auto/Flash Analysis + Hawkeye Workspace
- [x] Add AutoAnalysisSettings-style dialog v1 with start/end/time/visits and black/white/all-branch/save options.
- [x] Implement flash analysis entry for current branch/live engine by starting live analysis and setting `analysisJobMode = 'flash'`.
- [x] Implement auto analysis entry with range/filter UI v1 and `analysisJobMode = 'auto'`.
- [x] Add analysis job mode state to avoid ambiguous live/auto/flash UI.
- [x] Add Hawkeye workspace v1 using existing analysis/overview data.
- [x] Link Hawkeye display to selected-region constraints by showing selected point count/global state.
- [x] Add regression tests for SGF/markup persistence and run full checks.

### Verification
- [x] `$HOME/.cargo/bin/cargo test --manifest-path Cargo.toml` — passed, 78 tests.
- [x] `npm --prefix app run check` — passed, 0 errors, 4 pre-existing warnings.
- [x] `npm --prefix app run build` — passed.
- [x] Code review pass: checked toolbar grouping, backend command registration, SGF persistence, engine capability gates, and selected-point state flow.

## Review Checklist
- [x] Toolbar groups separate high-frequency workflows clearly.
- [x] Markup persists through SGF properties instead of frontend-only overlays.
- [x] Edit/setup modes avoid normal move placement and capture paths.
- [x] Engine constraints are capability-gated and cannot silently do nothing.
- [x] Analysis job UI has explicit live/flash/auto modes.
- [x] UI remains readable; low-frequency controls live in popovers/dialogs.

## Implementation Evidence

- `TopToolbar.svelte`: grouped command bar for 对局 / 行棋 / 分析 / 标注 / 摆子 / 选点 / 高级.
- `App.svelte`: state machine for board modes, markup modes, selected points, flash/auto/live analysis modes, Hawkeye visibility, paste SGF, komi, runtime params.
- `board.rs`: `BoardMarkup`, `BoardState.markup`, SGF-property-derived markup serialization.
- `board_cmd.rs`: `set_komi`, `set_markup`, `remove_markup`, `clear_markup`, setup-property-aware stone edit commands.
- `sgf.rs`: root/current-node property writing and tests for markup/setup persistence.
- `gtp.rs` / `engine_cmd.rs`: runtime analyze interval, reset params, KataGo point-constraint analysis.
- `BoardCanvas.svelte` / `board-renderer.ts`: markup rendering and selected-point overlay.

## Notes

- Auto analysis and flash analysis currently provide functional v1 entry points on top of live analysis; deeper queued per-move batch analysis can build on the new `analysisJobMode` and dialog state.
- Runtime engine parameters currently expose analyze interval because it is the supported runtime-safe parameter in `GtpEngine`; visits/playouts-specific controls should be added after engine capability plumbing is explicit.
- Point constraints use KataGo-compatible allow/avoid analysis and are disabled/error-gated when no engine or unsupported engine is active.
