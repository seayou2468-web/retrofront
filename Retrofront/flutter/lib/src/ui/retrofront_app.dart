import 'dart:async';
import 'dart:io' show Platform;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../native/retrofront_native.dart';

const _bg = Color(0xFF050B14);
const _panel = Color(0xCC081321);
const _panel2 = Color(0xDD0B1728);
const _line = Color(0xFF1C2B40);
const _muted = Color(0xFF8C9AB1);
const _text = Color(0xFFEAF1FF);
const _accent = Color(0xFF7B61FF);
const _cyan = Color(0xFF1FD7F5);

class RetrofrontApp extends StatefulWidget {
  const RetrofrontApp({super.key, required this.frontend});

  final RetrofrontFrontend frontend;

  @override
  State<RetrofrontApp> createState() => _RetrofrontAppState();
}

class _RetrofrontAppState extends State<RetrofrontApp> {
  Timer? _frameTimer;
  int _tab = 0;
  String _section = 'ライブラリ';
  GameEntry? _selectedGame;
  CoreEntry? _selectedCore;
  List<CoreOptionEntry> _coreOptions = const [];
  List<String> _overlayConfigs = const [];

  @override
  void initState() {
    super.initState();
    _selectedGame = widget.frontend.games.isNotEmpty ? widget.frontend.games.first : null;
    _selectedCore = widget.frontend.cores.isNotEmpty ? widget.frontend.cores.first : null;
    unawaited(_bootstrap());
    _frameTimer = Timer.periodic(const Duration(milliseconds: 16), (_) async {
      if (widget.frontend.runtime.running) {
        await widget.frontend.runFrame();
        if (mounted) setState(() {});
      }
    });
  }

  Future<void> _bootstrap() async {
    await widget.frontend.initialize();
    await _refreshRuntimeLists();
    _selectedGame ??= widget.frontend.games.isNotEmpty ? widget.frontend.games.first : null;
    _selectedCore ??= widget.frontend.cores.isNotEmpty ? widget.frontend.cores.first : null;
    if (mounted) setState(() {});
  }

  Future<void> _refreshRuntimeLists() async {
    _coreOptions = await widget.frontend.coreOptions();
    _overlayConfigs = await widget.frontend.availableOverlayConfigs();
  }

  @override
  void dispose() {
    _frameTimer?.cancel();
    super.dispose();
  }


  Future<void> _launchGame(GameEntry game) async {
    final fileName = game.path.split(Platform.pathSeparator).last;
    final ext = fileName.contains('.') ? fileName.split('.').last.toLowerCase() : '';
    final preferredCorePath = ext.isEmpty ? '' : (widget.frontend.settings['content_core_$ext'] ?? '');
    final plan = await widget.frontend.planContentLaunch(game.path, preferredCorePath: preferredCorePath);
    if (!mounted) return;
    var launchGame = game;
    var selectedCorePath = plan.selectedCorePath;
    if (plan.needsCoreChoice) {
      final selected = await _chooseCoreForGame(game, plan.candidates);
      if (selected == null) return;
      selectedCorePath = selected.path;
      await widget.frontend.setSetting('content_core_${plan.contentExtension}', selected.path);
      launchGame = GameEntry(title: game.title, system: selected.system, core: selected.name, lastPlayed: game.lastPlayed, playTime: game.playTime, path: game.path, initials: game.initials);
    } else if (plan.isSelected) {
      final selected = selectedCorePath.isEmpty
          ? (plan.candidates.isNotEmpty ? plan.candidates.first : null)
          : plan.candidates.where((core) => core.path == selectedCorePath).firstOrNull;
      if (selected != null) {
        selectedCorePath = selected.path;
        await widget.frontend.setSetting('content_core_${plan.contentExtension}', selected.path);
        launchGame = GameEntry(title: game.title, system: selected.system, core: selected.name, lastPlayed: game.lastPlayed, playTime: game.playTime, path: game.path, initials: game.initials);
      }
    } else if (plan.decision == 0) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(plan.reason.isEmpty ? '互換コアが見つかりません。info.zipを取得してコアを再スキャンしてください。' : plan.reason)));
      return;
    }
    final ok = await widget.frontend.launchPath(game.path, preferredCorePath: selectedCorePath);
    await _refreshRuntimeLists();
    if (!mounted) return;
    setState(() => _selectedGame = launchGame);
    if (ok) {
      await Navigator.of(context).push(MaterialPageRoute<void>(
        fullscreenDialog: true,
        builder: (_) => _PlayScreen(frontend: widget.frontend, game: launchGame),
      ));
      if (mounted) setState(() {});
    }
  }

  Future<CoreEntry?> _chooseCoreForGame(GameEntry game, List<CoreEntry> candidates) {
    return showModalBottomSheet<CoreEntry>(
      context: context,
      backgroundColor: const Color(0xFF081321),
      showDragHandle: true,
      builder: (context) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(18),
          child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('${game.title} のコアを選択', style: const TextStyle(fontSize: 20, fontWeight: FontWeight.w900)),
            const SizedBox(height: 12),
            for (final core in candidates)
              ListTile(
                leading: const Icon(Icons.extension, color: _accent),
                title: Text(core.name),
                subtitle: Text('${core.system} • ${core.supportedExtensions.join(', ')}'),
                onTap: () => Navigator.of(context).pop(core),
              ),
          ]),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'Retrofront',
      theme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: _bg,
        fontFamily: Platform.isIOS ? '.SF Pro Text' : 'Inter',
        useMaterial3: true,
      ),
      home: LayoutBuilder(
        builder: (context, constraints) {
          final mobile = constraints.maxWidth < 720 || (!kIsWeb && Platform.isIOS);
          return _ShellBackground(
            child: mobile ? _mobileHome(context) : _desktopHome(context),
          );
        },
      ),
    );
  }

  Widget _desktopHome(BuildContext context) {
    final game = _selectedGame;
    return Scaffold(
      backgroundColor: Colors.transparent,
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(18),
          child: Column(
            children: [
              Expanded(
                flex: 8,
                child: Row(
                  children: [
                    SizedBox(width: 230, child: _desktopRail()),
                    const SizedBox(width: 10),
                    Expanded(flex: 9, child: _libraryPanel(desktop: true)),
                    const SizedBox(width: 10),
                    SizedBox(width: 305, child: game == null ? _emptyDetails() : _gameDetails(game)),
                  ],
                ),
              ),
              const SizedBox(height: 10),
              Expanded(
                flex: 5,
                child: Row(
                  children: [
                    Expanded(flex: 3, child: _importPanel()),
                    const SizedBox(width: 10),
                    Expanded(flex: 3, child: _coreLoadPanel()),
                    const SizedBox(width: 10),
                    Expanded(flex: 5, child: _assetInstallPanel()),
                  ],
                ),
              ),
              const SizedBox(height: 10),
              SizedBox(height: 250, child: _settingsPanel(desktop: true)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _mobileHome(BuildContext context) {
    final pages = [
      _mobileLibraryPage(),
      _playlistPanel(),
      _coreMobilePanel(),
      _settingsPanel(desktop: false),
    ];
    return Scaffold(
      backgroundColor: Colors.transparent,
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(18, 10, 18, 0),
          child: Column(
            children: [
              Expanded(child: pages[_tab]),
              _mobileNav(),
              const SizedBox(height: 10),
            ],
          ),
        ),
      ),
    );
  }

  Widget _mobileLibraryPage() {
    return _libraryPanel(desktop: false);
  }

  Widget _emptyDetails() {
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.videogame_asset_off_outlined, size: 52, color: _muted),
            const SizedBox(height: 20),
            const Text('ライブラリが空です', style: TextStyle(fontSize: 24, fontWeight: FontWeight.w900)),
            const SizedBox(height: 10),
            Text(widget.frontend.statusMessage, style: const TextStyle(color: _muted)),
            const Spacer(),
            _GradientButton(label: 'ROMをインポート', onTap: _pickRom),
          ],
        ),
      ),
    );
  }

  Widget _desktopRail() {
    final items = ['ライブラリ', 'プレイリスト', 'コア', '設定', 'ダウンロード', '履歴'];
    final icons = [Icons.videogame_asset_outlined, Icons.playlist_play, Icons.extension_outlined, Icons.settings_outlined, Icons.download_outlined, Icons.history];
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            Row(
              children: [
                Container(
                  width: 42,
                  height: 42,
                  decoration: const BoxDecoration(
                    shape: BoxShape.circle,
                    gradient: LinearGradient(colors: [_accent, _cyan], begin: Alignment.topLeft, end: Alignment.bottomRight),
                  ),
                ),
                const Spacer(),
                _IconPill(icon: Icons.menu),
              ],
            ),
            const SizedBox(height: 28),
            for (var i = 0; i < items.length; i++)
              _RailItem(
                label: items[i],
                icon: icons[i],
                selected: _section == items[i],
                onTap: () => setState(() => _section = items[i]),
              ),
            const Spacer(),
            const Divider(color: _line),
            const Align(alignment: Alignment.centerLeft, child: Text('22:48', style: TextStyle(fontSize: 15, color: _text))),
            const Align(alignment: Alignment.centerLeft, child: Text('2024/05/20', style: TextStyle(fontSize: 11, color: _muted))),
            const SizedBox(height: 16),
            SizedBox(height: 58, child: CustomPaint(painter: _StorageBarsPainter())),
            const SizedBox(height: 10),
            Row(children: const [Text('ストレージ', style: TextStyle(color: _text, fontSize: 12)), Spacer(), Icon(Icons.sync, size: 13, color: _muted)]),
            const Align(alignment: Alignment.centerLeft, child: Text('512GB 中 243GB 使用', style: TextStyle(color: _muted, fontSize: 10))),
          ],
        ),
      ),
    );
  }

  Widget _libraryPanel({required bool desktop}) {
    return _GlassPanel(
      child: Padding(
        padding: EdgeInsets.all(desktop ? 28 : 14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Text('すべてのゲーム', style: TextStyle(fontSize: desktop ? 18 : 12, color: _muted)),
                    Text(desktop ? '${widget.frontend.games.length} ゲーム' : 'ライブラリ', style: TextStyle(fontSize: desktop ? 26 : 20, fontWeight: FontWeight.w800, color: _text)),
                  ]),
                ),
                if (desktop) const SizedBox(width: 235, child: _SearchBox()),
                if (!desktop) _IconPill(icon: Icons.search),
                const SizedBox(width: 8),
                InkWell(onTap: _pickRom, child: const _IconPill(icon: Icons.file_upload_outlined)),
                const SizedBox(width: 8),
                _IconPill(icon: Icons.filter_alt_outlined),
                const SizedBox(width: 8),
                _IconPill(icon: Icons.tune),
              ],
            ),
            const SizedBox(height: 22),
            if (desktop)
              const Padding(
                padding: EdgeInsets.symmetric(horizontal: 72),
                child: Row(children: [Expanded(flex: 3, child: Text('タイトル', style: TextStyle(color: _muted, fontSize: 12))), Expanded(child: Text('コア', style: TextStyle(color: _muted, fontSize: 12))), Expanded(child: Text('最後にプレイ', style: TextStyle(color: _muted, fontSize: 12))), Expanded(child: Text('プレイ時間', style: TextStyle(color: _muted, fontSize: 12)))]),
              ),
            Expanded(
              child: widget.frontend.games.isEmpty
                  ? _EmptyLibrary(onImport: _pickRom, message: widget.frontend.statusMessage)
                  : ListView.separated(
                itemCount: widget.frontend.games.length,
                separatorBuilder: (_, __) => Divider(color: desktop ? _line.withOpacity(.55) : Colors.transparent, height: desktop ? 1 : 8),
                itemBuilder: (context, index) {
                  final game = widget.frontend.games[index];
                  final selected = game == _selectedGame;
                  return _GameRow(
                    game: game,
                    desktop: desktop,
                    selected: selected,
                    onTap: () {
                      setState(() => _selectedGame = game);
                      if (!desktop) unawaited(_launchGame(game));
                    },
                    onPlay: () => _launchGame(game),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _gameDetails(GameEntry game) {
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(28),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            AspectRatio(
              aspectRatio: 1,
              child: Container(
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(14),
                  gradient: const LinearGradient(colors: [Color(0xFF0E1730), Color(0xFFD58A2A), Color(0xFF193B7D)], begin: Alignment.topLeft, end: Alignment.bottomRight),
                  border: Border.all(color: Colors.white54),
                ),
                child: Stack(children: [
                  Center(child: Text(game.initials, style: const TextStyle(fontSize: 48, fontWeight: FontWeight.w900, color: Colors.white))),
                  const Positioned(bottom: 14, right: 14, child: Text('CAPCOM', style: TextStyle(fontWeight: FontWeight.w900, color: Color(0xFFFFE071)))),
                ]),
              ),
            ),
            const SizedBox(height: 18),
            Text(game.title, style: const TextStyle(color: _text, fontSize: 28, fontWeight: FontWeight.w900, height: 1.02)),
            const SizedBox(height: 12),
            Text(game.system, style: const TextStyle(color: _muted)),
            const Text('Capcom', style: TextStyle(color: _muted)),
            const SizedBox(height: 18),
            Row(children: [
              _Chip(label: 'PSX'),
              const SizedBox(width: 12),
              const Icon(Icons.star, color: _accent, size: 16),
              const Icon(Icons.star, color: _muted, size: 16),
              const Icon(Icons.star, color: _muted, size: 16),
              const Icon(Icons.star_half, color: _muted, size: 16),
              const Spacer(),
              Text(game.playTime, style: const TextStyle(color: _muted, fontSize: 12)),
            ]),
            const SizedBox(height: 18),
            _GradientButton(label: 'プレイ', onTap: () => _launchGame(game)),
            const SizedBox(height: 18),
            Row(children: [
              Expanded(child: _ActionTile(icon: Icons.gamepad_outlined, label: 'コアをロード', onTap: () {})),
              const SizedBox(width: 18),
              Expanded(child: _ActionTile(icon: Icons.info_outline, label: '詳細を表示', onTap: () {})),
            ]),
          ],
        ),
      ),
    );
  }

  Widget _importPanel() {
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(22),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [const Text('ROMをインポート', style: TextStyle(fontSize: 17, fontWeight: FontWeight.w800)), const Spacer(), IconButton(onPressed: () {}, icon: const Icon(Icons.close, size: 18))]),
          const SizedBox(height: 8),
          Expanded(
            child: DecoratedBox(
              decoration: BoxDecoration(border: Border.all(color: _muted.withOpacity(.35), style: BorderStyle.solid), borderRadius: BorderRadius.circular(8)),
              child: Center(child: Column(mainAxisSize: MainAxisSize.min, children: [
                const Icon(Icons.file_upload_outlined, color: _muted, size: 40),
                const SizedBox(height: 10),
                const Text('ファイルをドラッグ＆ドロップ\nまたは', textAlign: TextAlign.center, style: TextStyle(color: _muted, fontSize: 12)),
                const SizedBox(height: 8),
                _SmallButton(label: 'ファイルを選択', onTap: _pickRom),
              ])),
            ),
          ),
          const SizedBox(height: 12),
          const Text('スキャン設定', style: TextStyle(color: _muted, fontSize: 12)),
          const SizedBox(height: 8),
          _PathSelector(path: '/storage/roms', onTap: _pickRomDirectory),
          const SizedBox(height: 10),
          Row(children: [const Expanded(child: Text('対応するコアを自動的に割り当てる', style: TextStyle(color: _muted, fontSize: 12))), Switch(value: true, activeColor: _accent, onChanged: (_) {})]),
          _GradientButton(label: 'インポートを開始', onTap: _pickRom),
        ]),
      ),
    );
  }

  Widget _assetInstallPanel() {
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(22),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          const Text('RetroArch フロントエンド資産', style: TextStyle(fontSize: 17, fontWeight: FontWeight.w800)),
          const SizedBox(height: 8),
          const Text('assets.zip / info.zip / overlays.zip を取得して展開します。info.zip はコアとROM拡張子の紐付け、overlays.zip はcfgオーバーレイに必要です。', style: TextStyle(color: _muted, fontSize: 12)),
          const SizedBox(height: 14),
          Expanded(
            child: ListView(children: [
              for (final package in RetrofrontNative.frontendAssetPackages)
                Padding(
                  padding: const EdgeInsets.only(bottom: 10),
                  child: _AssetPackageRow(
                    package: package,
                    destination: widget.frontend.settings[package.destinationSettingKey] ?? '',
                    onBundled: () => _installAssetPackage(package.name, download: false),
                    onDownload: () => _installAssetPackage(package.name, download: true),
                  ),
                ),
            ]),
          ),
          Text(widget.frontend.statusMessage, maxLines: 2, overflow: TextOverflow.ellipsis, style: const TextStyle(color: _muted, fontSize: 11)),
        ]),
      ),
    );
  }

  Widget _coreLoadPanel() {
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(22),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          const Text('コアをロード', style: TextStyle(fontSize: 17, fontWeight: FontWeight.w800)),
          const SizedBox(height: 16),
          const _SearchBox(hint: 'コアを検索'),
          const SizedBox(height: 12),
          Expanded(child: _coreList(compact: true)),
          const SizedBox(height: 12),
          _GradientButton(label: 'コアをロード', onTap: () async {
            if (_selectedCore != null) await widget.frontend.loadCore(_selectedCore!);
            await _refreshRuntimeLists();
            if (mounted) setState(() {});
          }),
        ]),
      ),
    );
  }

  Widget _coreMobilePanel() {
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: const [Text('コア', style: TextStyle(fontSize: 20, fontWeight: FontWeight.w900)), Spacer(), _IconPill(icon: Icons.search), SizedBox(width: 8), _IconPill(icon: Icons.tune)]),
          const SizedBox(height: 18),
          Expanded(child: _coreList(compact: false)),
          Row(children: [
            Expanded(child: _GradientButton(label: '選択中のコアをロード', onTap: () async {
              if (_selectedCore != null) await widget.frontend.loadCore(_selectedCore!);
              await _refreshRuntimeLists();
              if (mounted) setState(() {});
            })),
            const SizedBox(width: 10),
            FloatingActionButton(backgroundColor: _accent, onPressed: () async {
              await widget.frontend.scanCores(widget.frontend.settings['libretro_directory'] ?? '');
              await _refreshRuntimeLists();
              if (mounted) setState(() {});
            }, child: const Icon(Icons.sync)),
          ]),
        ]),
      ),
    );
  }

  Widget _coreList({required bool compact}) {
    if (widget.frontend.cores.isEmpty) {
      return _EmptyLibrary(onImport: () => widget.frontend.scanCores(widget.frontend.settings['libretro_directory'] ?? '').then((_) { if (mounted) setState(() {}); }), message: 'libretro_directory に *_libretro コアを配置してスキャンしてください。');
    }
    return ListView.separated(
      itemCount: widget.frontend.cores.length,
      separatorBuilder: (_, __) => const SizedBox(height: 6),
      itemBuilder: (context, index) {
        final core = widget.frontend.cores[index];
        final selected = core == _selectedCore || core.loaded;
        return _CoreRow(core: core, selected: selected, onTap: () async {
          setState(() => _selectedCore = core);
          if (!compact) {
            await widget.frontend.loadCore(core);
            await _refreshRuntimeLists();
            if (mounted) setState(() {});
          }
        });
      },
    );
  }

  Widget _gameViewport() {
    return _GlassPanel(
      padding: EdgeInsets.zero,
      child: Stack(children: [
        Column(children: [
          Expanded(
            child: ClipRRect(
              borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
              child: Container(
                width: double.infinity,
                decoration: const BoxDecoration(color: Colors.black),
                child: _VideoFrameView(frontend: widget.frontend),
              ),
            ),
          ),
          Container(
            height: 72,
            decoration: const BoxDecoration(color: Color(0xAA07111E), borderRadius: BorderRadius.vertical(bottom: Radius.circular(16))),
            child: Row(mainAxisAlignment: MainAxisAlignment.spaceEvenly, children: [
              _ViewportButton(icon: Icons.menu, label: 'メニュー', onTap: () => setState(widget.frontend.openQuickMenu)),
              _ViewportButton(icon: Icons.save_outlined, label: 'クイックセーブ', onTap: () async => widget.frontend.quickSave()),
              _ViewportButton(icon: Icons.download_for_offline_outlined, label: 'クイックロード', onTap: () async => widget.frontend.quickLoad()),
              _ViewportButton(icon: Icons.restart_alt, label: 'リスタート', onTap: () async { await widget.frontend.reset(); setState(() {}); }),
              _ViewportButton(icon: Icons.power_settings_new, label: 'シャットダウン', onTap: () => setState(() {})),
            ]),
          ),
        ]),
        if (widget.frontend.runtime.overlayEnabled) _GameOverlay(onButton: (id, down) => widget.frontend.setJoypadButton(id, down), onMenu: () => setState(widget.frontend.openQuickMenu)),
        if (widget.frontend.runtime.quickMenuOpen) _quickMenu(),
      ]),
    );
  }

  Widget _quickMenu() {
    return Positioned.fill(
      child: DecoratedBox(
        decoration: BoxDecoration(color: Colors.black.withOpacity(.58), borderRadius: BorderRadius.circular(16)),
        child: Center(
          child: SizedBox(
            width: 440,
            child: _GlassPanel(
              child: Padding(
                padding: const EdgeInsets.all(22),
                child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Row(children: [const Text('クイックメニュー', style: TextStyle(fontSize: 22, fontWeight: FontWeight.w900)), const Spacer(), IconButton(onPressed: () => setState(widget.frontend.closeQuickMenu), icon: const Icon(Icons.close))]),
                  _QuickAction(icon: Icons.save_outlined, title: 'ステートを保存', subtitle: 'RetroArch 互換ステートスロット 0', onTap: () => widget.frontend.quickSave()),
                  _QuickAction(icon: Icons.download, title: 'ステートをロード', subtitle: 'ステートスロット 0 を復元', onTap: () => widget.frontend.quickLoad()),
                  _QuickAction(icon: Icons.tune, title: '起動中のコア設定', subtitle: '${_coreOptions.length} 個のオプション', onTap: () => _showCoreOptions(context)),
                  _QuickAction(icon: Icons.gamepad_outlined, title: 'オーバーレイ', subtitle: widget.frontend.runtime.overlayEnabled ? '表示中' : '非表示', onTap: () => widget.frontend.setOverlayEnabled(!widget.frontend.runtime.overlayEnabled).then((_) => setState(() {}))),
                ]),
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _settingsPanel({required bool desktop}) {
    final categories = ['Video', 'Audio', 'Controller', 'Library', 'Loaded Core', 'Storage'];
    return _GlassPanel(
      child: Padding(
        padding: EdgeInsets.all(desktop ? 22 : 14),
        child: desktop
            ? Row(children: [
                SizedBox(
                  width: 210,
                  child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    const Text('設定', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w900)),
                    const SizedBox(height: 6),
                    const Text('実際に保存・反映されるアプリ設定', style: TextStyle(color: _muted, fontSize: 11)),
                    const SizedBox(height: 16),
                    for (final c in categories) _SettingCategory(label: c, selected: c == categories.first),
                  ]),
                ),
                const VerticalDivider(color: _line),
                Expanded(child: _settingsControls()),
              ])
            : Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                const Text('設定', style: TextStyle(fontSize: 20, fontWeight: FontWeight.w900)),
                const SizedBox(height: 4),
                const Text('実際に保存・反映されるアプリ設定', style: TextStyle(color: _muted, fontSize: 11)),
                const SizedBox(height: 14),
                Expanded(child: _settingsControls()),
              ]),
      ),
    );
  }

  Widget _settingsControls() {
    final s = widget.frontend.settings;
    return ListView(children: [
      _SettingsGroup(title: 'Video', children: [
        _SettingsChoiceRow(title: 'Renderer', value: _rendererLabel(s['video_driver']), choices: const ['Metal', 'Software', 'MoltenVK', 'OpenGL ES'], onChanged: (v) => _setMappedSetting('video_driver', v, const {'Metal': 'metal', 'Software': 'software', 'MoltenVK': 'moltenvk', 'OpenGL ES': 'opengl'})),
        _SettingsChoiceRow(title: 'Scale', value: _scaleLabel(s['video_scale_mode']), choices: const ['Aspect', 'Integer', 'Stretch'], onChanged: (v) => _setMappedSetting('video_scale_mode', v, const {'Aspect': 'keep_aspect', 'Integer': 'integer', 'Stretch': 'stretch'})),
        _SettingsChoiceRow(title: 'Filter', value: _filterLabel(s['video_filter_mode']), choices: const ['Nearest', 'Linear'], onChanged: (v) => _setMappedSetting('video_filter_mode', v, const {'Nearest': 'nearest', 'Linear': 'linear'})),
        _SettingsToggleRow(title: 'VSync', subtitle: '表示更新に同期', value: _settingBool('video_vsync'), onChanged: (v) => _setBoolSetting('video_vsync', v)),
      ]),
      _SettingsGroup(title: 'Audio', children: [
        _SettingsToggleRow(title: 'Audio Output', subtitle: '音声出力を有効化', value: _settingBool('audio_enable'), onChanged: (v) => _setBoolSetting('audio_enable', v)),
        _SettingsToggleRow(title: 'Audio Sync', subtitle: '音声同期', value: _settingBool('audio_sync'), onChanged: (v) => _setBoolSetting('audio_sync', v)),
        _SettingsChoiceRow(title: 'Latency', value: '${s['audio_latency_ms'] ?? '64'} ms', choices: const ['32 ms', '64 ms', '96 ms', '128 ms'], onChanged: (v) => _setPlainSetting('audio_latency_ms', v.replaceAll(' ms', ''))),
      ]),
      _SettingsGroup(title: 'Controller', children: [
        _SettingsToggleRow(title: 'Touch Overlay', subtitle: '画面上コントローラー', value: _settingBool('input_overlay_enable'), onChanged: (v) async { await widget.frontend.setOverlayEnabled(v); if (mounted) setState(() {}); }),
        _SettingsChoiceRow(title: 'Overlay Set', value: _overlayChoiceLabel(s['input_overlay']), choices: _overlayConfigs.isEmpty ? const ['Not selected'] : _overlayConfigs.map(_overlayChoiceLabel).toList(), onChanged: (v) async { final path = _overlayConfigs.where((path) => _overlayChoiceLabel(path) == v).firstOrNull; if (path != null) { await widget.frontend.loadOverlay(path); await _refreshRuntimeLists(); if (mounted) setState(() {}); } }),
        _SettingsChoiceRow(title: 'Overlay Opacity', value: _opacityLabel(s['input_overlay_opacity']), choices: const ['45%', '70%', '90%'], onChanged: (v) => _setMappedSetting('input_overlay_opacity', v, const {'45%': '0.45', '70%': '0.70', '90%': '0.90'})),
        _SettingsToggleRow(title: 'Haptics', subtitle: 'タッチ操作の振動フィードバック', value: _settingBool('input_haptic_feedback'), onChanged: (v) => _setBoolSetting('input_haptic_feedback', v)),
      ]),
      _SettingsGroup(title: 'Library', children: [
        _SettingsChoiceRow(title: 'Sort', value: _librarySortLabel(s['library_sort_mode']), choices: const ['Name ↑', 'Name ↓', 'Extension'], onChanged: (v) => _setMappedSetting('library_sort_mode', v, const {'Name ↑': 'name_ascending', 'Name ↓': 'name_descending', 'Extension': 'extension'})),
        _SettingsToggleRow(title: 'Core Badges', subtitle: '互換コア数をROMに表示', value: _settingBool('library_show_core_badges'), onChanged: (v) => _setBoolSetting('library_show_core_badges', v)),
        _SettingsToggleRow(title: 'File Details', subtitle: '拡張子とサイズを表示', value: _settingBool('library_show_file_details'), onChanged: (v) => _setBoolSetting('library_show_file_details', v)),
        _SettingsToggleRow(title: 'Auto Scan', subtitle: '起動時にROMを再スキャン', value: _settingBool('library_auto_scan_on_launch'), onChanged: (v) => _setBoolSetting('library_auto_scan_on_launch', v)),
      ]),
      _SettingsGroup(title: 'Loaded Core', children: [
        _SettingsRow(title: 'Current Core', value: widget.frontend.runtime.loadedCore.isEmpty ? 'Not loaded' : widget.frontend.runtime.loadedCore),
        if (widget.frontend.cores.isEmpty)
          const _SettingsRow(title: 'Bundled Cores', value: 'No bundled cores discovered')
        else
          for (final core in widget.frontend.cores)
            _SettingsCoreButton(core: core, onTap: () async { await widget.frontend.loadCore(core); await _refreshRuntimeLists(); if (mounted) setState(() {}); }),
        if (_coreOptions.isEmpty)
          const _SettingsRow(title: 'Core Options', value: 'Load a core to edit its options')
        else
          for (final option in _coreOptions)
            _SettingsChoiceRow(title: option.description.isEmpty ? option.key : option.description, value: option.value, choices: option.values.isEmpty ? [option.value] : option.values, onChanged: (v) async { await widget.frontend.setCoreOption(option.key, v); await _refreshRuntimeLists(); if (mounted) setState(() {}); }),
      ]),
      _SettingsGroup(title: 'Storage', children: [
        _SettingsRow(title: 'Content Folder', value: s['content_directory'] ?? ''),
        _SettingsRow(title: 'Core Folder', value: s['libretro_directory'] ?? ''),
        _SettingsRow(title: 'Info Folder', value: s['libretro_info_path'] ?? ''),
        _SettingsRow(title: 'Saves', value: s['savefile_directory'] ?? ''),
        _SettingsRow(title: 'States', value: s['savestate_directory'] ?? ''),
        _SettingsRow(title: 'System/BIOS', value: s['system_directory'] ?? ''),
        _SettingsRow(title: 'Screenshots', value: s['screenshot_directory'] ?? ''),
        for (final package in RetrofrontNative.frontendAssetPackages) _SettingsAssetRow(package: package, onBundled: () => _installAssetPackage(package.name, download: false), onDownload: () => _installAssetPackage(package.name, download: true)),
      ]),
      const SizedBox(height: 10),
      Text(widget.frontend.statusMessage, style: const TextStyle(color: _muted, fontSize: 11)),
    ]);
  }

  bool _settingBool(String key) => widget.frontend.settings[key] != 'false';

  Future<void> _setBoolSetting(String key, bool value) => _setPlainSetting(key, value ? 'true' : 'false');

  Future<void> _setMappedSetting(String key, String label, Map<String, String> values) => _setPlainSetting(key, values[label] ?? label);

  Future<void> _setPlainSetting(String key, String value) async {
    await widget.frontend.setSetting(key, value);
    if (mounted) setState(() {});
  }

  String _rendererLabel(String? value) => switch (value) { 'software' => 'Software', 'moltenvk' => 'MoltenVK', 'opengl' => 'OpenGL ES', _ => 'Metal' };
  String _scaleLabel(String? value) => switch (value) { 'integer' => 'Integer', 'stretch' => 'Stretch', _ => 'Aspect' };
  String _filterLabel(String? value) => value == 'linear' ? 'Linear' : 'Nearest';
  String _librarySortLabel(String? value) => switch (value) { 'extension' => 'Extension', 'name_descending' => 'Name ↓', _ => 'Name ↑' };
  String _opacityLabel(String? value) => '${(((double.tryParse(value ?? '0.70') ?? .70) * 100).round())}%';
  String _overlayLabel(String? path) {
    if (path == null || path.isEmpty) return 'Not selected';
    return path.split(Platform.pathSeparator).last.replaceAll('.cfg', '').replaceAll('_', ' ').replaceAll('-', ' ');
  }

  String _overlayChoiceLabel(String? path) {
    if (path == null || path.isEmpty) return 'Not selected';
    final parts = path.split(Platform.pathSeparator);
    final label = _overlayLabel(path);
    final gamepadsIndex = parts.indexOf('gamepads');
    if (gamepadsIndex >= 0 && gamepadsIndex + 1 < parts.length) {
      return '$label (${parts[gamepadsIndex + 1]})';
    }
    return parts.length > 1 ? '$label (${parts[parts.length - 2]})' : label;
  }

  Widget _playlistPanel() {
    return _GlassPanel(
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: const [Text('プレイリスト', style: TextStyle(fontSize: 20, fontWeight: FontWeight.w900)), Spacer(), _IconPill(icon: Icons.add)]),
          const SizedBox(height: 18),
          Expanded(child: ListView.separated(itemBuilder: (_, i) => _PlaylistRow(entry: widget.frontend.playlists[i]), separatorBuilder: (_, __) => const SizedBox(height: 10), itemCount: widget.frontend.playlists.length)),
        ]),
      ),
    );
  }

  Widget _mobileNav() {
    final items = [(Icons.videogame_asset_outlined, 'ライブラリ'), (Icons.playlist_play, 'プレイリスト'), (Icons.extension_outlined, 'コア'), (Icons.settings_outlined, '設定')];
    return Container(
      height: 70,
      decoration: BoxDecoration(color: const Color(0xEE07111E), border: Border(top: BorderSide(color: _line.withOpacity(.7)))),
      child: Row(mainAxisAlignment: MainAxisAlignment.spaceAround, children: [
        for (var i = 0; i < items.length; i++)
          InkWell(onTap: () => setState(() => _tab = i), child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [Icon(items[i].$1, color: i == _tab ? _text : _muted, size: 20), const SizedBox(height: 4), Text(items[i].$2, style: TextStyle(color: i == _tab ? _text : _muted, fontSize: 10))])),
      ]),
    );
  }

  Future<void> _installAssetPackage(String name, {required bool download}) async {
    await widget.frontend.installFrontendAssetPackage(name, download: download);
    await _refreshRuntimeLists();
    if (mounted) setState(() {});
  }

  Future<void> _pickRom() async {
    final paths = await _promptForPaths(
      title: 'ROMファイルのパス',
      hint: widget.frontend.settings['content_directory'] ?? '/path/to/game.rom',
      multiline: true,
    );
    if (paths.isEmpty) return;
    final importedGames = <GameEntry>[];
    for (final path in paths) {
      await widget.frontend.importRom(path);
      final imported = _latestImportedGameFor(path);
      if (imported != null) importedGames.add(imported);
    }
    if (!mounted) return;
    setState(() => _selectedGame = importedGames.isNotEmpty ? importedGames.first : _selectedGame);
    if (importedGames.length == 1) {
      await _launchGame(importedGames.single);
    }
  }

  GameEntry? _latestImportedGameFor(String sourcePath) {
    if (widget.frontend.games.isEmpty) return null;
    final sourceName = _pathBasename(sourcePath);
    for (final game in widget.frontend.games) {
      if (_pathBasename(game.path) == sourceName) return game;
    }
    return null;
  }

  String _pathBasename(String path) {
    final normalized = path.replaceAll('\\', '/');
    final index = normalized.lastIndexOf('/');
    return index == -1 ? normalized : normalized.substring(index + 1);
  }

  Future<void> _pickRomDirectory() async {
    final paths = await _promptForPaths(
      title: 'ROMディレクトリのパス',
      hint: widget.frontend.settings['content_directory'] ?? '/path/to/roms',
    );
    if (paths.isEmpty) return;
    await widget.frontend.scanRoms(paths.first);
    if (mounted) setState(() {});
  }

  Future<List<String>> _promptForPaths({required String title, required String hint, bool multiline = false}) async {
    final controller = TextEditingController(text: multiline ? '' : hint);
    final value = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xFF081321),
        title: Text(title),
        content: TextField(
          controller: controller,
          autofocus: true,
          minLines: multiline ? 3 : 1,
          maxLines: multiline ? 6 : 1,
          decoration: InputDecoration(
            hintText: multiline ? '$hint\n/path/to/another.rom' : hint,
            helperText: multiline ? '複数指定する場合は1行に1ファイルずつ入力してください。' : null,
          ),
          style: const TextStyle(color: _text),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('キャンセル')),
          FilledButton(onPressed: () => Navigator.of(context).pop(controller.text), child: const Text('OK')),
        ],
      ),
    );
    controller.dispose();
    if (value == null) return const [];
    return value
        .split(RegExp(r'\r?\n'))
        .map((path) => path.trim())
        .where((path) => path.isNotEmpty)
        .toList();
  }

  Future<void> _showCoreOptions(BuildContext context) async {
    await _refreshRuntimeLists();
    if (!mounted) return;
    await showModalBottomSheet<void>(
      context: context,
      backgroundColor: const Color(0xFF081321),
      showDragHandle: true,
      builder: (context) => StatefulBuilder(builder: (context, modalSetState) {
        final loadedCore = widget.frontend.runtime.loadedCore;
        return ListView(padding: const EdgeInsets.all(18), children: [
          Text(loadedCore.isEmpty ? '起動中のコア設定' : '$loadedCore のコア設定', style: const TextStyle(fontSize: 20, fontWeight: FontWeight.w900)),
          const SizedBox(height: 12),
          if (_coreOptions.isEmpty)
            const ListTile(
              leading: Icon(Icons.info_outline, color: _muted),
              title: Text('設定可能なコアオプションがありません'),
              subtitle: Text('コアをロードするかゲームを起動すると、コアが公開するオプションを編集できます。'),
            )
          else
            for (final option in _coreOptions)
              ListTile(
                title: Text(option.description.isEmpty ? option.key : option.description),
                subtitle: Text(option.key),
                trailing: DropdownButton<String>(
                  value: option.values.contains(option.value) ? option.value : option.values.firstOrNull,
                  items: option.values.map((v) => DropdownMenuItem(value: v, child: Text(v))).toList(),
                  onChanged: (value) async {
                    if (value == null) return;
                    await widget.frontend.setCoreOption(option.key, value);
                    await _refreshRuntimeLists();
                    if (mounted) setState(() {});
                    modalSetState(() {});
                  },
                ),
              ),
        ]);
      }),
    );
  }
}


class _PlayScreen extends StatefulWidget {
  const _PlayScreen({required this.frontend, required this.game});
  final RetrofrontFrontend frontend;
  final GameEntry game;

  @override
  State<_PlayScreen> createState() => _PlayScreenState();
}

class _PlayScreenState extends State<_PlayScreen> {
  Timer? _timer;

  @override
  void initState() {
    super.initState();
    _timer = Timer.periodic(const Duration(milliseconds: 16), (_) async {
      if (widget.frontend.runtime.running) {
        await widget.frontend.runFrame();
        if (mounted) setState(() {});
      }
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      body: LayoutBuilder(builder: (context, constraints) {
        void sendTouch(Offset position, bool active) {
          final x = (position.dx / constraints.maxWidth).clamp(0.0, 1.0);
          final y = (position.dy / constraints.maxHeight).clamp(0.0, 1.0);
          widget.frontend.setOverlayTouch(0, x, y, active);
        }
        return Listener(
          onPointerDown: (event) => sendTouch(event.localPosition, true),
          onPointerMove: (event) => sendTouch(event.localPosition, true),
          onPointerUp: (event) => sendTouch(event.localPosition, false),
          onPointerCancel: (event) => widget.frontend.setOverlayTouch(0, 0, 0, false),
          child: Stack(children: [
            Positioned.fill(child: _VideoFrameView(frontend: widget.frontend, fit: BoxFit.contain)),
            if (widget.frontend.runtime.overlayEnabled) _GameOverlay(onButton: (id, down) => widget.frontend.setJoypadButton(id, down), onMenu: () => setState(widget.frontend.openQuickMenu)),
            Positioned(top: 24, left: 24, child: SafeArea(child: _ViewportButton(icon: Icons.close_fullscreen, label: '戻る', onTap: () async => Navigator.of(context).pop()))),
            Positioned(top: 24, right: 24, child: SafeArea(child: Row(children: [
              _ViewportButton(icon: Icons.save_outlined, label: '保存', onTap: () => widget.frontend.quickSave()),
              const SizedBox(width: 14),
              _ViewportButton(icon: Icons.restart_alt, label: 'リセット', onTap: () async { await widget.frontend.reset(); if (mounted) setState(() {}); }),
            ]))),
          ]),
        );
      }),
    );
  }
}

class _VideoFrameView extends StatefulWidget {
  const _VideoFrameView({required this.frontend, this.fit = BoxFit.contain});
  final RetrofrontFrontend frontend;
  final BoxFit fit;

  @override
  State<_VideoFrameView> createState() => _VideoFrameViewState();
}

class _VideoFrameViewState extends State<_VideoFrameView> {
  ui.Image? _image;
  int _decodedFrame = -1;

  @override
  void didUpdateWidget(covariant _VideoFrameView oldWidget) {
    super.didUpdateWidget(oldWidget);
    _decodeLatest();
  }

  @override
  void initState() {
    super.initState();
    _decodeLatest();
  }

  Future<void> _decodeLatest() async {
    final frameNumber = widget.frontend.runtime.frameNumber;
    if (_decodedFrame == frameNumber) return;
    _decodedFrame = frameNumber;
    final frame = await widget.frontend.copyVideoFrame();
    if (frame == null || !mounted) return;
    final completer = Completer<ui.Image>();
    ui.decodeImageFromPixels(frame.rgba, frame.width, frame.height, ui.PixelFormat.rgba8888, completer.complete);
    final image = await completer.future;
    if (mounted) setState(() => _image = image);
  }

  @override
  Widget build(BuildContext context) {
    final image = _image;
    if (image == null) {
      return CustomPaint(painter: _GameScenePainter(frame: widget.frontend.runtime.frameNumber));
    }
    return ColoredBox(color: Colors.black, child: Center(child: RawImage(image: image, fit: widget.fit)));
  }
}

class _ShellBackground extends StatelessWidget {
  const _ShellBackground({required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: const BoxDecoration(gradient: LinearGradient(colors: [Color(0xFF04070F), Color(0xFF071524), Color(0xFF101B35)], begin: Alignment.topLeft, end: Alignment.bottomRight)),
    child: Stack(children: [
      Positioned(top: -100, left: -80, child: _Glow(color: _accent.withOpacity(.22), size: 300)),
      Positioned(bottom: -80, right: -40, child: _Glow(color: _cyan.withOpacity(.16), size: 360)),
      child,
    ]),
  );
}

class _Glow extends StatelessWidget { const _Glow({required this.color, required this.size}); final Color color; final double size; @override Widget build(BuildContext context) => Container(width: size, height: size, decoration: BoxDecoration(shape: BoxShape.circle, boxShadow: [BoxShadow(color: color, blurRadius: size / 2, spreadRadius: size / 5)])); }
class _GlassPanel extends StatelessWidget { const _GlassPanel({required this.child, this.padding}); final Widget child; final EdgeInsetsGeometry? padding; @override Widget build(BuildContext context) => Container(padding: padding, decoration: BoxDecoration(color: _panel, borderRadius: BorderRadius.circular(18), border: Border.all(color: const Color(0xFF24354D)), boxShadow: [BoxShadow(color: Colors.black.withOpacity(.35), blurRadius: 28, offset: const Offset(0, 18))]), child: child); }
class _IconPill extends StatelessWidget { const _IconPill({required this.icon}); final IconData icon; @override Widget build(BuildContext context) => Container(width: 36, height: 36, decoration: BoxDecoration(color: const Color(0xFF101C2E), borderRadius: BorderRadius.circular(10), border: Border.all(color: _line)), child: Icon(icon, size: 18, color: _text)); }
class _SearchBox extends StatelessWidget { const _SearchBox({this.hint = '検索'}); final String hint; @override Widget build(BuildContext context) => Container(height: 42, padding: const EdgeInsets.symmetric(horizontal: 14), decoration: BoxDecoration(color: const Color(0xFF101A2A), borderRadius: BorderRadius.circular(24), border: Border.all(color: _line)), child: Row(children: [const Icon(Icons.search, size: 18, color: _muted), const SizedBox(width: 8), Text(hint, style: const TextStyle(color: _muted, fontSize: 13))])); }
class _RailItem extends StatelessWidget { const _RailItem({required this.label, required this.icon, required this.selected, required this.onTap}); final String label; final IconData icon; final bool selected; final VoidCallback onTap; @override Widget build(BuildContext context) => Padding(padding: const EdgeInsets.only(bottom: 8), child: InkWell(onTap: onTap, borderRadius: BorderRadius.circular(10), child: Container(height: 44, padding: const EdgeInsets.symmetric(horizontal: 14), decoration: BoxDecoration(color: selected ? const Color(0xFF13264A) : Colors.transparent, borderRadius: BorderRadius.circular(10)), child: Row(children: [Icon(icon, size: 18, color: selected ? _text : _muted), const SizedBox(width: 12), Text(label, style: TextStyle(color: selected ? _text : _muted, fontWeight: selected ? FontWeight.w700 : FontWeight.w500))])))); }
class _Chip extends StatelessWidget { const _Chip({required this.label}); final String label; @override Widget build(BuildContext context) => Container(padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5), decoration: BoxDecoration(color: const Color(0xFF142033), borderRadius: BorderRadius.circular(7), border: Border.all(color: _line)), child: Text(label, style: const TextStyle(color: _text, fontSize: 12))); }
class _GradientButton extends StatelessWidget { const _GradientButton({required this.label, required this.onTap}); final String label; final FutureOr<void> Function() onTap; @override Widget build(BuildContext context) => InkWell(onTap: () => onTap(), borderRadius: BorderRadius.circular(10), child: Container(height: 52, alignment: Alignment.center, decoration: BoxDecoration(borderRadius: BorderRadius.circular(10), gradient: const LinearGradient(colors: [_accent, _cyan])), child: Text(label, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.w900)))); }
class _SmallButton extends StatelessWidget { const _SmallButton({required this.label, required this.onTap}); final String label; final VoidCallback onTap; @override Widget build(BuildContext context) => OutlinedButton(onPressed: onTap, style: OutlinedButton.styleFrom(foregroundColor: _text, side: const BorderSide(color: _line), backgroundColor: const Color(0xFF111D2C)), child: Text(label)); }
class _ActionTile extends StatelessWidget { const _ActionTile({required this.icon, required this.label, required this.onTap}); final IconData icon; final String label; final VoidCallback onTap; @override Widget build(BuildContext context) => InkWell(onTap: onTap, child: Container(height: 64, decoration: BoxDecoration(color: const Color(0xFF111C2C), borderRadius: BorderRadius.circular(14), border: Border.all(color: _line)), child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [Icon(icon, color: _text), const SizedBox(height: 8), Text(label, style: const TextStyle(color: _muted, fontSize: 12))]))); }
class _PathSelector extends StatelessWidget { const _PathSelector({required this.path, required this.onTap}); final String path; final VoidCallback onTap; @override Widget build(BuildContext context) => InkWell(onTap: onTap, child: Container(height: 38, padding: const EdgeInsets.symmetric(horizontal: 12), decoration: BoxDecoration(color: const Color(0xFF101A2A), borderRadius: BorderRadius.circular(8), border: Border.all(color: _line)), child: Row(children: [Expanded(child: Text(path, style: const TextStyle(color: _muted, fontSize: 12))), const Icon(Icons.more_horiz, color: _text)]))); }
class _GameRow extends StatelessWidget { const _GameRow({required this.game, required this.desktop, required this.selected, required this.onTap, required this.onPlay}); final GameEntry game; final bool desktop; final bool selected; final VoidCallback onTap; final VoidCallback onPlay; @override Widget build(BuildContext context) => InkWell(onTap: onTap, borderRadius: BorderRadius.circular(10), child: Container(height: desktop ? 58 : 58, padding: const EdgeInsets.symmetric(horizontal: 10), decoration: BoxDecoration(color: selected ? const Color(0xCC1D2A5B) : Colors.transparent, borderRadius: BorderRadius.circular(10), border: selected ? Border.all(color: _accent) : null), child: Row(children: [Container(width: 44, height: 44, decoration: BoxDecoration(borderRadius: BorderRadius.circular(7), gradient: const LinearGradient(colors: [Color(0xFFE6862D), Color(0xFF38235F), Color(0xFF1FC5D6)])), child: Center(child: Text(game.initials, style: const TextStyle(fontWeight: FontWeight.w900, color: Colors.white)))), const SizedBox(width: 12), Expanded(flex: 3, child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [Text(game.title, maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(color: _text, fontWeight: FontWeight.w800, fontSize: 13)), Text(game.system, style: const TextStyle(color: _muted, fontSize: 11))])), if (desktop) Expanded(child: Text(game.core, style: const TextStyle(color: _muted, fontSize: 12))), if (desktop) Expanded(child: Text(game.lastPlayed, style: const TextStyle(color: _muted, fontSize: 12))), SizedBox(width: desktop ? 90 : 54, child: Text(game.playTime, textAlign: TextAlign.right, style: const TextStyle(color: _muted, fontSize: 11))), const SizedBox(width: 10), InkWell(onTap: onPlay, customBorder: const CircleBorder(), child: CircleAvatar(radius: 16, backgroundColor: const Color(0xFF152035), child: Icon(selected ? Icons.play_arrow : Icons.more_horiz, color: _text, size: 18)))])) ); }
class _CoreRow extends StatelessWidget { const _CoreRow({required this.core, required this.selected, required this.onTap}); final CoreEntry core; final bool selected; final VoidCallback onTap; @override Widget build(BuildContext context) => InkWell(onTap: onTap, borderRadius: BorderRadius.circular(12), child: Container(height: 52, padding: const EdgeInsets.symmetric(horizontal: 12), decoration: BoxDecoration(color: selected ? const Color(0xFF2C2368) : Colors.transparent, borderRadius: BorderRadius.circular(12), border: selected ? Border.all(color: const Color(0xFF6E8CFF)) : null), child: Row(children: [Container(width: 32, height: 32, decoration: BoxDecoration(borderRadius: BorderRadius.circular(10), gradient: const LinearGradient(colors: [_accent, _cyan])), child: const Icon(Icons.extension, size: 16)), const SizedBox(width: 12), Expanded(child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [Text(core.name, style: const TextStyle(color: _text, fontWeight: FontWeight.w900, fontSize: 12)), Text(core.system, style: const TextStyle(color: _muted, fontSize: 10))])), Icon(selected ? Icons.check_circle : Icons.info_outline, color: selected ? _accent : _muted, size: 18)]))); }
class _ViewportButton extends StatelessWidget { const _ViewportButton({required this.icon, required this.label, required this.onTap}); final IconData icon; final String label; final FutureOr<void> Function() onTap; @override Widget build(BuildContext context) => InkWell(onTap: () => onTap(), child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [CircleAvatar(radius: 17, backgroundColor: const Color(0xFF131F31), child: Icon(icon, size: 16, color: _text)), const SizedBox(height: 5), Text(label, style: const TextStyle(color: _muted, fontSize: 10))])); }
class _QuickAction extends StatelessWidget { const _QuickAction({required this.icon, required this.title, required this.subtitle, required this.onTap}); final IconData icon; final String title; final String subtitle; final FutureOr<void> Function() onTap; @override Widget build(BuildContext context) => ListTile(onTap: () => onTap(), leading: CircleAvatar(backgroundColor: const Color(0xFF16243A), child: Icon(icon, color: _text)), title: Text(title, style: const TextStyle(fontWeight: FontWeight.w800)), subtitle: Text(subtitle), trailing: const Icon(Icons.chevron_right)); }
class _SettingCategory extends StatelessWidget { const _SettingCategory({required this.label, required this.selected}); final String label; final bool selected; @override Widget build(BuildContext context) => Container(height: 28, margin: const EdgeInsets.only(bottom: 4), padding: const EdgeInsets.symmetric(horizontal: 10), decoration: BoxDecoration(color: selected ? const Color(0xFF2B2364) : Colors.transparent, borderRadius: BorderRadius.circular(6)), alignment: Alignment.centerLeft, child: Text(label, style: TextStyle(color: selected ? _text : _muted, fontSize: 12))); }
class _ThemeCard extends StatelessWidget { const _ThemeCard({required this.label, required this.selected}); final String label; final bool selected; @override Widget build(BuildContext context) => Container(height: 58, alignment: Alignment.bottomCenter, padding: const EdgeInsets.only(bottom: 6), decoration: BoxDecoration(borderRadius: BorderRadius.circular(9), border: Border.all(color: selected ? _accent : _line), gradient: const LinearGradient(colors: [Color(0xFF091223), Color(0xFF203151)])), child: Text(label, style: const TextStyle(fontSize: 11))); }
class _AssetPackageRow extends StatelessWidget { const _AssetPackageRow({required this.package, required this.destination, required this.onBundled, required this.onDownload}); final FrontendAssetPackageEntry package; final String destination; final FutureOr<void> Function() onBundled; final FutureOr<void> Function() onDownload; @override Widget build(BuildContext context) => Container(padding: const EdgeInsets.all(12), decoration: BoxDecoration(color: _panel2, borderRadius: BorderRadius.circular(12), border: Border.all(color: _line)), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Text('${package.name}.zip', style: const TextStyle(fontWeight: FontWeight.w900)), const SizedBox(height: 4), Text('${package.label} → $destination', maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(color: _muted, fontSize: 11)), const SizedBox(height: 10), Row(children: [Expanded(child: _SmallButton(label: '同梱ZIPを展開', onTap: onBundled)), const SizedBox(width: 8), Expanded(child: _SmallButton(label: '最新版を取得', onTap: onDownload))]) ])); }

class _SettingsGroup extends StatelessWidget {
  const _SettingsGroup({required this.title, required this.children});
  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.only(bottom: 16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text(title, style: const TextStyle(color: _text, fontSize: 16, fontWeight: FontWeight.w900)),
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(color: _panel2, borderRadius: BorderRadius.circular(14), border: Border.all(color: _line)),
            child: Column(children: children),
          ),
        ]),
      );
}

class _SettingsToggleRow extends StatelessWidget {
  const _SettingsToggleRow({required this.title, required this.subtitle, required this.value, required this.onChanged});
  final String title;
  final String subtitle;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) => SwitchListTile(
        contentPadding: EdgeInsets.zero,
        dense: true,
        title: Text(title, style: const TextStyle(color: _text, fontSize: 13)),
        subtitle: Text(subtitle, style: const TextStyle(color: _muted, fontSize: 11)),
        value: value,
        activeColor: _accent,
        onChanged: onChanged,
      );
}

class _SettingsCoreButton extends StatelessWidget {
  const _SettingsCoreButton({required this.core, required this.onTap});
  final CoreEntry core;
  final FutureOr<void> Function() onTap;

  @override
  Widget build(BuildContext context) => ListTile(
        dense: true,
        contentPadding: EdgeInsets.zero,
        leading: Icon(core.loaded ? Icons.check_circle : Icons.extension, color: core.loaded ? _accent : _muted, size: 18),
        title: Text(core.name, style: const TextStyle(color: _text, fontSize: 13)),
        subtitle: Text([core.system, core.supportedExtensions.take(6).join(', ')].where((part) => part.isNotEmpty).join(' • '), style: const TextStyle(color: _muted, fontSize: 11), maxLines: 1, overflow: TextOverflow.ellipsis),
        onTap: () => onTap(),
      );
}

class _SettingsAssetRow extends StatelessWidget { const _SettingsAssetRow({required this.package, required this.onBundled, required this.onDownload}); final FrontendAssetPackageEntry package; final FutureOr<void> Function() onBundled; final FutureOr<void> Function() onDownload; @override Widget build(BuildContext context) => ListTile(dense: true, contentPadding: EdgeInsets.zero, title: Text('${package.name}.zip', style: const TextStyle(color: _text, fontSize: 13)), subtitle: Text(package.label, style: const TextStyle(color: _muted, fontSize: 11)), trailing: Wrap(spacing: 8, children: [TextButton(onPressed: () => onBundled(), child: const Text('同梱')), TextButton(onPressed: () => onDownload(), child: const Text('取得'))])); }
class _SettingsChoiceRow extends StatelessWidget { const _SettingsChoiceRow({required this.title, required this.value, required this.choices, required this.onChanged}); final String title; final String value; final List<String> choices; final ValueChanged<String> onChanged; @override Widget build(BuildContext context) { final safeChoices = choices.isEmpty ? <String>[value] : choices; return ListTile(dense: true, contentPadding: EdgeInsets.zero, title: Text(title, style: const TextStyle(color: _text, fontSize: 13)), subtitle: Text(value, style: const TextStyle(color: _muted, fontSize: 11)), trailing: DropdownButton<String>(value: safeChoices.contains(value) ? value : safeChoices.first, dropdownColor: _panel2, underline: const SizedBox.shrink(), items: [for (final choice in safeChoices) DropdownMenuItem(value: choice, child: Text(choice))], onChanged: (value) { if (value != null) onChanged(value); })); } }
class _SettingsRow extends StatelessWidget { const _SettingsRow({required this.title, required this.value}); final String title; final String value; @override Widget build(BuildContext context) => ListTile(dense: true, contentPadding: EdgeInsets.zero, title: Text(title, style: const TextStyle(color: _text, fontSize: 13)), subtitle: Text(value, style: const TextStyle(color: _muted, fontSize: 11)), trailing: const Icon(Icons.chevron_right, color: _muted, size: 18)); }
class _PlaylistRow extends StatelessWidget { const _PlaylistRow({required this.entry}); final PlaylistEntry entry; @override Widget build(BuildContext context) => Container(height: 68, padding: const EdgeInsets.all(10), decoration: BoxDecoration(color: _panel2, borderRadius: BorderRadius.circular(10), border: Border.all(color: _line)), child: Row(children: [Container(width: 50, decoration: BoxDecoration(borderRadius: BorderRadius.circular(8), gradient: const LinearGradient(colors: [Color(0xFF322267), Color(0xFF0C3F3A)])), child: Center(child: Text(entry.icon, style: const TextStyle(fontSize: 22)))), const SizedBox(width: 12), Expanded(child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [Text(entry.name, style: const TextStyle(fontWeight: FontWeight.w800)), Text(entry.count, style: const TextStyle(color: _muted, fontSize: 12))])), const Icon(Icons.chevron_right, color: _muted)])); }
Widget _settingsPreview() => Center(child: Container(width: 250, height: 130, decoration: BoxDecoration(borderRadius: BorderRadius.circular(16), border: Border.all(color: _accent), gradient: LinearGradient(colors: [_cyan.withOpacity(.25), _accent.withOpacity(.22)])), child: const Icon(Icons.image, size: 46, color: _muted)));


class _EmptyLibrary extends StatelessWidget {
  const _EmptyLibrary({required this.onImport, required this.message});

  final VoidCallback onImport;
  final String message;

  @override
  Widget build(BuildContext context) => Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.inventory_2_outlined, color: _muted, size: 42),
            const SizedBox(height: 12),
            const Text('まだコンテンツがありません', style: TextStyle(color: _text, fontWeight: FontWeight.w900)),
            const SizedBox(height: 8),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 20),
              child: Text(message, textAlign: TextAlign.center, style: const TextStyle(color: _muted, fontSize: 12)),
            ),
            const SizedBox(height: 14),
            _SmallButton(label: 'インポート / 再スキャン', onTap: onImport),
          ],
        ),
      );
}

class _GameOverlay extends StatelessWidget { const _GameOverlay({required this.onButton, required this.onMenu}); final void Function(int, bool) onButton; final VoidCallback onMenu; @override Widget build(BuildContext context) => Positioned.fill(child: Stack(children: [Positioned(left: 28, bottom: 92, child: _OverlayButton(label: 'MENU', onTap: onMenu)), Positioned(right: 38, bottom: 115, child: _OverlayButton(label: 'A', onTap: () => onButton(8, true))), Positioned(right: 88, bottom: 80, child: _OverlayButton(label: 'B', onTap: () => onButton(0, true))), Positioned(left: 90, bottom: 130, child: _Dpad(onButton: onButton))])); }
class _OverlayButton extends StatelessWidget { const _OverlayButton({required this.label, required this.onTap}); final String label; final VoidCallback onTap; @override Widget build(BuildContext context) => InkWell(onTap: onTap, customBorder: const CircleBorder(), child: Container(width: 54, height: 54, alignment: Alignment.center, decoration: BoxDecoration(shape: BoxShape.circle, color: Colors.black.withOpacity(.34), border: Border.all(color: Colors.white24)), child: Text(label, style: const TextStyle(fontWeight: FontWeight.w900)))); }
class _Dpad extends StatelessWidget { const _Dpad({required this.onButton}); final void Function(int, bool) onButton; @override Widget build(BuildContext context) => SizedBox(width: 86, height: 86, child: Stack(children: [Positioned(left: 28, top: 0, child: _OverlayButton(label: '▲', onTap: () => onButton(4, true))), Positioned(left: 28, bottom: 0, child: _OverlayButton(label: '▼', onTap: () => onButton(5, true))), Positioned(left: 0, top: 28, child: _OverlayButton(label: '◀', onTap: () => onButton(6, true))), Positioned(right: 0, top: 28, child: _OverlayButton(label: '▶', onTap: () => onButton(7, true)))])); }

class _StorageBarsPainter extends CustomPainter { @override void paint(Canvas canvas, Size size) { final paint = Paint(); for (var i = 0; i < 22; i++) { paint.color = Color.lerp(_accent, _cyan, i / 22)!.withOpacity(.35 + i / 38); final h = (i * 13 % 54) + 6.0; canvas.drawRect(Rect.fromLTWH(i * (size.width / 24), size.height - h, 3, h), paint); } } @override bool shouldRepaint(covariant CustomPainter oldDelegate) => false; }
class _GameScenePainter extends CustomPainter { _GameScenePainter({required this.frame}); final int frame; @override void paint(Canvas canvas, Size size) { final p = Paint(); p.color = const Color(0xFF2C2E36); canvas.drawRect(Offset.zero & size, p); p.color = const Color(0xFF5B3C28); canvas.drawRect(Rect.fromLTWH(0, size.height * .65, size.width, size.height * .35), p); for (var i = 0; i < 8; i++) { p.color = Color.lerp(const Color(0xFF2F493E), const Color(0xFFB8904D), i / 7)!; canvas.drawRect(Rect.fromLTWH(i * size.width / 8, 0, size.width / 9, size.height * .66), p); } _fighter(canvas, Offset(size.width * .36 + (frame % 50) * .2, size.height * .55), const Color(0xFF3135A3)); _fighter(canvas, Offset(size.width * .73 - (frame % 40) * .18, size.height * .56), const Color(0xFFE66A22)); p.color = Colors.yellow; canvas.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(size.width * .12, 22, size.width * .30, 8), const Radius.circular(4)), p); canvas.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(size.width * .58, 22, size.width * .30, 8), const Radius.circular(4)), p); final tp = TextPainter(text: const TextSpan(text: '87', style: TextStyle(color: Color(0xFFFFED54), fontSize: 30, fontWeight: FontWeight.w900)), textDirection: TextDirection.ltr); tp.layout(); tp.paint(canvas, Offset(size.width / 2 - tp.width / 2, 10)); } void _fighter(Canvas c, Offset o, Color color) { final p = Paint()..color = color; c.drawCircle(o.translate(0, -34), 12, p); c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: o, width: 32, height: 55), const Radius.circular(12)), p); p.strokeWidth = 9; p.strokeCap = StrokeCap.round; c.drawLine(o.translate(-10, 24), o.translate(-30, 55), p); c.drawLine(o.translate(10, 24), o.translate(28, 55), p); c.drawLine(o.translate(-15, -10), o.translate(-42, -20), p); c.drawLine(o.translate(15, -10), o.translate(44, -8), p); } @override bool shouldRepaint(covariant _GameScenePainter oldDelegate) => oldDelegate.frame != frame; }
