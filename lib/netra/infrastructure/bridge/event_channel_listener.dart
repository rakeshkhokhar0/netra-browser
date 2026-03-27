import 'dart:async';

import 'package:flutter/services.dart';

import '../../shared/config/app_config.dart';

/// Receives browser events from the native C++ bridge via an [EventChannel].
///
/// This infrastructure adapter owns only the native event transport boundary.
/// It keeps a single shared [EventChannel] instance, transforms raw native
/// payloads into `Map<String, dynamic>` values, and exposes a broadcast stream
/// for later engine-layer consumers.
///
/// The listener does not contain business logic and does not interpret native
/// events beyond basic field mapping required to produce a Dart-friendly map.
class EventChannelListener {
  EventChannelListener._();

  /// Shared EventChannel used for browser event delivery from native code.
  ///
  /// The channel name is sourced from [AppConfig] so Dart and native layers
  /// depend on one stable event transport identifier.
  static final EventChannel _eventChannel =
      EventChannel(AppConfig.browserEventChannelName);

  /// Shared broadcast stream of raw browser events emitted by the native
  /// bridge.
  ///
  /// The stream is created once from
  /// [EventChannel.receiveBroadcastStream], transformed into
  /// `Map<String, dynamic>` payloads, and then exposed as a broadcast stream
  /// so multiple Dart listeners can observe the same native event source
  /// without creating duplicate subscriptions.
  static final Stream<Map<String, dynamic>> _events = _eventChannel
      .receiveBroadcastStream()
      .where((event) => event is Map)
      .map(
        (event) => Map<String, dynamic>.from(
          (event as Map).map(
            (key, value) => MapEntry(key.toString(), value),
          ),
        ),
      )
      .asBroadcastStream();

  /// Broadcast stream of raw browser events emitted by the native bridge.
  ///
  /// This getter exposes the shared transformed event stream owned by the
  /// listener so future engine-layer consumers can subscribe without managing
  /// channel or stream setup themselves.
  static Stream<Map<String, dynamic>> get events => _events;
}
