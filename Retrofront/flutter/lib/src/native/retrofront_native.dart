import 'dart:ffi' as ffi;
import 'dart:io' show Directory, File, Platform;
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

final class RfFrontend extends ffi.Opaque {}

final class RfVideoFrameInfo extends ffi.Struct {
  @ffi.Uint32()
  external int width;
  @ffi.Uint32()
  external int height;
  @ffi.Uint64()
  external int pitch;
  @ffi.Uint64()
  external int rgbaLen;
  @ffi.Uint32()
  external int pixelFormat;
  @ffi.Uint64()
  external int frameNumber;
}

final class RfOverlayInfo extends ffi.Struct {
  @ffi.Bool()
  external bool enabled;
  @ffi.UintPtr()
  external int activeIndex;
  @ffi.UintPtr()
  external int overlayCount;
  external ffi.Pointer<Utf8> activeName;
}

final class RfCoreOptionValue extends ffi.Struct {
  external ffi.Pointer<Utf8> value;
  external ffi.Pointer<Utf8> label;
}

final class RfCoreOption extends ffi.Struct {
  external ffi.Pointer<Utf8> key;
  external ffi.Pointer<Utf8> desc;
  external ffi.Pointer<Utf8> info;
  external ffi.Pointer<Utf8> value;
  external ffi.Pointer<RfCoreOptionValue> values;
  @ffi.UintPtr()
  external int valuesCount;
}

class GameEntry {
  const GameEntry({
    required this.title,
    required this.system,
    required this.core,
    required this.lastPlayed,
    required this.playTime,
    required this.path,
    required this.initials,
  });

  final String title;
  final String system;
  final String core;
  final String lastPlayed;
  final String playTime;
  final String path;
  final String initials;
}

class CoreEntry {
  const CoreEntry({required this.name, required this.system, required this.path, this.loaded = false});

  final String name;
  final String system;
  final String path;
  final bool loaded;
}

class PlaylistEntry {
  const PlaylistEntry({required this.name, required this.count, required this.icon});

  final String name;
  final String count;
  final String icon;
}

class CoreOptionEntry {
  const CoreOptionEntry({required this.key, required this.description, required this.value, required this.values});

  final String key;
  final String description;
  final String value;
  final List<String> values;
}

class RuntimeState {
  const RuntimeState({
    required this.loadedCore,
    required this.loadedGame,
    required this.running,
    required this.frameNumber,
    required this.overlayEnabled,
    required this.quickMenuOpen,
  });

  final String loadedCore;
  final String loadedGame;
  final bool running;
  final int frameNumber;
  final bool overlayEnabled;
  final bool quickMenuOpen;

  RuntimeState copyWith({
    String? loadedCore,
    String? loadedGame,
    bool? running,
    int? frameNumber,
    bool? overlayEnabled,
    bool? quickMenuOpen,
  }) {
    return RuntimeState(
      loadedCore: loadedCore ?? this.loadedCore,
      loadedGame: loadedGame ?? this.loadedGame,
      running: running ?? this.running,
      frameNumber: frameNumber ?? this.frameNumber,
      overlayEnabled: overlayEnabled ?? this.overlayEnabled,
      quickMenuOpen: quickMenuOpen ?? this.quickMenuOpen,
    );
  }
}

abstract interface class RetrofrontFrontend {
  List<GameEntry> get games;
  List<CoreEntry> get cores;
  List<PlaylistEntry> get playlists;
  Map<String, String> get settings;
  RuntimeState get runtime;

  Future<void> initialize();
  Future<void> importRom(String path, {bool autoAssignCore});
  Future<void> scanRoms(String directory);
  Future<bool> loadCore(CoreEntry core);
  Future<bool> launch(GameEntry game);
  Future<bool> runFrame();
  Future<bool> quickSave({int slot = 0});
  Future<bool> quickLoad({int slot = 0});
  Future<bool> reset();
  Future<void> setJoypadButton(int buttonId, bool pressed);
  Future<void> loadOverlay(String path);
  Future<void> setOverlayEnabled(bool enabled);
  Future<List<CoreOptionEntry>> coreOptions();
  Future<bool> setCoreOption(String key, String value);
  Future<void> setSetting(String key, String value);
  Future<Uint8List?> copyVideoFrameRgba();
  void openQuickMenu();
  void closeQuickMenu();
}

class RetrofrontNative implements RetrofrontFrontend {
  RetrofrontNative._(this._library) {
    if (_library != null) {
      _bindNative(_library!);
      _handle = _create!();
    }
  }

  factory RetrofrontNative.create() {
    return RetrofrontNative._(_openLibraryOrNull());
  }

  static ffi.DynamicLibrary? _openLibraryOrNull() {
    final candidates = <String>[
      if (Platform.isIOS) 'retrofront_core.framework/retrofront_core',
      if (Platform.isIOS) 'libretrofront_core.a',
      if (Platform.isLinux) 'libretrofront_core.so',
      if (Platform.isLinux) p.join(Directory.current.path, 'libretrofront_core.so'),
    ];
    for (final candidate in candidates) {
      try {
        return ffi.DynamicLibrary.open(candidate);
      } catch (_) {
        // Try the next packaged/library-path location. The demo fallback keeps
        // the UI testable when the native core has not been bundled yet.
      }
    }
    return null;
  }

  final ffi.DynamicLibrary? _library;
  ffi.Pointer<RfFrontend>? _handle;

  late final ffi.Pointer<RfFrontend> Function() _create;
  late final void Function(ffi.Pointer<RfFrontend>) _destroy;
  late final int Function(ffi.Pointer<RfFrontend>) _state;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _loadCore;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _launchContent;
  late final bool Function(ffi.Pointer<RfFrontend>) _runFrame;
  late final bool Function(ffi.Pointer<RfFrontend>) _reset;
  late final bool Function(ffi.Pointer<RfFrontend>, int) _saveState;
  late final bool Function(ffi.Pointer<RfFrontend>, int) _loadState;
  late final bool Function(ffi.Pointer<RfFrontend>, int, bool) _setJoypadButton;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _loadOverlay;
  late final void Function(ffi.Pointer<RfFrontend>, bool) _setOverlayEnabled;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfOverlayInfo>) _overlayInfo;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfVideoFrameInfo>) _videoFrameInfo;
  late final int Function(ffi.Pointer<RfFrontend>, ffi.Pointer<ffi.Uint8>, int) _copyFrame;
  late final int Function(ffi.Pointer<RfFrontend>) _optionsCount;
  late final bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreOption>) _getOption;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _setOption;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _setSettingNative;

  @override
  final List<GameEntry> games = [
    const GameEntry(title: 'Street Fighter III 3rd Strike', system: 'PlayStation', core: 'Beetle PSX', lastPlayed: '2024/05/20', playTime: '12h 30m', path: '/storage/roms/sfiii3.zip', initials: 'SF'),
    const GameEntry(title: 'Metal Slug X', system: 'Neo Geo', core: 'FB Neo', lastPlayed: '2024/05/19', playTime: '6h 15m', path: '/storage/roms/mslugx.zip', initials: 'MS'),
    const GameEntry(title: 'The Legend of Zelda', system: 'Super Famicom', core: 'Snes9x', lastPlayed: '2024/05/18', playTime: '8h 45m', path: '/storage/roms/zelda.sfc', initials: 'Z'),
    const GameEntry(title: 'Sonic Adventure 2', system: 'Dreamcast', core: 'Flycast', lastPlayed: '2024/05/17', playTime: '5h 20m', path: '/storage/roms/sa2.chd', initials: 'SA'),
    const GameEntry(title: 'Castlevania: Symphony of the Night', system: 'PlayStation', core: 'Beetle PSX', lastPlayed: '2024/05/16', playTime: '7h 10m', path: '/storage/roms/sotn.cue', initials: 'CV'),
    const GameEntry(title: 'Radiant Silvergun', system: 'Saturn', core: 'YabaSanshiro', lastPlayed: '2024/05/15', playTime: '3h 50m', path: '/storage/roms/rs.cue', initials: 'RS'),
    const GameEntry(title: 'Super Mario World', system: 'Super Famicom', core: 'Snes9x', lastPlayed: '2024/05/14', playTime: '4h 05m', path: '/storage/roms/smw.sfc', initials: 'SM'),
    const GameEntry(title: 'Chrono Trigger', system: 'Super Famicom', core: 'Snes9x', lastPlayed: '2024/05/14', playTime: '9h 25m', path: '/storage/roms/ct.sfc', initials: 'CT'),
  ];

  @override
  List<CoreEntry> cores = const [
    CoreEntry(name: 'Beetle PSX', system: 'PlayStation', path: 'beetle_psx_libretro', loaded: true),
    CoreEntry(name: 'Snes9x', system: 'Super Nintendo', path: 'snes9x_libretro'),
    CoreEntry(name: 'FB Neo', system: 'Arcade (Neo Geo)', path: 'fbneo_libretro'),
    CoreEntry(name: 'Flycast', system: 'Sega Dreamcast', path: 'flycast_libretro'),
    CoreEntry(name: 'YabaSanshiro', system: 'Sega Saturn', path: 'yabasanshiro_libretro'),
    CoreEntry(name: 'Mupen64Plus-Next', system: 'Nintendo 64', path: 'mupen64plus_next_libretro'),
  ];

  @override
  final List<PlaylistEntry> playlists = const [
    PlaylistEntry(name: 'お気に入り', count: '32 ゲーム', icon: '♥'),
    PlaylistEntry(name: '最近プレイ', count: '15 ゲーム', icon: '◴'),
    PlaylistEntry(name: 'クリア済み', count: '48 ゲーム', icon: '◆'),
    PlaylistEntry(name: 'アーケード', count: '128 ゲーム', icon: '🎮'),
    PlaylistEntry(name: 'RPG', count: '64 ゲーム', icon: '☯'),
    PlaylistEntry(name: 'レトロアクション', count: '101 ゲーム', icon: '▣'),
  ];

  @override
  final Map<String, String> settings = {
    'theme': 'dark',
    'accent_color': 'violet',
    'video_driver': Platform.isIOS ? 'metal' : 'opengl',
    'video_vsync': 'true',
    'video_scale_mode': 'integer_fit',
    'audio_driver': 'native',
    'audio_latency_ms': '64',
    'input_driver': 'gamecontroller',
    'input_overlay_enable': 'true',
    'input_overlay_opacity': '0.70',
    'savestate_auto_save': 'false',
    'savestate_auto_load': 'false',
    'rewind_enable': 'false',
    'libretro_directory': Platform.isIOS ? 'Documents/RetroArch/cores' : '~/.config/retroarch/cores',
    'content_directory': Platform.isIOS ? 'Documents/RetroArch/downloads' : '~/Games/ROMs',
    'savefile_directory': 'saves',
    'savestate_directory': 'states',
    'overlay_directory': 'overlays',
  };

  @override
  RuntimeState runtime = const RuntimeState(
    loadedCore: 'Beetle PSX',
    loadedGame: 'Street Fighter III 3rd Strike',
    running: false,
    frameNumber: 0,
    overlayEnabled: true,
    quickMenuOpen: false,
  );

  void _bindNative(ffi.DynamicLibrary lib) {
    _create = lib.lookupFunction<ffi.Pointer<RfFrontend> Function(), ffi.Pointer<RfFrontend> Function()>('rf_frontend_create');
    _destroy = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>), void Function(ffi.Pointer<RfFrontend>)>('rf_frontend_destroy');
    _state = lib.lookupFunction<ffi.Uint32 Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_state');
    _loadCore = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_load_core');
    _launchContent = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_launch_content');
    _runFrame = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>), bool Function(ffi.Pointer<RfFrontend>)>('rf_frontend_run_frame');
    _reset = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>), bool Function(ffi.Pointer<RfFrontend>)>('rf_frontend_reset');
    _saveState = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Uint32), bool Function(ffi.Pointer<RfFrontend>, int)>('rf_frontend_save_state');
    _loadState = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Uint32), bool Function(ffi.Pointer<RfFrontend>, int)>('rf_frontend_load_state');
    _setJoypadButton = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Uint32, ffi.Bool), bool Function(ffi.Pointer<RfFrontend>, int, bool)>('rf_frontend_set_joypad_button');
    _loadOverlay = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_load_overlay');
    _setOverlayEnabled = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Bool), void Function(ffi.Pointer<RfFrontend>, bool)>('rf_frontend_set_overlay_enabled');
    _overlayInfo = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfOverlayInfo>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfOverlayInfo>)>('rf_frontend_overlay_info');
    _videoFrameInfo = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfVideoFrameInfo>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfVideoFrameInfo>)>('rf_frontend_video_frame_info');
    _copyFrame = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>, ffi.Pointer<ffi.Uint8>, ffi.UintPtr), int Function(ffi.Pointer<RfFrontend>, ffi.Pointer<ffi.Uint8>, int)>('rf_frontend_copy_video_frame_rgba');
    _optionsCount = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_options_count');
    _getOption = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Pointer<RfCoreOption>), bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreOption>)>('rf_frontend_get_option');
    _setOption = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_set_option');
    _setSettingNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_set_setting');
  }

  @override
  Future<void> initialize() async {
    final baseDir = await getApplicationSupportDirectory();
    settings['base_dir'] = p.join(baseDir.path, 'RetroArch');
    settings['savefile_directory'] = p.join(settings['base_dir']!, 'saves');
    settings['savestate_directory'] = p.join(settings['base_dir']!, 'states');
    settings['overlay_directory'] = p.join(settings['base_dir']!, 'overlays');
    for (final key in ['savefile_directory', 'savestate_directory', 'overlay_directory']) {
      Directory(settings[key]!).createSync(recursive: true);
    }
  }

  @override
  Future<void> importRom(String path, {bool autoAssignCore = true}) async {
    final name = p.basenameWithoutExtension(path);
    final ext = p.extension(path).replaceFirst('.', '').toLowerCase();
    final core = autoAssignCore ? _bestCoreForExtension(ext) : cores.first.name;
    games.insert(0, GameEntry(title: _titleCase(name), system: _systemForCore(core), core: core, lastPlayed: '未プレイ', playTime: '0m', path: path, initials: _initials(name)));
  }

  @override
  Future<void> scanRoms(String directory) async {
    final dir = Directory(directory);
    if (!dir.existsSync()) return;
    final supported = {'zip', 'gba', 'gb', 'gbc', 'sfc', 'smc', 'nes', 'cue', 'chd', 'iso'};
    for (final entity in dir.listSync(recursive: true).whereType<File>()) {
      final ext = p.extension(entity.path).replaceFirst('.', '').toLowerCase();
      if (supported.contains(ext) && !games.any((game) => game.path == entity.path)) {
        await importRom(entity.path);
      }
    }
  }

  @override
  Future<bool> loadCore(CoreEntry core) async {
    var ok = true;
    final handle = _handle;
    if (handle != null) {
      final cPath = core.path.toNativeUtf8();
      try {
        ok = _loadCore(handle, cPath);
      } finally {
        malloc.free(cPath);
      }
    }
    if (ok || handle == null) {
      cores = [for (final item in cores) CoreEntry(name: item.name, system: item.system, path: item.path, loaded: item.name == core.name)];
      runtime = runtime.copyWith(loadedCore: core.name);
    }
    return ok;
  }

  @override
  Future<bool> launch(GameEntry game) async {
    var ok = true;
    final handle = _handle;
    if (handle != null) {
      final cPath = game.path.toNativeUtf8();
      final cCore = game.core.toNativeUtf8();
      final cMeta = ''.toNativeUtf8();
      try {
        ok = _launchContent(handle, cPath, cCore, cMeta);
      } finally {
        malloc.free(cPath);
        malloc.free(cCore);
        malloc.free(cMeta);
      }
    }
    if (ok || handle == null) {
      runtime = runtime.copyWith(loadedGame: game.title, loadedCore: game.core, running: true);
    }
    return ok;
  }

  @override
  Future<bool> runFrame() async {
    final handle = _handle;
    final ok = handle == null ? true : _runFrame(handle);
    if (ok) runtime = runtime.copyWith(frameNumber: runtime.frameNumber + 1, running: true);
    return ok;
  }

  @override
  Future<bool> quickSave({int slot = 0}) async => _handle == null ? true : _saveState(_handle!, slot);

  @override
  Future<bool> quickLoad({int slot = 0}) async => _handle == null ? true : _loadState(_handle!, slot);

  @override
  Future<bool> reset() async {
    final ok = _handle == null ? true : _reset(_handle!);
    if (ok) runtime = runtime.copyWith(frameNumber: 0, running: true);
    return ok;
  }

  @override
  Future<void> setJoypadButton(int buttonId, bool pressed) async {
    if (_handle != null) _setJoypadButton(_handle!, buttonId, pressed);
  }

  @override
  Future<void> loadOverlay(String path) async {
    if (_handle != null) {
      final cPath = path.toNativeUtf8();
      try {
        _loadOverlay(_handle!, cPath);
      } finally {
        malloc.free(cPath);
      }
    }
    settings['input_overlay'] = path;
  }

  @override
  Future<void> setOverlayEnabled(bool enabled) async {
    if (_handle != null) _setOverlayEnabled(_handle!, enabled);
    runtime = runtime.copyWith(overlayEnabled: enabled);
    settings['input_overlay_enable'] = enabled.toString();
  }

  @override
  Future<List<CoreOptionEntry>> coreOptions() async {
    final handle = _handle;
    if (handle == null) {
      return const [
        CoreOptionEntry(key: 'psx_cpu_freq', description: 'CPU Frequency Scaling', value: '100%', values: ['50%', '75%', '100%', '125%']),
        CoreOptionEntry(key: 'psx_renderer', description: 'Renderer', value: 'Hardware', values: ['Software', 'Hardware']),
        CoreOptionEntry(key: 'psx_dithering', description: 'Dithering Pattern', value: 'Internal', values: ['Off', 'Internal', '1x Native']),
      ];
    }
    final result = <CoreOptionEntry>[];
    final count = _optionsCount(handle);
    final raw = malloc<RfCoreOption>();
    try {
      for (var i = 0; i < count; i++) {
        if (_getOption(handle, i, raw)) {
          final values = <String>[];
          for (var j = 0; j < raw.ref.valuesCount; j++) {
            values.add(raw.ref.values[j].value.toDartString());
          }
          result.add(CoreOptionEntry(
            key: raw.ref.key.toDartString(),
            description: raw.ref.desc.toDartString(),
            value: raw.ref.value.toDartString(),
            values: values,
          ));
        }
      }
    } finally {
      malloc.free(raw);
    }
    return result;
  }

  @override
  Future<bool> setCoreOption(String key, String value) async {
    final handle = _handle;
    if (handle == null) return true;
    final cKey = key.toNativeUtf8();
    final cValue = value.toNativeUtf8();
    try {
      return _setOption(handle, cKey, cValue);
    } finally {
      malloc.free(cKey);
      malloc.free(cValue);
    }
  }

  @override
  Future<void> setSetting(String key, String value) async {
    settings[key] = value;
    final handle = _handle;
    if (handle == null) return;
    final cKey = key.toNativeUtf8();
    final cValue = value.toNativeUtf8();
    try {
      _setSettingNative(handle, cKey, cValue);
    } finally {
      malloc.free(cKey);
      malloc.free(cValue);
    }
  }

  @override
  Future<Uint8List?> copyVideoFrameRgba() async {
    final handle = _handle;
    if (handle == null) return _demoFrame(runtime.frameNumber);
    final info = malloc<RfVideoFrameInfo>();
    try {
      if (!_videoFrameInfo(handle, info) || info.ref.rgbaLen == 0) return null;
      final buffer = malloc<ffi.Uint8>(info.ref.rgbaLen);
      try {
        final copied = _copyFrame(handle, buffer, info.ref.rgbaLen);
        return Uint8List.fromList(buffer.asTypedList(copied));
      } finally {
        malloc.free(buffer);
      }
    } finally {
      malloc.free(info);
    }
  }

  @override
  void openQuickMenu() => runtime = runtime.copyWith(quickMenuOpen: true);

  @override
  void closeQuickMenu() => runtime = runtime.copyWith(quickMenuOpen: false);

  void dispose() {
    final handle = _handle;
    if (handle != null) _destroy(handle);
  }

  String _bestCoreForExtension(String ext) {
    return switch (ext) {
      'sfc' || 'smc' => 'Snes9x',
      'zip' => 'FB Neo',
      'cue' || 'chd' => 'Beetle PSX',
      _ => cores.first.name,
    };
  }

  String _systemForCore(String core) => cores.firstWhere((item) => item.name == core, orElse: () => cores.first).system;

  String _titleCase(String name) => name.replaceAll(RegExp('[_-]+'), ' ').split(' ').where((part) => part.isNotEmpty).map((part) => '${part[0].toUpperCase()}${part.substring(1)}').join(' ');

  String _initials(String name) => name.split(RegExp(r'[_\-\s]+')).where((part) => part.isNotEmpty).take(2).map((part) => part[0].toUpperCase()).join();

  Uint8List _demoFrame(int frame) {
    const width = 320;
    const height = 180;
    final bytes = Uint8List(width * height * 4);
    for (var y = 0; y < height; y++) {
      for (var x = 0; x < width; x++) {
        final index = (y * width + x) * 4;
        final wave = (math.sin((x + frame) / 18) + math.cos((y - frame) / 15)) * 40 + 90;
        bytes[index] = (30 + wave).clamp(0, 255).toInt();
        bytes[index + 1] = (80 + x / width * 120).clamp(0, 255).toInt();
        bytes[index + 2] = (120 + y / height * 90).clamp(0, 255).toInt();
        bytes[index + 3] = 255;
      }
    }
    return bytes;
  }
}
