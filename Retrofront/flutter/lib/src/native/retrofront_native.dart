import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io' show Directory, File, HttpClient, HttpStatus, Platform;
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as p;

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

final class RfLaunchPlan extends ffi.Struct {
  external ffi.Pointer<Utf8> contentPath;
  external ffi.Pointer<Utf8> contentExtension;
  @ffi.Uint32()
  external int decision;
  external ffi.Pointer<Utf8> selectedCorePath;
  @ffi.UintPtr()
  external int candidateCount;
  external ffi.Pointer<Utf8> reason;
}

final class RfAssetInstallReport extends ffi.Struct {
  @ffi.UintPtr()
  external int filesWritten;
  @ffi.UintPtr()
  external int directoriesCreated;
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
  const CoreEntry({required this.name, required this.system, required this.path, this.supportedExtensions = const {}, this.loaded = false});

  final String name;
  final String system;
  final String path;
  final Set<String> supportedExtensions;
  final bool loaded;
}

class LaunchPlanEntry {
  const LaunchPlanEntry({required this.decision, required this.contentExtension, required this.selectedCorePath, required this.candidates, required this.reason});

  final int decision;
  final String contentExtension;
  final String selectedCorePath;
  final List<CoreEntry> candidates;
  final String reason;

  bool get isSelected => decision == 1;
  bool get needsCoreChoice => decision == 2;
}

class FrontendAssetPackageEntry {
  const FrontendAssetPackageEntry({required this.name, required this.destinationSettingKey, required this.label});

  final String name;
  final String destinationSettingKey;
  final String label;
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

class VideoFrame {
  const VideoFrame({required this.width, required this.height, required this.rgba});

  final int width;
  final int height;
  final Uint8List rgba;
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
  Future<LaunchPlanEntry> planContentLaunch(String path, {String preferredCorePath = ''});
  Future<bool> launch(GameEntry game);
  Future<bool> launchPath(String path, {String preferredCorePath = ''});
  Future<bool> runFrame();
  Future<bool> quickSave({int slot = 0});
  Future<bool> quickLoad({int slot = 0});
  Future<bool> reset();
  Future<void> setJoypadButton(int buttonId, bool pressed);
  Future<void> setOverlayTouch(int slot, double x, double y, bool active);
  Future<void> loadOverlay(String path);
  Future<void> setOverlayEnabled(bool enabled);
  Future<List<CoreOptionEntry>> coreOptions();
  Future<bool> setCoreOption(String key, String value);
  Future<void> setSetting(String key, String value);
  Future<List<String>> availableOverlayConfigs();
  Future<int> installFrontendAssetPackage(String name, {bool download = false});
  Future<VideoFrame?> copyVideoFrame();
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

  static const frontendAssetPackages = <FrontendAssetPackageEntry>[
    FrontendAssetPackageEntry(name: 'assets', destinationSettingKey: 'assets_directory', label: 'Menu assets'),
    FrontendAssetPackageEntry(name: 'info', destinationSettingKey: 'libretro_info_path', label: 'Core info'),
    FrontendAssetPackageEntry(name: 'overlays', destinationSettingKey: 'overlay_directory', label: 'Input overlays'),
  ];

  factory RetrofrontNative.create() {
    return RetrofrontNative._(_openLibraryOrNull());
  }

  static ffi.DynamicLibrary? _openLibraryOrNull() {
    final executableDir = p.dirname(Platform.resolvedExecutable);
    final current = Directory.current.path;
    final bundledNativeDir = Platform.environment['RETROFRONT_BUNDLED_NATIVE_DIR'];
    final candidates = <String>[
      if (Platform.isIOS) 'libretrofront_core.dylib',
      if (Platform.isIOS) 'retrofront_core.framework/retrofront_core',
      if (Platform.isIOS) p.join(executableDir, 'Frameworks', 'libretrofront_core.dylib'),
      if (Platform.isIOS) p.join(executableDir, 'Frameworks', 'retrofront_core.framework', 'retrofront_core'),
      if (Platform.isLinux) 'libretrofront_core.so',
      if (Platform.isLinux && bundledNativeDir != null) p.join(bundledNativeDir, 'libretrofront_core.so'),
      if (Platform.isLinux) p.join(current, 'libretrofront_core.so'),
      if (Platform.isLinux) p.join(current, '..', 'target', 'debug', 'libretrofront_core.so'),
      if (Platform.isLinux) p.join(current, '..', 'target', 'release', 'libretrofront_core.so'),
      if (Platform.isLinux) p.join(executableDir, 'libretrofront_core.so'),
      if (Platform.isLinux) p.join(executableDir, 'lib', 'libretrofront_core.so'),
      if (Platform.isLinux) p.join(executableDir, '..', 'lib', 'retrofront', 'native', 'libretrofront_core.so'),
    ];
    for (final candidate in candidates) {
      final library = _openAndValidateLibrary(candidate);
      if (library != null) return library;
    }
    if (Platform.isIOS) {
      try {
        final process = ffi.DynamicLibrary.process();
        process.lookup<ffi.NativeFunction<ffi.Pointer<RfFrontend> Function()>>('rf_frontend_create');
        return process;
      } catch (_) {
        // Statically linked symbols are optional; continue in preview mode.
      }
    }
    return null;
  }

  static ffi.DynamicLibrary? _openAndValidateLibrary(String candidate) {
    try {
      final library = ffi.DynamicLibrary.open(candidate);
      library.lookup<ffi.NativeFunction<ffi.Pointer<RfFrontend> Function()>>('rf_frontend_create');
      return library;
    } catch (_) {
      // Try the next packaged/library-path location. The demo fallback keeps
      // the UI testable when the native core has not been bundled yet.
      return null;
    }
  }

  final ffi.DynamicLibrary? _library;
  ffi.Pointer<RfFrontend>? _handle;

  late final ffi.Pointer<RfFrontend> Function() _create;
  late final void Function(ffi.Pointer<RfFrontend>) _destroy;
  late final int Function(ffi.Pointer<RfFrontend>) _state;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _loadCore;
  late final bool Function(ffi.Pointer<RfFrontend>, int) _setGfxBackendNative;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _launchContent;
  late final bool Function(ffi.Pointer<RfFrontend>) _runFrame;
  late final bool Function(ffi.Pointer<RfFrontend>) _reset;
  late final bool Function(ffi.Pointer<RfFrontend>, int) _saveState;
  late final bool Function(ffi.Pointer<RfFrontend>, int) _loadState;
  late final bool Function(ffi.Pointer<RfFrontend>, int, bool) _setJoypadButton;
  late final bool Function(ffi.Pointer<RfFrontend>, int, double, double, bool) _setOverlayTouchNative;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _loadOverlay;
  late final void Function(ffi.Pointer<RfFrontend>, bool) _setOverlayEnabled;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfOverlayInfo>) _overlayInfo;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfVideoFrameInfo>) _videoFrameInfo;
  late final int Function(ffi.Pointer<RfFrontend>, ffi.Pointer<ffi.Uint8>, int) _copyFrame;
  late final int Function(ffi.Pointer<RfFrontend>) _optionsCount;
  late final bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreOption>) _getOption;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _setOption;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _setSettingNative;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<RfLaunchPlan>) _planContentLaunchNative;
  late final int Function(ffi.Pointer<RfFrontend>) _launchCandidateCountNative;
  late final bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreInfo>) _getLaunchCandidateNative;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<RfAssetInstallReport>) _installAssetsZipNative;
  late final bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _setBaseDirNative;
  late final void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _setInfoDirNative;
  late final void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>) _scanCoresNative;
  late final void Function(ffi.Pointer<RfFrontend>) _scanConfiguredCoresNative;
  late final int Function(ffi.Pointer<RfFrontend>) _coresCountNative;
  late final bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreInfo>) _getCoreInfoNative;
  late final ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>) _allExtensionsNative;
  late final void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>) _scanGamesNative;
  late final int Function(ffi.Pointer<RfFrontend>) _gamesCountNative;
  late final bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfGameInfo>) _getGameInfoNative;
  late final ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>) _lastErrorNative;

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
    _setGfxBackendNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Uint32), bool Function(ffi.Pointer<RfFrontend>, int)>('rf_frontend_set_gfx_backend');
    _launchContent = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_launch_content');
    _runFrame = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>), bool Function(ffi.Pointer<RfFrontend>)>('rf_frontend_run_frame');
    _reset = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>), bool Function(ffi.Pointer<RfFrontend>)>('rf_frontend_reset');
    _saveState = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Uint32), bool Function(ffi.Pointer<RfFrontend>, int)>('rf_frontend_save_state');
    _loadState = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Uint32), bool Function(ffi.Pointer<RfFrontend>, int)>('rf_frontend_load_state');
    _setJoypadButton = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Uint32, ffi.Bool), bool Function(ffi.Pointer<RfFrontend>, int, bool)>('rf_frontend_set_joypad_button');
    _setOverlayTouchNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Float, ffi.Float, ffi.Bool), bool Function(ffi.Pointer<RfFrontend>, int, double, double, bool)>('rf_frontend_set_overlay_touch');
    _loadOverlay = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_load_overlay');
    _setOverlayEnabled = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Bool), void Function(ffi.Pointer<RfFrontend>, bool)>('rf_frontend_set_overlay_enabled');
    _overlayInfo = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfOverlayInfo>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfOverlayInfo>)>('rf_frontend_overlay_info');
    _videoFrameInfo = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfVideoFrameInfo>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<RfVideoFrameInfo>)>('rf_frontend_video_frame_info');
    _copyFrame = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>, ffi.Pointer<ffi.Uint8>, ffi.UintPtr), int Function(ffi.Pointer<RfFrontend>, ffi.Pointer<ffi.Uint8>, int)>('rf_frontend_copy_video_frame_rgba');
    _optionsCount = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_options_count');
    _getOption = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Pointer<RfCoreOption>), bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreOption>)>('rf_frontend_get_option');
    _setOption = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_set_option');
    _setSettingNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_set_setting');
    _planContentLaunchNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<RfLaunchPlan>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<RfLaunchPlan>)>('rf_frontend_plan_content_launch');
    _launchCandidateCountNative = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_launch_candidate_count');
    _getLaunchCandidateNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Pointer<RfCoreInfo>), bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreInfo>)>('rf_frontend_get_launch_candidate');
    _installAssetsZipNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<RfAssetInstallReport>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>, ffi.Pointer<RfAssetInstallReport>)>('rf_frontend_install_assets_zip');
    _setBaseDirNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), bool Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_set_base_dir');
    _setInfoDirNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_set_info_dir');
    _scanCoresNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>), void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>)>('rf_frontend_scan_cores');
    _scanConfiguredCoresNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>), void Function(ffi.Pointer<RfFrontend>)>('rf_frontend_scan_configured_cores');
    _coresCountNative = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_cores_count');
    _getCoreInfoNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Pointer<RfCoreInfo>), bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfCoreInfo>)>('rf_frontend_get_core_info');
    _allExtensionsNative = lib.lookupFunction<ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>), ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>)>('rf_frontend_all_extensions');
    _scanGamesNative = lib.lookupFunction<ffi.Void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>), void Function(ffi.Pointer<RfFrontend>, ffi.Pointer<Utf8>, ffi.Pointer<Utf8>)>('rf_frontend_scan_games');
    _gamesCountNative = lib.lookupFunction<ffi.UintPtr Function(ffi.Pointer<RfFrontend>), int Function(ffi.Pointer<RfFrontend>)>('rf_frontend_games_count');
    _getGameInfoNative = lib.lookupFunction<ffi.Bool Function(ffi.Pointer<RfFrontend>, ffi.UintPtr, ffi.Pointer<RfGameInfo>), bool Function(ffi.Pointer<RfFrontend>, int, ffi.Pointer<RfGameInfo>)>('rf_frontend_get_game_info');
    _lastErrorNative = lib.lookupFunction<ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>), ffi.Pointer<Utf8> Function(ffi.Pointer<RfFrontend>)>('rf_frontend_last_error');
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
      'input_overlay_enable': 'true',
      'input_overlay_opacity': '0.70',
      'input_haptic_feedback': 'true',
      'audio_enable': 'true',
      'audio_sync': 'true',
      'audio_latency_ms': '64',
      'video_driver': Platform.isIOS ? 'metal' : 'software',
      'video_bgfx_renderer': Platform.isIOS ? 'metal' : 'glcore',
      'video_scale_mode': 'keep_aspect',
      'video_filter_mode': 'nearest',
      'video_vsync': 'true',
      'library_sort_mode': 'name_ascending',
      'library_show_core_badges': 'true',
      'library_show_file_details': 'true',
      'library_auto_scan_on_launch': 'true',
      'screenshot_directory': p.join(root.path, 'screenshots'),
    };
    settings.addAll(dirs);
    for (final dir in dirs.entries.where((entry) => _directorySettingKeys.contains(entry.key))) {
      Directory(dir.value).createSync(recursive: true);
    }
    Directory(p.dirname(settings['input_overlay']!)).createSync(recursive: true);
    _ensureDefaultOverlayConfig();
    await _loadPersistedSettings();
    _normalizeConfiguredDirectories(root);
    _ensureDefaultOverlayConfig();
    _applyAllSettingsToNative();
    await _installBundledAssetsIfNeeded();
    _applyAllSettingsToNative();
    await _scanBundledCoreDirectories();
    await scanCores(settings['libretro_directory']!);
    _scanConfiguredCores();
    if ((settings['input_overlay'] ?? '').isNotEmpty) {
      await loadOverlay(settings['input_overlay']!);
    }
    await scanRoms(settings['content_directory']!);
    statusMessage = _handle == null
        ? 'Native core library not found; UI is running in preview mode.'
        : 'RetroArch-compatible storage initialized (${cores.length} cores, ${games.length} ROMs).';
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
    _sortGames();
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
      final found = dir
          .listSync(recursive: true)
          .where((entity) => entity is File || (entity is Directory && entity.path.toLowerCase().endsWith('.framework')))
          .where((entity) => _isCoreLibrary(entity.path))
          .map((entity) => _fallbackCoreEntryForPath(entity.path))
          .toList();
      final byPath = {for (final core in [...cores, ...found]) core.path: core};
      cores = byPath.values.toList()..sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
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
      cores = [for (final item in cores) CoreEntry(name: item.name, system: item.system, path: item.path, supportedExtensions: item.supportedExtensions, loaded: item.name == core.name)];
      runtime = runtime.copyWith(loadedCore: core.name);
      statusMessage = 'Loaded core: ${core.name}';
    } else {
      statusMessage = 'Core load failed: ${_lastError()}';
    }
    return ok;
  }

  @override
  Future<LaunchPlanEntry> planContentLaunch(String path, {String preferredCorePath = ''}) async {
    final handle = _handle;
    if (handle == null) {
      final ext = p.extension(path).replaceFirst('.', '').toLowerCase();
      final compatible = cores.where((core) => core.supportedExtensions.contains(ext) || core.name.toLowerCase().contains(ext) || core.path.toLowerCase().contains(ext)).toList();
      return LaunchPlanEntry(
        decision: compatible.length > 1 ? 2 : compatible.isEmpty ? 0 : 1,
        contentExtension: ext,
        selectedCorePath: compatible.length == 1 ? compatible.first.path : '',
        candidates: compatible,
        reason: compatible.isEmpty ? 'No compatible core found' : 'Preview launch plan',
      );
    }
    final cPath = path.toNativeUtf8();
    final cPreferred = preferredCorePath.toNativeUtf8();
    final out = malloc<RfLaunchPlan>();
    try {
      if (!_planContentLaunchNative(handle, cPath, cPreferred, out)) {
        return const LaunchPlanEntry(decision: 0, contentExtension: '', selectedCorePath: '', candidates: [], reason: 'Launch planning failed');
      }
      final candidates = <CoreEntry>[];
      final rawCore = malloc<RfCoreInfo>();
      try {
        final count = _launchCandidateCountNative(handle);
        for (var i = 0; i < count; i++) {
          if (_getLaunchCandidateNative(handle, i, rawCore)) {
            candidates.add(_coreEntryFromNative(rawCore.ref));
          }
        }
      } finally {
        malloc.free(rawCore);
      }
      return LaunchPlanEntry(
        decision: out.ref.decision,
        contentExtension: out.ref.contentExtension.toDartString(),
        selectedCorePath: out.ref.selectedCorePath.toDartString(),
        candidates: candidates,
        reason: out.ref.reason.toDartString(),
      );
    } finally {
      malloc.free(cPath);
      malloc.free(cPreferred);
      malloc.free(out);
    }
  }

  @override
  Future<bool> launch(GameEntry game) async {
    final preferred = _corePathForGame(game);
    final ok = await launchPath(game.path, preferredCorePath: preferred);
    if (ok) {
      final loadedCore = _coreNameForPath(preferred);
      runtime = runtime.copyWith(
        loadedGame: game.title,
        loadedCore: loadedCore.isEmpty ? game.core : loadedCore,
        running: true,
      );
    }
    return ok;
  }

  @override
  Future<bool> launchPath(String path, {String preferredCorePath = ''}) async {
    if (!File(path).existsSync()) {
      statusMessage = 'Launch failed: ROM file is missing.';
      return false;
    }

    final handle = _handle;
    var selectedCorePath = preferredCorePath;
    var ok = handle == null;

    if (handle != null) {
      if (selectedCorePath.isEmpty) {
        final ext = p.extension(path).replaceFirst('.', '').toLowerCase();
        final plan = await planContentLaunch(path, preferredCorePath: settings['content_core_$ext'] ?? '');
        if (!plan.isSelected) {
          statusMessage = plan.needsCoreChoice
              ? 'Launch failed: select a core for .$ext.'
              : (plan.reason.isEmpty ? 'Launch failed: no compatible core found for .$ext.' : plan.reason);
          return false;
        }
        selectedCorePath = plan.selectedCorePath.isNotEmpty
            ? plan.selectedCorePath
            : (plan.candidates.isNotEmpty ? plan.candidates.first.path : '');
      }
      if (selectedCorePath.isEmpty) {
        statusMessage = 'Launch failed: no compatible core was selected.';
        return false;
      }

      final cPath = path.toNativeUtf8();
      final cCore = selectedCorePath.toNativeUtf8();
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
      runtime = runtime.copyWith(
        loadedGame: _titleCase(p.basenameWithoutExtension(path)),
        loadedCore: _coreNameForPath(selectedCorePath),
        running: true,
      );
      statusMessage = 'Loaded game: ${p.basename(path)}';
    } else {
      statusMessage = 'Launch failed: ${_lastError()}';
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
  Future<void> setOverlayTouch(int slot, double x, double y, bool active) async {
    if (_handle != null) _setOverlayTouchNative(_handle!, slot, x, y, active);
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
    await _persistSettings();
  }

  @override
  Future<void> setOverlayEnabled(bool enabled) async {
    if (_handle != null) _setOverlayEnabled(_handle!, enabled);
    runtime = runtime.copyWith(overlayEnabled: enabled);
    settings['input_overlay_enable'] = enabled.toString();
    await _persistSettings();
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
    settings['core_option_$key'] = value;
    if (handle == null) {
      await _persistSettings();
      return true;
    }
    final cKey = key.toNativeUtf8();
    final cValue = value.toNativeUtf8();
    try {
      final ok = _setOption(handle, cKey, cValue);
      if (ok) statusMessage = 'Core option updated: $key';
      await _persistSettings();
      return ok;
    } finally {
      malloc.free(cKey);
      malloc.free(cValue);
    }
  }

  @override
  Future<void> setSetting(String key, String value) async {
    settings[key] = value;
    if (key == 'video_driver') {
      settings['video_bgfx_renderer'] = value == 'software' ? 'metal' : value;
      _applyRendererSetting();
    }
    if (key == 'library_sort_mode') _sortGames();
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
  Future<List<String>> availableOverlayConfigs() async {
    final roots = <String>{
      if ((settings['overlay_directory'] ?? '').isNotEmpty) settings['overlay_directory']!,
      p.join((_retroArchRoot ?? await _resolveRetroArchRoot()).path, 'overlays'),
    };
    final paths = <String>[];
    for (final root in roots) {
      final dir = Directory(root);
      if (!dir.existsSync()) continue;
      for (final file in dir.listSync(recursive: true).whereType<File>()) {
        if (p.extension(file.path).toLowerCase() == '.cfg') paths.add(file.path);
      }
    }
    paths.sort((a, b) {
      final ar = _overlayRelativeSortKey(a);
      final br = _overlayRelativeSortKey(b);
      final ag = ar.contains('gamepads/');
      final bg = br.contains('gamepads/');
      if (ag != bg) return ag ? -1 : 1;
      return _overlayDisplayLabel(a).toLowerCase().compareTo(_overlayDisplayLabel(b).toLowerCase());
    });
    return paths;
  }

  @override
  Future<int> installFrontendAssetPackage(String name, {bool download = false}) async {
    final package = frontendAssetPackages.firstWhere(
      (entry) => entry.name == name,
      orElse: () => throw ArgumentError('Unknown frontend asset package: $name'),
    );
    final destination = settings[package.destinationSettingKey];
    if (destination == null || destination.isEmpty) {
      statusMessage = 'Install failed: destination is not configured for ${package.name}.zip';
      return 0;
    }
    File? zipFile;
    try {
      zipFile = download ? await _downloadFrontendAssetPackage(package.name) : _findBundledAssetZip(package.name);
    } catch (error) {
      statusMessage = 'Download failed for ${package.name}.zip: $error';
      return 0;
    }
    if (zipFile == null || !zipFile.existsSync()) {
      statusMessage = 'Install failed: ${package.name}.zip was not found.';
      return 0;
    }
    Directory(destination).createSync(recursive: true);
    final handle = _handle;
    int filesWritten;
    if (handle != null) {
      final cZip = zipFile.path.toNativeUtf8();
      final cDestination = destination.toNativeUtf8();
      final report = malloc<RfAssetInstallReport>();
      try {
        final ok = _installAssetsZipNative(handle, cZip, cDestination, report);
        if (!ok) {
          statusMessage = 'Install failed: ${_lastError()}';
          return 0;
        }
        filesWritten = report.ref.filesWritten;
      } finally {
        malloc.free(cZip);
        malloc.free(cDestination);
        malloc.free(report);
      }
    } else {
      try {
        filesWritten = await _extractFrontendAssetZip(zipFile, Directory(destination));
      } catch (error) {
        statusMessage = 'Install failed: could not extract ${package.name}.zip: $error';
        return 0;
      }
    }
    if (package.name == 'info') {
      _applyAllSettingsToNative();
      await _scanBundledCoreDirectories();
      await scanCores(settings['libretro_directory'] ?? '');
      _scanConfiguredCores();
    }
    if (package.name == 'overlays') {
      _ensureDefaultOverlayConfig();
      if ((settings['input_overlay'] ?? '').isNotEmpty) {
        await loadOverlay(settings['input_overlay']!);
      }
    }
    statusMessage = '${package.name}.zip installed: $filesWritten files';
    return filesWritten;
  }

  @override
  Future<VideoFrame?> copyVideoFrame() async {
    final handle = _handle;
    if (handle == null) return VideoFrame(width: 320, height: 180, rgba: _demoFrame(runtime.frameNumber));
    final info = malloc<RfVideoFrameInfo>();
    try {
      if (!_videoFrameInfo(handle, info) || info.ref.rgbaLen == 0 || info.ref.width == 0 || info.ref.height == 0) return null;
      final buffer = malloc<ffi.Uint8>(info.ref.rgbaLen);
      try {
        final copied = _copyFrame(handle, buffer, info.ref.rgbaLen);
        return VideoFrame(width: info.ref.width, height: info.ref.height, rgba: Uint8List.fromList(buffer.asTypedList(copied)));
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


  static const _directorySettingKeys = {
    'base_dir',
    'config_directory',
    'assets_directory',
    'menu_assets_directory',
    'libretro_info_path',
    'overlay_directory',
    'core_assets_directory',
    'content_directory',
    'menu_content_directory',
    'savefile_directory',
    'savestate_directory',
    'system_directory',
    'playlist_directory',
    'cache_directory',
    'libretro_directory',
    'screenshot_directory',
  };

  void _normalizeConfiguredDirectories(Directory root) {
    for (final key in _directorySettingKeys) {
      final value = settings[key];
      if (value == null || value.isEmpty) continue;
      Directory(value).createSync(recursive: true);
    }
    final coreDir = settings['libretro_directory'];
    if (coreDir == null || coreDir.isEmpty || !Directory(coreDir).existsSync()) {
      settings['libretro_directory'] = p.join(root.path, 'cores');
      Directory(settings['libretro_directory']!).createSync(recursive: true);
    }
  }

  String _lastError() {
    final handle = _handle;
    if (handle == null) return 'native core unavailable';
    final ptr = _lastErrorNative(handle);
    if (ptr == ffi.nullptr) return 'unknown native error';
    final value = ptr.toDartString();
    return value.isEmpty ? 'unknown native error' : value;
  }

  String _corePathForGame(GameEntry game) {
    if (game.core.isEmpty) return '';
    if (File(game.core).existsSync() || Directory(game.core).existsSync()) return game.core;
    return cores.firstWhere((core) => core.name == game.core, orElse: () => const CoreEntry(name: '', system: '', path: '')).path;
  }

  String _coreNameForPath(String path) {
    if (path.isEmpty) return '';
    return cores.firstWhere((core) => core.path == path, orElse: () => const CoreEntry(name: '', system: '', path: '')).name;
  }

  void _scanConfiguredCores() {
    final handle = _handle;
    if (handle == null) return;
    _scanConfiguredCoresNative(handle);
    _refreshNativeCores();
  }

  Future<void> _scanBundledCoreDirectories() async {
    for (final directory in _bundledCoreDirectories()) {
      if (Directory(directory).existsSync()) {
        await scanCores(directory);
      }
    }
  }

  List<String> _bundledCoreDirectories() {
    final current = Directory.current.path;
    final executableDir = p.dirname(Platform.resolvedExecutable);
    final root = _retroArchRoot?.path;
    final envCoreDir = Platform.environment['RETROFRONT_BUNDLED_CORE_DIR'];
    return <String>{
      if (envCoreDir != null && envCoreDir.isNotEmpty) envCoreDir,
      if (Platform.isIOS) p.join(executableDir, 'dylibs'),
      if (Platform.isIOS) p.join(executableDir, 'Resources', 'dylibs'),
      if (Platform.isIOS) p.join(executableDir, 'Frameworks'),
      if (Platform.isIOS) executableDir,
      if (Platform.isIOS) p.join(current, 'archifacts', 'ios'),
      if (Platform.isIOS) p.join(current, '..', 'archifacts', 'ios'),
      if (Platform.isLinux) p.join(current, 'archifacts', 'linux'),
      if (Platform.isLinux) p.join(current, '..', 'archifacts', 'linux'),
      if (Platform.isLinux) p.join(executableDir, 'cores'),
      if (Platform.isLinux) p.join(executableDir, 'lib', 'cores'),
      if (root != null) p.join(root, 'cores'),
      if (root != null) p.join(root, 'Cores'),
    }.toList();
  }

  Future<void> _installBundledAssetsIfNeeded() async {
    final infoProbe = File(p.join(settings['libretro_info_path'] ?? '', 'mgba_libretro.info'));
    if (infoProbe.existsSync()) return;
    var installed = 0;
    for (final package in frontendAssetPackages) {
      if (_findBundledAssetZip(package.name) == null) continue;
      installed += await installFrontendAssetPackage(package.name);
    }
    if (installed > 0) {
      statusMessage = 'Installed bundled frontend assets: $installed files';
    }
  }

  void _ensureDefaultOverlayConfig() {
    final overlayPath = settings['input_overlay'];
    if (overlayPath == null || overlayPath.isEmpty) return;
    final file = File(overlayPath);
    if (file.existsSync()) return;
    file.parent.createSync(recursive: true);
    file.writeAsStringSync("""overlays = 1
overlay0_name = "retropad"
overlay0_full_screen = true
overlay0_normalized = true
overlay0_descs = 11
overlay0_desc0 = "up,0.16,0.70,rect,0.055,0.055"
overlay0_desc1 = "down,0.16,0.88,rect,0.055,0.055"
overlay0_desc2 = "left,0.07,0.79,rect,0.055,0.055"
overlay0_desc3 = "right,0.25,0.79,rect,0.055,0.055"
overlay0_desc4 = "b,0.78,0.83,radial,0.06,0.06"
overlay0_desc5 = "a,0.88,0.72,radial,0.06,0.06"
overlay0_desc6 = "y,0.68,0.72,radial,0.052,0.052"
overlay0_desc7 = "x,0.78,0.61,radial,0.052,0.052"
overlay0_desc8 = "select,0.42,0.92,rect,0.05,0.035"
overlay0_desc9 = "start,0.58,0.92,rect,0.05,0.035"
overlay0_desc10 = "menu_toggle,0.08,0.13,rect,0.06,0.04"
""");
  }

  File? _findBundledAssetZip(String name) {
    final current = Directory.current.path;
    final executableDir = p.dirname(Platform.resolvedExecutable);
    final envAssetDir = Platform.environment['RETROFRONT_BUNDLED_ASSET_DIR'];
    final candidates = <String>[
      if (envAssetDir != null && envAssetDir.isNotEmpty) p.join(envAssetDir, '$name.zip'),
      p.join(current, 'apps', 'iOS', 'Resources', '$name.zip'),
      p.join(current, '..', 'apps', 'iOS', 'Resources', '$name.zip'),
      p.join(current, 'assets', 'retroarch', '$name.zip'),
      p.join(executableDir, '$name.zip'),
      p.join(executableDir, 'Resources', '$name.zip'),
      p.join(executableDir, 'assets', 'retroarch', '$name.zip'),
    ];
    for (final candidate in candidates) {
      final file = File(candidate);
      if (file.existsSync()) return file;
    }
    return null;
  }

  Future<File> _downloadFrontendAssetPackage(String name) async {
    final cache = Directory(p.join((_retroArchRoot ?? await _resolveRetroArchRoot()).path, 'cache', 'frontend-assets'));
    cache.createSync(recursive: true);
    final destination = File(p.join(cache.path, '$name.zip'));
    final temporary = File(p.join(cache.path, '$name.zip.download'));
    final uri = Uri.parse('https://buildbot.libretro.com/assets/frontend/$name.zip');
    final client = HttpClient()..connectionTimeout = const Duration(seconds: 20);
    try {
      final request = await client.getUrl(uri);
      request.headers.set('Accept', 'application/zip, application/octet-stream, */*');
      final response = await request.close();
      if (response.statusCode != HttpStatus.ok) {
        await response.drain();
        throw StateError('HTTP ${response.statusCode} while downloading $uri');
      }
      if (temporary.existsSync()) temporary.deleteSync();
      await response.pipe(temporary.openWrite());
      if (temporary.lengthSync() == 0) {
        temporary.deleteSync();
        throw StateError('downloaded $name.zip is empty');
      }
      if (destination.existsSync()) destination.deleteSync();
      temporary.renameSync(destination.path);
      return destination;
    } finally {
      client.close(force: true);
    }
  }

  Future<int> _extractFrontendAssetZip(File zipFile, Directory destination) async {
    destination.createSync(recursive: true);
    final archive = ZipDecoder().decodeBytes(await zipFile.readAsBytes(), verify: true);
    var filesWritten = 0;
    for (final entry in archive) {
      final safePath = _safeZipPath(entry.name);
      if (safePath == null || _isMacOSZipMetadataPath(safePath)) continue;
      final normalized = _normalizeFrontendAssetZipPath(safePath);
      if (normalized.isEmpty) continue;
      final outputPath = p.joinAll([destination.path, ...normalized.split('/')]);
      if (entry.isFile) {
        final output = File(outputPath);
        if (output.existsSync()) output.deleteSync();
        output.parent.createSync(recursive: true);
        final content = entry.content;
        final bytes = content is List<int> ? content : List<int>.from(content as Iterable<int>);
        output.writeAsBytesSync(bytes);
        filesWritten++;
      } else {
        Directory(outputPath).createSync(recursive: true);
      }
    }
    return filesWritten;
  }

  Future<Directory> _resolveRetroArchRoot() async {
    final home = Platform.environment['HOME'];
    if (Platform.isIOS && home != null && home.isNotEmpty) {
      return Directory(p.join(home, 'Documents', 'RetroArch'));
    }
    if (home != null && home.isNotEmpty) {
      return Directory(p.join(home, '.config', 'retroarch'));
    }
    return Directory(p.join(Directory.systemTemp.path, 'RetroArch'));
  }

  void _applyAllSettingsToNative() {
    final handle = _handle;
    if (handle == null) return;
    final base = settings['base_dir']?.toNativeUtf8();
    if (base != null) {
      try {
        _setBaseDirNative(handle, base);
      } finally {
        malloc.free(base);
      }
    }
    _applyRendererSetting();
    final info = settings['libretro_info_path']?.toNativeUtf8();
    if (info != null) {
      try {
        _setInfoDirNative(handle, info);
      } finally {
        malloc.free(info);
      }
    }
    for (final entry in settings.entries) {
      final cKey = entry.key.toNativeUtf8();
      final cValue = entry.value.toNativeUtf8();
      try { _setSettingNative(handle, cKey, cValue); } finally { malloc.free(cKey); malloc.free(cValue); }
    }
  }


  void _applyRendererSetting() {
    final handle = _handle;
    if (handle == null) return;
    final driver = settings['video_driver'] ?? (Platform.isIOS ? 'metal' : 'software');
    _setGfxBackendNative(handle, driver == 'software' ? 0 : 1);
  }

  CoreEntry _coreEntryFromNative(RfCoreInfo info) {
    final extensions = info.supportedExtensions
        .toDartString()
        .split('|')
        .map((ext) => ext.trim().replaceFirst('.', '').toLowerCase())
        .where((ext) => ext.isNotEmpty)
        .toSet();
    final displayName = info.displayName.toDartString();
    final systemName = info.systemName.toDartString();
    return CoreEntry(
      name: displayName,
      system: systemName.isEmpty ? 'Libretro' : systemName,
      path: info.path.toDartString(),
      supportedExtensions: extensions,
      loaded: displayName == runtime.loadedCore,
    );
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
          next.add(_coreEntryFromNative(out.ref));
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


  void _sortGames() {
    switch (settings['library_sort_mode']) {
      case 'extension':
        games.sort((a, b) {
          final ext = p.extension(a.path).toLowerCase().compareTo(p.extension(b.path).toLowerCase());
          return ext == 0 ? a.title.toLowerCase().compareTo(b.title.toLowerCase()) : ext;
        });
        break;
      case 'name_descending':
        games.sort((a, b) => b.title.toLowerCase().compareTo(a.title.toLowerCase()));
        break;
      default:
        games.sort((a, b) => a.title.toLowerCase().compareTo(b.title.toLowerCase()));
        break;
    }
  }

  CoreEntry _fallbackCoreEntryForPath(String path) {
    final info = _fallbackInfoForCorePath(path);
    final stem = p.basenameWithoutExtension(path);
    final name = info['display_name']?.isNotEmpty == true
        ? info['display_name']!
        : _titleCase(stem.replaceAll('_libretro', '').replaceAll('.libretro', '').replaceAll('_ios', ''));
    final system = info['systemname'] ?? info['system_name'] ?? 'Libretro';
    final extensions = (info['supported_extensions'] ?? '')
        .split('|')
        .map((ext) => ext.trim().replaceFirst('.', '').toLowerCase())
        .where((ext) => ext.isNotEmpty)
        .toSet();
    return CoreEntry(name: name, system: system.isEmpty ? 'Libretro' : system, path: path, supportedExtensions: extensions);
  }

  Map<String, String> _fallbackInfoForCorePath(String corePath) {
    final infoRoot = settings['libretro_info_path'];
    if (infoRoot == null || infoRoot.isEmpty) return const {};
    for (final dir in _fallbackInfoSearchDirectories(infoRoot)) {
      for (final candidate in _fallbackInfoNameCandidates(corePath)) {
        final file = File(p.join(dir, '$candidate.info'));
        if (file.existsSync()) return _parseInfoFile(file);
      }
    }
    return const {};
  }

  List<String> _fallbackInfoSearchDirectories(String infoRoot) {
    final parent = p.dirname(infoRoot);
    return <String>{
      infoRoot,
      p.join(infoRoot, 'info'),
      p.join(infoRoot, 'assets', 'info'),
      p.join(parent, 'assets', 'info'),
    }.toList();
  }

  List<String> _fallbackInfoNameCandidates(String corePath) {
    final stems = <String>{p.basenameWithoutExtension(corePath)};
    if (corePath.toLowerCase().endsWith('.framework')) {
      stems.add(p.basenameWithoutExtension(corePath));
    }
    final bases = <String>{};
    for (final stem in stems) {
      bases.add(stem);
      if (stem.startsWith('lib') && stem.length > 3) bases.add(stem.substring(3));
      bases.add(stem.replaceAll('-', '_'));
      bases.add(stem.replaceAll('.', '_'));
    }
    for (final base in [...bases]) {
      for (final suffix in const ['_ios', '_macos', '_android']) {
        if (base.endsWith(suffix)) {
          final stripped = base.substring(0, base.length - suffix.length);
          bases.add(stripped);
          if (stripped.endsWith('_libretro')) bases.add(stripped.substring(0, stripped.length - '_libretro'.length));
          if (stripped.endsWith('.libretro')) bases.add(stripped.substring(0, stripped.length - '.libretro'.length));
        }
      }
      if (base.endsWith('_libretro')) bases.add(base.substring(0, base.length - '_libretro'.length));
      if (base.endsWith('.libretro')) bases.add(base.substring(0, base.length - '.libretro'.length));
    }
    final candidates = <String>{};
    for (final base in bases.where((base) => base.isNotEmpty)) {
      final lower = base.toLowerCase();
      for (final variant in <String>{
        base,
        lower,
        lower.replaceAll(RegExp(r'[-.]'), '_'),
        lower.replaceAll(RegExp(r'[-_]'), '.'),
        lower.replaceAll(RegExp(r'[_.]'), '-'),
      }) {
        candidates.add(variant);
        if (!variant.endsWith('_libretro') && !variant.endsWith('.libretro') && !variant.endsWith('-libretro')) {
          candidates.add('${variant}_libretro');
        }
      }
    }
    return candidates.toList();
  }

  Map<String, String> _parseInfoFile(File file) {
    final values = <String, String>{};
    for (final line in file.readAsLinesSync()) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#') || !trimmed.contains('=')) continue;
      final index = trimmed.indexOf('=');
      final key = trimmed.substring(0, index).trim();
      var value = trimmed.substring(index + 1).trim();
      if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) {
        value = value.substring(1, value.length - 1);
      }
      values[key] = value;
    }
    return values;
  }

  bool _isCoreLibrary(String path) {
    final lower = path.toLowerCase();
    final name = p.basename(lower);
    if (name == 'foundation.dylib' || name == 'libretrofront_core.dylib') {
      return false;
    }
    if (lower.endsWith('.framework')) {
      return name.contains('libretro');
    }
    if (!(lower.endsWith('.so') || lower.endsWith('.dylib') || lower.endsWith('.dll'))) {
      return false;
    }
    return name.contains('_libretro') || name.contains('.libretro');
  }

  String? _safeZipPath(String name) {
    final normalized = name.replaceAll('\\', '/');
    final parts = normalized.split('/').where((part) => part.isNotEmpty && part != '.');
    final safe = <String>[];
    for (final part in parts) {
      if (part == '..' || part.contains(':')) return null;
      safe.add(part);
    }
    return safe.isEmpty ? null : safe.join('/');
  }

  String _normalizeFrontendAssetZipPath(String path) {
    final parts = path.split('/').where((part) => part.isNotEmpty).toList();
    if (parts.isEmpty) return '';
    if (parts.first == 'assets') {
      while (parts.isNotEmpty && parts.first == 'assets') {
        parts.removeAt(0);
      }
      return parts.join('/');
    }
    if (parts.first == 'info' || parts.first == 'overlays') {
      parts.removeAt(0);
      return parts.join('/');
    }
    return parts.join('/');
  }

  bool _isMacOSZipMetadataPath(String path) => path.split('/').any((part) => part == '__MACOSX' || part.startsWith('._'));

  static const _fallbackExtensions = {'zip', 'gba', 'gb', 'gbc', 'sfc', 'smc', 'nes', 'cue', 'chd', 'iso', 'bin', 'md', 'gen'};

  String _bestCoreForExtension(String ext) {
    final normalized = ext.trim().replaceFirst('.', '').toLowerCase();
    final compatible = cores.where((core) => core.supportedExtensions.contains(normalized));
    if (compatible.isNotEmpty) return compatible.first.name;
    final nameCompatible = cores.where((core) => core.path.toLowerCase().contains(normalized) || core.name.toLowerCase().contains(normalized));
    if (nameCompatible.isNotEmpty) return nameCompatible.first.name;
    return cores.isNotEmpty ? cores.first.name : '';
  }

  String _systemForCore(String core) {
    if (core.isEmpty || cores.isEmpty) return 'Unknown';
    return cores.firstWhere((item) => item.name == core, orElse: () => cores.first).system;
  }

  String _overlayRelativeSortKey(String path) {
    final overlayDir = settings['overlay_directory'];
    if (overlayDir == null || overlayDir.isEmpty) return path.replaceAll('\\', '/');
    return path.replaceFirst(RegExp('^${RegExp.escape(overlayDir)}(?:/|\\\\)?'), '').replaceAll('\\', '/');
  }

  String _overlayDisplayLabel(String path) {
    final fileLabel = p.basenameWithoutExtension(path).replaceAll('_', ' ').replaceAll('-', ' ');
    return _overlayRelativeSortKey(path).contains('gamepads/') ? fileLabel : 'Other: $fileLabel';
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
