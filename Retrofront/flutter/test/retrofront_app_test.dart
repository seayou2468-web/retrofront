import 'package:flutter_test/flutter_test.dart';
import 'package:retrofront/src/native/retrofront_native.dart';

void main() {
  test('demo frontend exposes library, cores, settings, and quick-menu state', () async {
    final frontend = RetrofrontNative.create();
    await frontend.initialize();

    expect(frontend.games, isNotEmpty);
    expect(frontend.cores, isNotEmpty);
    expect(frontend.settings['input_overlay_enable'], 'true');

    frontend.openQuickMenu();
    expect(frontend.runtime.quickMenuOpen, isTrue);

    frontend.closeQuickMenu();
    expect(frontend.runtime.quickMenuOpen, isFalse);
  });
}
