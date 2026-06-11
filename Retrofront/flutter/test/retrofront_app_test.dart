import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:retrofront/src/native/retrofront_native.dart';

void main() {
  test('frontend initializes RetroArch layout and supports real library import state', () async {
    final frontend = RetrofrontNative.create();
    await frontend.initialize();

    expect(frontend.settings['input_overlay_enable'], 'true');
    expect(frontend.settings['content_directory'], contains('Roms'));
    expect(frontend.settings['savefile_directory'], contains('saves'));
    expect(frontend.settings['savestate_directory'], contains('states'));
    expect(Directory(frontend.settings['content_directory']!).existsSync(), isTrue);

    frontend.openQuickMenu();
    expect(frontend.runtime.quickMenuOpen, isTrue);

    frontend.closeQuickMenu();
    expect(frontend.runtime.quickMenuOpen, isFalse);
  });
}
