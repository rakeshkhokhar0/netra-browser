# Netra Browser Audit Fix Tracker

Last updated: 2026-04-04

Status legend:
- `pending`
- `in_progress`
- `verified`
- `deferred`

## Execution Rules

- Scope: Critical + High first
- Batch size: one audit issue per implementation step
- Validation flow: implement -> automated check -> manual verification -> next step

## Audit Items

### Critical

| ID | Status | Current assessment |
| --- | --- | --- |
| C1 | in_progress | Code path looks corrected in `bridge.dart`, but remains open until analyzer/runtime confirmation is completed under the stricter closure standard. |
| C2 | in_progress | Native duplicate lifecycle emissions have been removed from the C++ bridge. Pending runtime verification for create/close behavior and UI stability. |
| C3 | in_progress | Re-entrant deadlock risk appears addressed by the current safe-controller pattern, but remains open until runtime create/close paths are confirmed non-blocking. |
| C4 | in_progress | `create_tab_safe()` now re-reads the authoritative Rust tab snapshot after native creation succeeds. Pending final runtime verification. |

### High

| ID | Status | Notes |
| --- | --- | --- |
| H1 | in_progress | Flutter-side URL normalization has been removed from the control path and the duplicate Dart utility was deleted. Pending analyzer/runtime confirmation. |
| H2 | in_progress | Tab switching now routes through `TabProvider.switchTab()` only. Pending analyzer/runtime confirmation. |
| H3 | in_progress | Closing a tab now removes its navigation history in Rust state. Pending runtime confirmation under manual close-tab scenarios. |
| H4 | in_progress | Rust-to-Flutter event names are now canonicalized in the FFI bridge and Dart models. Pending analyzer/runtime confirmation. |
| H5 | in_progress | Rust now enforces a hard max tab count of 20. Pending runtime/manual confirmation at the creation boundary. |
| H6 | in_progress | Tab crash now has its own dedicated event type through native, Rust, and Flutter. Pending runtime/manual confirmation. |
| H7 | in_progress | Event dispatcher registration now avoids holding the browser-controller mutex while subscribing to the event bus. Pending runtime/manual confirmation under event-heavy flows. |

### Deferred After Blockers

Medium / Low items remain deferred until Critical + High are completed and verified.

## Step Log

### Step 0 - Tracking setup

Status: `verified`

Changes:
- Added this tracker file to the repo under `docs/audit_fix_plan.md`

Automated validation:
- none required

Manual validation:
- not required

### Step 1 - Re-validate critical findings against current head

Status: `verified`

Changes:
- Re-checked C1, C2, C3, and C4 against current source
- Updated statuses above to reflect current head instead of the original audit wording

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed

Manual validation:
- not required

### Step 3 - C2 duplicate native lifecycle event removal

Status: `in_progress`

Changes:
- Removed `EmitTabCreated(tab_id)` from the native create-tab bridge callback
- Removed `EmitTabClosed(tab_id)` from the native close-tab bridge path
- Removed the now-unused native lifecycle emitter functions from `event_emitter.cpp`
- Kept Rust as the single owner of tab lifecycle events (`TabCreated` / `TabClosed`)
- Added Rust-owned lifecycle browser events from `BrowserController` so Flutter still receives authoritative create/close updates without depending on native duplicate emissions

Files changed:
- `windows/runner/bridge/ffi/rust_bridge.cpp`
- `windows/runner/bridge/webview2/event_emitter.cpp`
- `rust/browser/browser_controller.rs`

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `flutter build windows` -> timed out twice in this environment without returning a compile failure

Manual validation:
- pending
- Please verify:
  - create tab once -> no duplicate tab/ghost state
  - close active tab once -> no flicker or duplicate close processing
  - create and close several tabs -> no unexpected extra lifecycle events in UI behavior

### Step 5 - C4 create-tab return contract

Status: `in_progress`

Changes:
- Updated `create_tab_safe()` to return the current authoritative Rust tab snapshot after the native create call succeeds
- Kept the existing interface intact while removing the pre-native snapshot return behavior

Files changed:
- `rust/browser/browser_controller.rs`

Automated validation:
- pending

Manual validation:
- pending
- Please verify:
  - create a new tab
  - confirm the created tab identity matches the rendered active tab
  - confirm startup still opens into a usable first tab

### Step 6 - H1 duplicate URL normalization

Status: `in_progress`

Changes:
- Removed Dart-side address-bar normalization from `BrowserProvider.navigate()`
- Flutter now forwards the raw trimmed input directly to Rust control
- Deleted the unused duplicate Dart normalization utility so Rust remains the only owner of URL/search interpretation rules

Files changed:
- `lib/netra/ui/providers/browser_provider.dart`
- `lib/netra/shared/utils/url_utils.dart`

Automated validation:
- `rg -n "normalizeNavigationInput" lib rust -S` -> only the deleted Dart utility remained before removal; no control-path usages remain now
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `flutter analyze lib/netra/ui/providers/browser_provider.dart lib/netra/ui/components/address_bar.dart` -> timed out in this environment

Manual validation:
- pending
- Please verify:
  - plain search term -> opens search results
  - bare domain -> opens the site
  - full URL -> opens directly
  - localhost/IP input -> still opens directly

### Step 7 - H2 duplicate active-tab control path

Status: `in_progress`

Changes:
- Updated tab-click UI to call `TabProvider.switchTab()` instead of `BrowserProvider.setActiveTab()`
- Removed the redundant `BrowserProvider.setActiveTab()` command path
- Kept the existing refresh behavior inside `TabProvider.switchTab()` so active-tab UI stays Rust-driven

Files changed:
- `lib/netra/ui/components/tab_item.dart`
- `lib/netra/ui/providers/browser_provider.dart`

Automated validation:
- `rg -n "browserProvider\\)\\.setActiveTab|Future<void> setActiveTab\\(|switchTab\\(" lib -S` -> only `TabProvider.switchTab()` remains as the Flutter UI tab-switch command path
- `flutter analyze lib/netra/ui/components/tab_item.dart lib/netra/ui/providers/tab_provider.dart lib/netra/ui/providers/browser_provider.dart` -> timed out in this environment

Manual validation:
- pending
- Please verify:
  - click different tabs -> active highlight follows correctly
  - rapid tab switching -> correct tab remains active
  - switching tabs does not break the visible page content

### Step 8 - H3 navigation history cleanup on closed tabs

Status: `in_progress`

Changes:
- Added `NavigationController::remove_tab_history()` as the explicit history-owner cleanup path
- Updated `BrowserState.close_tab()` to remove navigation history before removing the tab from `TabManager`
- Added Rust tests covering both direct history cleanup and state-level tab-close cleanup

Files changed:
- `rust/browser/navigation_controller.rs`
- `rust/browser/browser_state.rs`

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `cargo test --manifest-path rust/Cargo.toml` -> passed
- Added tests:
  - `browser::navigation_controller::tests::remove_tab_history_clears_only_target_tab_history`
  - `browser::browser_state::tests::close_tab_removes_navigation_history_for_closed_tab`

Manual validation:
- pending
- Please verify:
  - create several tabs, navigate in each, then close one middle tab
  - create/close tabs repeatedly
  - confirm remaining tabs still navigate back/forward correctly without odd state carry-over

### Step 9 - H4 event naming normalization

Status: `in_progress`

Changes:
- Normalized Rust-to-Flutter bridge event names to one canonical lower-camel name per event
- Removed `Native` suffixed transport names from the active Dart event pipeline
- Added typed Dart event classes for `frameCreated`, `frameDestroyed`, and `loadFinished`
- Updated `BrowserProvider` special-case handling to use canonical `frameDestroyed`

Files changed:
- `lib/netra/ffi/bridge.dart`
- `lib/netra/engine/models/browser_event.dart`
- `lib/netra/ui/providers/browser_provider.dart`

Automated validation:
- `rg -n "NavigationCompletedNative|RequestBlockedNative|FrameCreated|FrameDestroyed|LoadStarted|LoadFinished|TitleChanged|UrlChanged|HistoryStateChanged|NavigationFailed" lib/netra -S` -> no remaining legacy transport-name usages in the active bridge/provider path; remaining matches are typed Dart class names
- `flutter analyze lib/netra/ffi/bridge.dart lib/netra/engine/models/browser_event.dart lib/netra/ui/providers/browser_provider.dart` -> timed out in this environment

Manual validation:
- pending
- Please verify:
  - navigation completion still updates correctly
  - blocked-request count still increments correctly
  - failed navigation still shows the error state
  - closing a tab still removes it correctly through the `frameDestroyed` event path

### Step 10 - H5 hard max tab count

Status: `in_progress`

Changes:
- Added a Rust-owned hard max tab count guard in `TabManager`
- `create_tab()` now fails once the browser reaches 20 tabs
- Propagated the create-tab failure through `BrowserState` and `BrowserController`
- Added a Rust test covering rejection after the limit is reached

Files changed:
- `rust/browser/tab_manager.rs`
- `rust/browser/browser_state.rs`
- `rust/browser/browser_controller.rs`

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `cargo test --manifest-path rust/Cargo.toml` -> passed
- Added test:
  - `browser::tab_manager::tests::create_tab_rejects_creation_after_max_tab_limit`

Manual validation:
- pending
- Please verify:
  - create tabs up to 20 -> should still work
  - try creating tab 21 -> should be rejected
  - app should remain usable after the rejection

### Step 11 - H6 tab crash misclassification

Status: `in_progress`

Changes:
- Added `BrowserEvent::TabCrashed` in Rust
- Assigned a dedicated native/FFI event type `13` for tab crashes
- Updated the native emitter to send crash as `event_type = 13` instead of reusing navigation failure
- Updated Rust state to clear loading on crash without treating it as navigation failure
- Updated Flutter bridge and provider to surface a crash-specific UI error message

Files changed:
- `rust/core/entities/browser_event.rs`
- `rust/ffi/mod.rs`
- `rust/browser/browser_state.rs`
- `windows/runner/bridge/webview2/event_emitter.cpp`
- `lib/netra/ffi/bridge.dart`
- `lib/netra/ui/providers/browser_provider.dart`

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `cargo test --manifest-path rust/Cargo.toml` -> passed
- Code search confirms dedicated crash mapping now exists end-to-end via `event_type = 13` and `tabCrashed`

Manual validation:
- pending
- Please verify:
  - trigger a tab/process crash if feasible
  - confirm the UI shows a crash-specific message, not a navigation-failed message
  - confirm the tab stops loading after the crash

### Step 12 - H7 event dispatch under lock

Status: `in_progress`

Changes:
- Re-checked all `event_bus.publish(...)` sites and confirmed they run after `with_locked_controller(...)` returns
- Added `BrowserController::subscribe_safe()` so event subscribers can be registered without holding the browser-controller mutex
- Updated FFI event-dispatcher registration to use the new safe subscription path instead of locking the shared controller to reach `event_bus`
- Kept event delivery synchronous and unchanged at runtime

Files changed:
- `rust/browser/browser_controller.rs`
- `rust/ffi/mod.rs`

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `cargo test --manifest-path rust/Cargo.toml` -> passed
- Publish-site search confirms all `event_bus.publish(...)` calls remain outside `with_locked_controller(...)`

Manual validation:
- pending
- Please verify:
  - create/close/switch tabs repeatedly while watching for freezes
  - navigate rapidly across multiple tabs
  - confirm event-heavy flows do not stall the browser UI
  - confirm no new regression appears in startup or event delivery

### Next actionable step

Wait for manual verification of Step 3 / C2, Step 5 / C4, Step 6 / H1, Step 7 / H2, Step 8 / H3, Step 9 / H4, Step 10 / H5, Step 11 / H6, and Step 12 / H7, then plan the deferred queue.

## Audit Report 5 Follow-up

Execution order:
1. C1 - ConsoleMessage FFI leak
2. C2 - rollback corruption risk
3. C3 - premature NavigationCompleted semantics
4. remaining high-impact correctness and UX items

### Step 1 - Report 5 C1 console-message FFI leak

Status: `in_progress`

Changes:
- Reworked outbound FFI event conversion so ignored `ConsoleMessage` events return before any `FfiBrowserEvent` or `FfiTabState` string allocation occurs
- Added `to_ffi_browser_event(...)` as the single conversion point for Rust-to-Flutter browser events
- Preserved existing Flutter-visible event behavior for all non-console events

Files changed:
- `rust/ffi/mod.rs`

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `cargo test --manifest-path rust/Cargo.toml` -> passed
- Source check confirms `BrowserEvent::ConsoleMessage { .. } => return None` happens before `tab_state: to_ffi_tab_state(current_tab)`

Manual validation:
- pending
- Please verify:
  - browse a console-heavy site for a few minutes
  - switch tabs and navigate normally
  - confirm no new regression appears in event delivery or browser behavior

### Step 2 - Report 5 C2 rollback corruption risk

Status: `in_progress`

Changes:
- Removed full-state snapshot rollback from the controller safe methods:
  - `close_tab_safe`
  - `set_active_tab_safe`
  - `navigate_safe`
  - `go_back_safe`
  - `go_forward_safe`
  - `reload_safe`
  - `stop_loading_safe`
- Native failures in those paths now return directly instead of restoring a stale `BrowserState` clone
- Left the narrow `create_tab_safe` cleanup path in place because it compensates only the just-created Rust tab and does not overwrite the whole browser state

Files changed:
- `rust/browser/browser_controller.rs`

Automated validation:
- `cargo check --manifest-path rust/Cargo.toml` -> passed
- `cargo test --manifest-path rust/Cargo.toml` -> passed
- Search confirms no remaining `controller.state = previous_state` rollback path remains in `browser_controller.rs`

Manual validation:
- pending
- Please verify:
  - create, close, and switch tabs repeatedly
  - navigate, reload, and use back/forward across multiple tabs
  - confirm no strange state rewind or tab reappearance happens during normal use
