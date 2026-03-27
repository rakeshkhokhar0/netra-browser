#include <flutter/binary_messenger.h>
#include <flutter/encodable_value.h>
#include <flutter/event_channel.h>
#include <flutter/event_sink.h>
#include <flutter/standard_method_codec.h>
#include <flutter/event_stream_handler_functions.h>

#include <cstdint>
#include <memory>
#include <string>
#include <unordered_map>
#include <utility>

namespace {

using EncodableMap = flutter::EncodableMap;
using EncodableValue = flutter::EncodableValue;
using EventChannel = flutter::EventChannel<EncodableValue>;
using EventSink = flutter::EventSink<EncodableValue>;
using StreamHandlerError = flutter::StreamHandlerError<EncodableValue>;

/// Emits browser events from the native WebView2 bridge to Flutter.
///
/// The emitter owns only event translation and delivery. It does not manage
/// browser state, perform filtering, or contain business logic. Events are
/// translated into Flutter-friendly maps and broadcast to all active channel
/// listeners.
class WebViewEventEmitter {
 public:
  /// Returns the global event emitter instance used by the native bridge.
  static WebViewEventEmitter& GetInstance() {
    static WebViewEventEmitter instance;
    return instance;
  }

  WebViewEventEmitter(const WebViewEventEmitter&) = delete;
  WebViewEventEmitter& operator=(const WebViewEventEmitter&) = delete;

  /// Registers the Flutter EventChannel used to stream browser events.
  ///
  /// The emitter keeps the channel alive for the lifetime of the process and
  /// stores active listeners as they subscribe through Flutter.
  void Register(flutter::BinaryMessenger* messenger) {
    if (channel_) {
      return;
    }

    channel_ = std::make_unique<EventChannel>(
        messenger, "netra/browser/events",
        &flutter::StandardMethodCodec::GetInstance());

    auto handler = std::make_unique<flutter::StreamHandlerFunctions<EncodableValue>>(
        [this](const EncodableValue* /*arguments*/,
               std::unique_ptr<EventSink> &&events)
            -> std::unique_ptr<StreamHandlerError> {
          const int64_t listener_id = next_listener_id_++;
          listeners_[listener_id] =
              std::shared_ptr<EventSink>(events.release());
          last_listener_id_ = listener_id;
          return nullptr;
        },
        [this](const EncodableValue* /*arguments*/)
            -> std::unique_ptr<StreamHandlerError> {
          // Erase only the listener that cancelled so other active listeners
          // continue to receive events.
          listeners_.erase(last_listener_id_);
          return nullptr;
        });

    channel_->SetStreamHandler(std::move(handler));
  }

  /// Emits a navigation-starting event for a tab.
  void EmitNavigationStarting(const std::string& tab_id, const std::string& url) {
    Broadcast(BuildEvent("navigationStarting", tab_id, {
                                                       {"url", EncodableValue(url)},
                                                   }));
  }

  /// Emits a content-loading event for a tab.
  void EmitContentLoading(const std::string& tab_id) {
    Broadcast(BuildEvent("contentLoading", tab_id, {}));
  }

  /// Emits a URL-changed event for a tab.
  void EmitUrlChanged(const std::string& tab_id, const std::string& url) {
    Broadcast(BuildEvent("urlChanged", tab_id, {
                                             {"url", EncodableValue(url)},
                                         }));
  }

  /// Emits a navigation-completed event for a tab.
  void EmitNavigationCompleted(const std::string& tab_id,
                               const std::string& url,
                               bool success) {
    Broadcast(BuildEvent("navigationCompleted", tab_id, {
                                                        {"url", EncodableValue(url)},
                                                        {"success", EncodableValue(success)},
                                                    }));
  }

  /// Emits a title-changed event for a tab.
  void EmitTitleChanged(const std::string& tab_id, const std::string& title) {
    Broadcast(BuildEvent("titleChanged", tab_id, {
                                                   {"title", EncodableValue(title)},
                                               }));
  }

  /// Emits a favicon-changed event for a tab.
  void EmitFaviconChanged(const std::string& tab_id,
                          const std::string& favicon_url) {
    Broadcast(BuildEvent("faviconChanged", tab_id, {
                                                     {"faviconUrl", EncodableValue(favicon_url)},
                                                 }));
  }

  /// Emits a history-changed event for a tab.
  void EmitHistoryChanged(const std::string& tab_id,
                          bool can_go_back,
                          bool can_go_forward) {
    Broadcast(BuildEvent("historyChanged", tab_id, {
                                                    {"canGoBack", EncodableValue(can_go_back)},
                                                    {"canGoForward", EncodableValue(can_go_forward)},
                                                }));
  }

  /// Emits a request-blocked event for a tab.
  void EmitRequestBlocked(const std::string& tab_id, const std::string& url) {
    Broadcast(BuildEvent("requestBlocked", tab_id, {
                                                     {"url", EncodableValue(url)},
                                                 }));
  }

  /// Emits an engine-ready event for a tab.
  void EmitEngineReady(const std::string& tab_id) {
    Broadcast(BuildEvent("engineReady", tab_id, {}));
  }

  /// Emits a tab-crashed event for a tab.
  void EmitTabCrashed(const std::string& tab_id) {
    Broadcast(BuildEvent("tabCrashed", tab_id, {}));
  }

 private:
  WebViewEventEmitter() = default;
  ~WebViewEventEmitter() = default;

  /// Builds the standard Flutter event payload shape used by every emission.
  ///
  /// Each event always includes `type` and `tabId`, with any extra fields
  /// placed alongside them so Dart listeners can decode a consistent map shape.
  EncodableValue BuildEvent(
      const char* event_name,
      const std::string& tab_id,
      std::initializer_list<std::pair<const char*, EncodableValue>> extra_fields) {
    EncodableMap event_payload = {
        {EncodableValue("type"), EncodableValue(event_name)},
        {EncodableValue("tabId"), EncodableValue(tab_id)},
    };

    for (const auto& field : extra_fields) {
      event_payload[EncodableValue(field.first)] = field.second;
    }

    return EncodableValue(event_payload);
  }

  /// Broadcasts an event payload to all active Flutter listeners.
  ///
  /// Delivery is non-blocking in the sense that the emitter does not wait on
  /// acknowledgements or perform any synchronous follow-up work beyond writing
  /// the event into each active sink.
  void Broadcast(const EncodableValue& event_payload) {
    for (const auto& listener : listeners_) {
      if (listener.second) {
        listener.second->Success(event_payload);
      }
    }
  }

  /// Event channel instance registered against the Flutter binary messenger.
  std::unique_ptr<EventChannel> channel_;

  /// Active listener registry keyed by a locally assigned identifier.
  std::unordered_map<int64_t, std::shared_ptr<EventSink>> listeners_;

  /// Monotonic identifier assigned as listeners subscribe.
  int64_t next_listener_id_ = 1;

  /// Identifier of the most recently registered listener, used by onCancel to
  /// erase only that specific entry and leave other active listeners intact.
  int64_t last_listener_id_ = 0;
};

}  // namespace

/// Registers the native browser event channel used by Flutter listeners.
///
/// This should be called once during Windows runner startup after the Flutter
/// engine has produced a binary messenger.
void RegisterWebViewEventEmitter(flutter::BinaryMessenger* messenger) {
  WebViewEventEmitter::GetInstance().Register(messenger);
}

/// Emits a navigation-starting event into the Flutter EventChannel stream.
void EmitNavigationStarting(const std::string& tab_id, const std::string& url) {
  WebViewEventEmitter::GetInstance().EmitNavigationStarting(tab_id, url);
}

/// Emits a content-loading event into the Flutter EventChannel stream.
void EmitContentLoading(const std::string& tab_id) {
  WebViewEventEmitter::GetInstance().EmitContentLoading(tab_id);
}

/// Emits a URL-changed event into the Flutter EventChannel stream.
void EmitUrlChanged(const std::string& tab_id, const std::string& url) {
  WebViewEventEmitter::GetInstance().EmitUrlChanged(tab_id, url);
}

/// Emits a navigation-completed event into the Flutter EventChannel stream.
void EmitNavigationCompleted(const std::string& tab_id,
                             const std::string& url,
                             bool success) {
  WebViewEventEmitter::GetInstance().EmitNavigationCompleted(tab_id, url, success);
}

/// Emits a title-changed event into the Flutter EventChannel stream.
void EmitTitleChanged(const std::string& tab_id, const std::string& title) {
  WebViewEventEmitter::GetInstance().EmitTitleChanged(tab_id, title);
}

/// Emits a favicon-changed event into the Flutter EventChannel stream.
void EmitFaviconChanged(const std::string& tab_id,
                        const std::string& favicon_url) {
  WebViewEventEmitter::GetInstance().EmitFaviconChanged(tab_id, favicon_url);
}

/// Emits a history-changed event into the Flutter EventChannel stream.
void EmitHistoryChanged(const std::string& tab_id,
                        bool can_go_back,
                        bool can_go_forward) {
  WebViewEventEmitter::GetInstance().EmitHistoryChanged(
      tab_id, can_go_back, can_go_forward);
}

/// Emits a request-blocked event into the Flutter EventChannel stream.
void EmitRequestBlocked(const std::string& tab_id, const std::string& url) {
  WebViewEventEmitter::GetInstance().EmitRequestBlocked(tab_id, url);
}

/// Emits an engine-ready event into the Flutter EventChannel stream.
void EmitEngineReady(const std::string& tab_id) {
  WebViewEventEmitter::GetInstance().EmitEngineReady(tab_id);
}

/// Emits a tab-crashed event into the Flutter EventChannel stream.
void EmitTabCrashed(const std::string& tab_id) {
  WebViewEventEmitter::GetInstance().EmitTabCrashed(tab_id);
}
