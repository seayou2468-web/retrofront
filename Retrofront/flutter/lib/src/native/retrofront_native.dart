import 'dart:convert';
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


final class RfCoreInfo extends ffi.Struct {
  external ffi.Pointer<Utf8> path;
  external ffi.Pointer<Utf8> displayName;
  external ffi.Pointer<Utf8> systemName;
  external ffi.Pointer<Utf8> supportedExtensions;
}

final class RfGameInfo extends ffi.Struct {
  external ffi.Pointer<Utf8> path;
  external ffi.Pointer<Utf8> label;
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
  String get statusMessage;
  Future<void> importRom(String path, {bool autoAssignCore = true, bool copyToLibrary = true});
  Future<void> scanRoms(String directory);
  Future<void> scanCores(String directory);
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
  late final void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _setBaseDirNative;
  late final void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _setInfoDirNative;
  late final void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _scanCoresNative;
  late final int Function(ffi.Pointer<RfFrontend>) _coresCountNative;
  late final bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreInfo>) _getCoreInfoNative;
  late final ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>) _allExtensionsNative;
  late final void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _scanGamesNative;
  late final int Function(ffi.Pointer<RfFrontend>) _gamesCountNative;
  late final bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfGameInfo>) _getGameInfoNative;

  @override
  final List<GameEntry> games = <GameEntry>[];

  @override
  List<CoreEntry> cores = <CoreEntry>[];

  @override
  final List<PlaylistEntry> playlists = const [
    PlaylistEntry(name: 'お気に入り', count: '0 ゲーム', icon: '♥'),
    PlaylistEntry(name: '最近プレイ', count: '0 ゲーム', icon: '◴'),
    PlaylistEntry(name: 'インポート済み', count: '0 ゲーム', icon: '◆'),
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
    'libretro_directory': 'cores',
    'libretro_info_path': 'info',
    'assets_directory': 'assets',
    'menu_assets_directory': 'assets',
    'content_directory': 'Roms',
    'menu_content_directory': 'RetroArch',
    'core_assets_directory': 'downloads',
    'system_directory': 'system',
    'playlist_directory': 'playlists',
    'cache_directory': 'cache',
    'savefile_directory': 'saves',
    'savestate_directory': 'states',
    'overlay_directory': 'overlays',
  };

  @override
  RuntimeState runtime = const RuntimeState(
    loadedCore: '',
    loadedGame: '',
    running: false,
    frameNumber: 0,
    overlayEnabled: true,
    quickMenuOpen: false,
  );

  @override
  String statusMessage = 'Ready';

  Directory? _retroArchRoot;

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
    _setBaseDirNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_set_base_dir');
    _setInfoDirNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_set_info_dir');
    _scanCoresNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_scan_cores');
    _coresCountNative = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_cores_count');
    _getCoreInfoNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Pointer<RfCoreInfo>), bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreInfo>)>('rf_frontend_get_core_info');
    _allExtensionsNative = lib.lookupFunction<ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>), ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>)>('rf_frontend_all_extensions');
    _scanGamesNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_scan_games');
    _gamesCountNative = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_games_count');
    _getGameInfoNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Pointer<RfGameInfo>), bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfGameInfo>)>('rf_frontend_get_game_info');
  }

  @override
  Future<void> initialize() async {
    final root = await _resolveRetroArchRoot();
    _retroArchRoot = root;
    final dirs = <String, String>{
      'base_dir': root.path,
      'config_directory': p.join(root.path, 'config'),
      'assets_directory': p.join(root.path, 'assets'),
      'menu_assets_directory': p.join(root.path, 'assets'),
      'libretro_info_path': p.join(root.path, 'info'),
      'overlay_directory': p.join(root.path, 'overlays'),
      'core_assets_directory': p.join(root.path, 'downloads'),
      'content_directory': p.join(root.path, 'Roms'),
      'menu_content_directory': root.path,
      'savefile_directory': p.join(root.path, 'saves'),
      'savestate_directory': p.join(root.path, 'states'),
      'system_directory': p.join(root.path, 'system'),
      'playlist_directory': p.join(root.path, 'playlists'),
      'cache_directory': p.join(root.path, 'cache'),
      'libretro_directory': p.join(root.path, 'cores'),
      'core_options_path': p.join(root.path, 'retroarch-core-options.cfg'),
      'input_overlay': p.join(root.path, 'overlays', 'gamepads', 'flat', 'retropad.cfg'),
    };
    settings.addAll(dirs);
    for (final dir in dirs.entries.where((entry) => entry.key != 'core_options_path' && entry.key != 'input_overlay')) {
      Directory(dir.value).createSync(recursive: true);
    }
    Directory(p.dirname(settings['input_overlay']!)).createSync(recursive: true);
    await _loadPersistedSettings();
    _applyAllSettingsToNative();
    await scanCores(settings['libretro_directory']!);
    await scanRoms(settings['content_directory']!);
    statusMessage = _handle == null
        ? 'Native core library not found; UI is running in preview mode.'
        : 'RetroArch-compatible storage initialized.';
  }

  @override
  Future<void> importRom(String path, {bool autoAssignCore = true, bool copyToLibrary = true}) async {
    final source = File(path);
    if (!source.existsSync()) {
      statusMessage = 'Import failed: file does not exist.';
      return;
    }
    final contentDir = Directory(settings['content_directory'] ?? p.dirname(path));
    contentDir.createSync(recursive: true);
    var importedPath = source.path;
    if (copyToLibrary) {
      final destination = File(p.join(contentDir.path, p.basename(path)));
      if (source.absolute.path != destination.absolute.path) {
        if (destination.existsSync()) destination.deleteSync();
        source.copySync(destination.path);
      }
      importedPath = destination.path;
    }
    final name = p.basenameWithoutExtension(importedPath);
    final ext = p.extension(importedPath).replaceFirst('.', '').toLowerCase();
    final core = autoAssignCore ? _bestCoreForExtension(ext) : (cores.isNotEmpty ? cores.first.name : '');
    games.removeWhere((game) => game.path == importedPath);
    games.insert(0, GameEntry(title: _titleCase(name), system: _systemForCore(core), core: core, lastPlayed: '未プレイ', playTime: '0m', path: importedPath, initials: _initials(name)));
    await _persistLibrary();
    statusMessage = 'Imported ${p.basename(importedPath)}';
  }

  @override
  Future<void> scanRoms(String directory) async {
    final dir = Directory(directory);
    if (!dir.existsSync()) return;
    if (_handle != null) {
      final extString = _allExtensionsNative(_handle!).toDartString();
      final cDir = directory.toNativeUtf8();
      final cExt = (extString.isEmpty ? _fallbackExtensions.join('|') : extString).toNativeUtf8();
      try {
        _scanGamesNative(_handle!, cDir, cExt);
        _refreshNativeGames();
      } finally {
        malloc.free(cDir);
        malloc.free(cExt);
      }
    } else {
      final supported = _fallbackExtensions;
      for (final entity in dir.listSync(recursive: true).whereType<File>()) {
        final ext = p.extension(entity.path).replaceFirst('.', '').toLowerCase();
        if (supported.contains(ext) && !games.any((game) => game.path == entity.path)) {
          await importRom(entity.path, copyToLibrary: false);
        }
      }
    }
    await _loadPersistedLibrary();
  }

  @override
  Future<void> scanCores(String directory) async {
    final dir = Directory(directory);
    dir.createSync(recursive: true);
    if (_handle != null) {
      final cDir = directory.toNativeUtf8();
      try {
        _scanCoresNative(_handle!, cDir);
        _refreshNativeCores();
      } finally {
        malloc.free(cDir);
      }
    } else {
      cores = dir
          .listSync(recursive: true)
          .whereType<File>()
          .where((file) => _isCoreLibrary(file.path))
          .map((file) => CoreEntry(name: _titleCase(p.basenameWithoutExtension(file.path).replaceAll('_libretro', '')), system: 'Libretro', path: file.path))
          .toList();
    }
  }

  @override
  Future<bool> loadCore(CoreEntry core) async {
    var ok = File(core.path).existsSync() || Directory(core.path).existsSync() || _handle == null;
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
    if (!File(game.path).existsSync()) {
      statusMessage = 'Launch failed: ROM file is missing.';
      return false;
    }
    var ok = _handle == null;
    final handle = _handle;
    if (handle != null) {
      final cPath = game.path.toNativeUtf8();
      final preferredCorePath = cores
          .firstWhere((core) => core.name == game.core, orElse: () => cores.isNotEmpty ? cores.first : const CoreEntry(name: '', system: '', path: ''))
          .path;
      final cCore = preferredCorePath.toNativeUtf8();
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
      statusMessage = 'Running ${game.title}';
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
    if (handle == null) {
      await _persistSettings();
      return;
    }
    final cKey = key.toNativeUtf8();
    final cValue = value.toNativeUtf8();
    try {
      _setSettingNative(handle, cKey, cValue);
    } finally {
      malloc.free(cKey);
      malloc.free(cValue);
    }
    await _persistSettings();
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

  Future<Directory> _resolveRetroArchRoot() async {
    if (Platform.isIOS) {
      final docs = await getApplicationDocumentsDirectory();
      return Directory(p.join(docs.path, 'RetroArch'));
    }
    final home = Platform.environment['HOME'];
    if (home != null && home.isNotEmpty) {
      return Directory(p.join(home, '.config', 'retroarch'));
    }
    final support = await getApplicationSupportDirectory();
    return Directory(p.join(support.path, 'RetroArch'));
  }

  void _applyAllSettingsToNative() {
    final handle = _handle;
    if (handle == null) return;
    final base = settings['base_dir']?.toNativeUtf8();
    if (base != null) {
      try { _setBaseDirNative(handle, base); } finally { malloc.free(base); }
    }
    final info = settings['libretro_info_path']?.toNativeUtf8();
    if (info != null) {
      try { _setInfoDirNative(handle, info); } finally { malloc.free(info); }
    }
    for (final entry in settings.entries) {
      final cKey = entry.key.toNativeUtf8();
      final cValue = entry.value.toNativeUtf8();
      try { _setSettingNative(handle, cKey, cValue); } finally { malloc.free(cKey); malloc.free(cValue); }
    }
  }

  void _refreshNativeCores() {
    final handle = _handle;
    if (handle == null) return;
    final count = _coresCountNative(handle);
    final out = malloc<RfCoreInfo>();
    final next = <CoreEntry>[];
    try {
      for (var i = 0; i < count; i++) {
        if (_getCoreInfoNative(handle, i, out)) {
          next.add(CoreEntry(
            name: out.ref.displayName.toDartString(),
            system: out.ref.systemName.toDartString().isEmpty ? 'Libretro' : out.ref.systemName.toDartString(),
            path: out.ref.path.toDartString(),
            loaded: out.ref.displayName.toDartString() == runtime.loadedCore,
          ));
        }
      }
    } finally {
      malloc.free(out);
    }
    cores = next;
  }

  void _refreshNativeGames() {
    final handle = _handle;
    if (handle == null) return;
    final count = _gamesCountNative(handle);
    final out = malloc<RfGameInfo>();
    final next = <GameEntry>[];
    try {
      for (var i = 0; i < count; i++) {
        if (_getGameInfoNative(handle, i, out)) {
          final path = out.ref.path.toDartString();
          final label = out.ref.label.toDartString();
          final ext = p.extension(path).replaceFirst('.', '').toLowerCase();
          final core = _bestCoreForExtension(ext);
          next.add(GameEntry(title: label.isEmpty ? _titleCase(p.basenameWithoutExtension(path)) : label, system: _systemForCore(core), core: core, lastPlayed: '未プレイ', playTime: '0m', path: path, initials: _initials(label.isEmpty ? p.basenameWithoutExtension(path) : label)));
        }
      }
    } finally {
      malloc.free(out);
    }
    for (final game in next.reversed) {
      games.removeWhere((existing) => existing.path == game.path);
      games.insert(0, game);
    }
  }

  Future<File> get _libraryFile async => File(p.join((_retroArchRoot ?? await _resolveRetroArchRoot()).path, 'playlists', 'retrofront-library.json'));

  Future<void> _persistLibrary() async {
    final file = await _libraryFile;
    file.parent.createSync(recursive: true);
    final data = games.map((game) => {'title': game.title, 'system': game.system, 'core': game.core, 'lastPlayed': game.lastPlayed, 'playTime': game.playTime, 'path': game.path, 'initials': game.initials}).toList();
    file.writeAsStringSync(const JsonEncoder.withIndent('  ').convert(data));
  }

  Future<void> _loadPersistedLibrary() async {
    final file = await _libraryFile;
    if (!file.existsSync()) return;
    final decoded = jsonDecode(file.readAsStringSync());
    if (decoded is! List) return;
    for (final item in decoded.whereType<Map>()) {
      final path = item['path']?.toString() ?? '';
      if (path.isEmpty || games.any((game) => game.path == path)) continue;
      games.add(GameEntry(title: item['title']?.toString() ?? _titleCase(p.basenameWithoutExtension(path)), system: item['system']?.toString() ?? 'Unknown', core: item['core']?.toString() ?? '', lastPlayed: item['lastPlayed']?.toString() ?? '未プレイ', playTime: item['playTime']?.toString() ?? '0m', path: path, initials: item['initials']?.toString() ?? _initials(path)));
    }
  }

  Future<void> _loadPersistedSettings() async {
    final file = File(p.join((_retroArchRoot ?? await _resolveRetroArchRoot()).path, 'config', 'retroarch.cfg'));
    if (!file.existsSync()) return;
    for (final line in file.readAsLinesSync()) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#') || !trimmed.contains('=')) continue;
      final parts = trimmed.split('=');
      final key = parts.removeAt(0).trim();
      settings[key] = parts.join('=').trim().replaceAll('"', '');
    }
  }

  Future<void> _persistSettings() async {
    final file = File(p.join((_retroArchRoot ?? await _resolveRetroArchRoot()).path, 'config', 'retroarch.cfg'));
    file.parent.createSync(recursive: true);
    file.writeAsStringSync(settings.entries.map((entry) => '${entry.key} = "${entry.value}"').join('\n'));
  }

  bool _isCoreLibrary(String path) {
    final lower = path.toLowerCase();
    return lower.endsWith('_libretro.so') || lower.endsWith('_libretro.dylib') || lower.endsWith('_libretro.framework');
  }

  static const _fallbackExtensions = {'zip', 'gba', 'gb', 'gbc', 'sfc', 'smc', 'nes', 'cue', 'chd', 'iso', 'bin', 'md', 'gen'};

  String _bestCoreForExtension(String ext) {
    final compatible = cores.where((core) => core.path.toLowerCase().contains(ext) || core.name.toLowerCase().contains(ext));
    if (compatible.isNotEmpty) return compatible.first.name;
    return cores.isNotEmpty ? cores.first.name : '';
  }

  String _systemForCore(String core) {
    if (core.isEmpty || cores.isEmpty) return 'Unknown';
    return cores.firstWhere((item) => item.name == core, orElse: () => cores.first).system;
  }

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
