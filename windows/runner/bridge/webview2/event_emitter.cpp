#include <cstdio>
#include <cstdlib>
#include <string>
#include <unordered_map>

#include "bridge/ffi/rust_bridge.h"

namespace {

std::unordered_map<std::string, uint32_t> g_tab_event_sequences;

uint32_t NextSequenceForTab(const std::string& tab_id) {
  if (tab_id.empty()) {
    return 0;
  }

  auto entry = g_tab_event_sequences.find(tab_id);
  if (entry == g_tab_event_sequences.end()) {
    g_tab_event_sequences.emplace(tab_id, 1);
    return 1;
  }

  const uint32_t next_value = entry->second + 1;
  entry->second = next_value;
  return next_value;
}

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
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventNavigationStarted;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  SendEventToRust(ffi_event);
}

/// Emits a content-loading event into the Rust FFI event pipeline.
void EmitContentLoading(const std::string& tab_id) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventLoadStarted;
  ffi_event.tab_id = DupCString(tab_id);
  SendEventToRust(ffi_event);
}

/// Emits a URL-changed event into the Rust FFI event pipeline.
void EmitUrlChanged(const std::string& tab_id, const std::string& url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventUrlChanged;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  SendEventToRust(ffi_event);
}

/// Emits a navigation-completed event into the Rust FFI event pipeline.
void EmitNavigationCompleted(const std::string& tab_id,
                             const std::string& url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventNavigationCompleted;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  SendEventToRust(ffi_event);
}

/// Emits a navigation-failed event into the Rust FFI event pipeline.
void EmitNavigationFailed(const std::string& tab_id,
                          const std::string& url,
                          int error_code,
                          const std::string& description) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventNavigationFailed;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  ffi_event.secondary_string = DupCString(description);
  ffi_event.int_value = error_code;
  SendEventToRust(ffi_event);
}

/// Emits a title-changed event into the Rust FFI event pipeline.
void EmitTitleChanged(const std::string& tab_id, const std::string& title) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventTitleChanged;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(title);
  SendEventToRust(ffi_event);
}

/// Emits a favicon-changed event into the Rust FFI event pipeline.
void EmitFaviconChanged(const std::string& tab_id,
                        const std::string& favicon_url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventFaviconChanged;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(favicon_url);
  SendEventToRust(ffi_event);
}

/// Emits a history-changed event into the Rust FFI event pipeline.
void EmitHistoryChanged(const std::string& tab_id,
                        bool can_go_back,
                        bool can_go_forward) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventHistoryStateChanged;
  ffi_event.tab_id = DupCString(tab_id);
  if (can_go_back) ffi_event.int_value |= 1;
  if (can_go_forward) ffi_event.int_value |= 2;
  SendEventToRust(ffi_event);
}

/// Emits a request-blocked event into the Rust FFI event pipeline.
void EmitRequestBlocked(const std::string& tab_id, const std::string& url) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventRequestBlocked;
  ffi_event.tab_id = DupCString(tab_id);
  ffi_event.primary_string = DupCString(url);
  SendEventToRust(ffi_event);
}

/// Emits an engine-ready event into the Rust FFI event pipeline.
void EmitEngineReady(const std::string& tab_id) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventFrameCreated;
  ffi_event.tab_id = DupCString(tab_id);
  SendEventToRust(ffi_event);
}

/// Emits a tab-crashed event into the Rust FFI event pipeline.
void EmitTabCrashed(const std::string& tab_id) {
  netra::bridge::ffi::FfiNativeBrowserEvent ffi_event{};
  ffi_event.sequence_number = NextSequenceForTab(tab_id);
  ffi_event.event_type = netra::bridge::ffi::kEventTabCrashed;
  ffi_event.tab_id = DupCString(tab_id);
  SendEventToRust(ffi_event);
}
