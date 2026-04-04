#include <cstdio>
#include <cstdlib>
#include <string>

#include "bridge/ffi/rust_bridge.h"

namespace {

char* DupCString(const std::string& value) {
  return value.empty() ? nullptr : _strdup(value.c_str());
}

void SendEventToRust(netra::bridge::ffi::FfiNativeBrowserEvent event) {
  std::printf("[C++] Event emitted: %d\n", event.event_type);
  netra::bridge::ffi::SendNativeEventToRust(event);
}

}  // namespace

/// Emits a navigation-starting event into the Rust FFI event pipeline.
void EmitNavigationStarting(const std::string& tab_id, const std::string& url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 1;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  SendEventToRust(ffi_event);
}

/// Emits a content-loading event into the Rust FFI event pipeline.
void EmitContentLoading(const std::string& tab_id) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 4;
  ffi_event.tab_id = DupCString(tab_id);
  SendEventToRust(ffi_event);
}

/// Emits a URL-changed event into the Rust FFI event pipeline.
void EmitUrlChanged(const std::string& tab_id, const std::string& url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 10;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  SendEventToRust(ffi_event);
}

/// Emits a navigation-completed event into the Rust FFI event pipeline.
void EmitNavigationCompleted(const std::string& tab_id,
                             const std::string& url,
                             bool success) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 5;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  ffi_event.bool_value = success ? 1 : 0;
  SendEventToRust(ffi_event);
}

/// Emits a title-changed event into the Rust FFI event pipeline.
void EmitTitleChanged(const std::string& tab_id, const std::string& title) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 6;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(title);
  SendEventToRust(ffi_event);
}

/// Emits a favicon-changed event into the Rust FFI event pipeline.
void EmitFaviconChanged(const std::string& tab_id,
                        const std::string& favicon_url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 12;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(favicon_url);
  SendEventToRust(ffi_event);
}

/// Emits a history-changed event into the Rust FFI event pipeline.
void EmitHistoryChanged(const std::string& tab_id,
                        bool can_go_back,
                        bool can_go_forward) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 7;
  ffi_event.tab_id = DupCString(tab_id);
  if (can_go_back) ffi_event.int_value |= 1;
  if (can_go_forward) ffi_event.int_value |= 2;
  SendEventToRust(ffi_event);
}

/// Emits a request-blocked event into the Rust FFI event pipeline.
void EmitRequestBlocked(const std::string& tab_id, const std::string& url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 8;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  SendEventToRust(ffi_event);
}

/// Emits an engine-ready event into the Rust FFI event pipeline.
void EmitEngineReady(const std::string& tab_id) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 2;
  ffi_event.tab_id = DupCString(tab_id);
  SendEventToRust(ffi_event);
}

/// Emits a tab-crashed event into the Rust FFI event pipeline.
void EmitTabCrashed(const std::string& tab_id) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.event_type = 13;
  ffi_event.tab_id = DupCString(tab_id);
  SendEventToRust(ffi_event);
}
