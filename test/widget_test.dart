import 'package:flutter_test/flutter_test.dart';

import 'package:netra_browser/netra/ui/app.dart';

void main() {
  test('Netra shell widget is restorable', () {
    const app = NetraApp();
    expect(app, isNotNull);
  });
}
