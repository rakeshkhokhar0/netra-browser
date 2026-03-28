/// Normalizes raw address-bar input into a browser-ready navigation target.
///
/// The address bar behaves like a browser omnibox:
/// - free-form queries are converted into a search URL
/// - direct addresses are opened as URLs
/// - missing schemes are prefixed with `https://`
String normalizeNavigationInput(String input) {
  final trimmedInput = input.trim();
  if (trimmedInput.isEmpty) {
    return '';
  }

  if (trimmedInput.contains(' ')) {
    return _toSearchUrl(trimmedInput);
  }

  if (_hasScheme(trimmedInput)) {
    return trimmedInput;
  }

  if (_looksLikeDirectAddress(trimmedInput)) {
    return 'https://$trimmedInput';
  }

  return _toSearchUrl(trimmedInput);
}

/// Converts free-form search text into a Google search URL.
String _toSearchUrl(String query) {
  final encodedQuery = query.trim().split(RegExp(r'\s+')).join('+');
  return 'https://www.google.com/search?q=$encodedQuery';
}

/// Returns whether the input already contains an explicit URI scheme.
bool _hasScheme(String input) {
  final parsed = Uri.tryParse(input);
  return parsed?.hasScheme ?? false;
}

/// Determines whether the input looks like a direct address rather than a
/// search query.
bool _looksLikeDirectAddress(String input) {
  if (input.startsWith('localhost')) {
    return true;
  }

  if (_ipv4Pattern.hasMatch(input)) {
    return true;
  }

  return input.contains('.');
}

final RegExp _ipv4Pattern = RegExp(
  r'^(\d{1,3}\.){3}\d{1,3}(:\d+)?([/?#].*)?$',
);
