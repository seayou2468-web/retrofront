import 'package:flutter/material.dart';
import 'src/native/retrofront_native.dart';
import 'src/ui/retrofront_app.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(RetrofrontApp(frontend: RetrofrontNative.create()));
}
