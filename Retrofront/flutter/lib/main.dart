import 'dart:io' show Platform;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import 'src/native/retrofront_native.dart';
import 'src/ui/retrofront_app.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (!kIsWeb && Platform.isLinux) {
    await windowManager.ensureInitialized();
    const options = WindowOptions(
      size: Size(1440, 960),
      minimumSize: Size(1180, 760),
      center: true,
      title: 'Retrofront',
      backgroundColor: Color(0xFF050B14),
    );
    await windowManager.waitUntilReadyToShow(options, () async {
      await windowManager.show();
      await windowManager.focus();
    });
  }

  runApp(RetrofrontApp(frontend: RetrofrontNative.create()));
}
