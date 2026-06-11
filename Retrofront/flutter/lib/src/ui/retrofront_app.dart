import 'dart:async';
import 'dart:io' show Platform;
import 'dart:ui' as ui;

import 'package:file_picker/file_picker.dart';
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
    _coreOptions = await widget.frontend.coreOptions();
    _selectedGame ??= widget.frontend.games.isNotEmpty ? widget.frontend.games.first : null;
    _selectedCore ??= widget.frontend.cores.isNotEmpty ? widget.frontend.cores.first : null;
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    _frameTimer?.cancel();
    super.dispose();
  }


  Future<void> _launchGame(GameEntry game) async {
    final ok = await widget.frontend.launch(game);
    if (!mounted) return;
    setState(() => _selectedGame = game);
    if (ok) {
      await Navigator.of(context).push(MaterialPageRoute<void>(
        fullscreenDialog: true,
        builder: (_) => _PlayScreen(frontend: widget.frontend, game: game),
      ));
      if (mounted) setState(() {});
    }
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
                    Expanded(flex: 5, child: _gameViewport()),
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
    return Column(
      children: [
        Expanded(child: _libraryPanel(desktop: false)),
        const SizedBox(height: 10),
        SizedBox(height: 220, child: _gameViewport()),
      ],
    );
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
                    onTap: () => setState(() => _selectedGame = game),
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
            _coreOptions = await widget.frontend.coreOptions();
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
          Align(alignment: Alignment.bottomRight, child: FloatingActionButton(backgroundColor: _accent, onPressed: () {}, child: const Icon(Icons.add))),
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
        return _CoreRow(core: core, selected: selected, onTap: () => setState(() => _selectedCore = core));
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
    final categories = ['ユーザーインターフェース', 'ビデオ', 'オーディオ', '入力', 'コア', 'システム', 'プレイリスト'];
    final rows = [
      ('テーマ', 'ダーク / ライト / システム'),
      ('アクセントカラー', '紫・青・シアン・緑・黄・橙・赤'),
      ('透明度（グラス効果）', '60%'),
      ('ビデオドライバ', widget.frontend.settings['video_driver'] ?? 'metal'),
      ('スケール', widget.frontend.settings['video_scale_mode'] ?? 'integer_fit'),
      ('オーディオ遅延', '${widget.frontend.settings['audio_latency_ms']} ms'),
      ('入力オーバーレイ', widget.frontend.settings['input_overlay_enable'] ?? 'true'),
      ('セーブディレクトリ', widget.frontend.settings['savefile_directory'] ?? 'saves'),
      ('ステートディレクトリ', widget.frontend.settings['savestate_directory'] ?? 'states'),
    ];
    return _GlassPanel(
      child: Padding(
        padding: EdgeInsets.all(desktop ? 22 : 14),
        child: desktop
            ? Row(children: [
                SizedBox(width: 210, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [const Text('設定', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w900)), const SizedBox(height: 16), for (final c in categories) _SettingCategory(label: c, selected: c == categories.first)])),
                const VerticalDivider(color: _line),
                Expanded(child: _settingsControls(rows)),
                const VerticalDivider(color: _line),
                SizedBox(width: 300, child: _settingsPreview()),
              ])
            : Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                const Text('設定', style: TextStyle(fontSize: 20, fontWeight: FontWeight.w900)),
                const SizedBox(height: 14),
                _settingsControls(rows),
              ]),
      ),
    );
  }

  Widget _settingsControls(List<(String, String)> rows) {
    return ListView(children: [
      const Text('テーマ', style: TextStyle(color: _muted, fontSize: 12)),
      const SizedBox(height: 8),
      Row(children: ['ダーク', 'ライト', 'システム'].map((t) => Expanded(child: Padding(padding: const EdgeInsets.only(right: 8), child: InkWell(onTap: () async { await widget.frontend.setSetting('theme', t == 'ダーク' ? 'dark' : t == 'ライト' ? 'light' : 'system'); if (mounted) setState(() {}); }, child: _ThemeCard(label: t, selected: widget.frontend.settings['theme'] == (t == 'ダーク' ? 'dark' : t == 'ライト' ? 'light' : 'system')))))).toList()),
      const SizedBox(height: 16),
      const Text('アクセントカラー', style: TextStyle(color: _muted, fontSize: 12)),
      const SizedBox(height: 10),
      Wrap(spacing: 13, children: const [Color(0xFF7B61FF), Color(0xFF4058FF), Color(0xFF18A9E6), Color(0xFF11CBD7), Color(0xFF5BD17E), Color(0xFFE1C13C), Color(0xFFE98525), Color(0xFFE9485D), Color(0xFFC653A8), Color(0xFF263247)].map((c) => InkWell(onTap: () async { await widget.frontend.setSetting('accent_color', c.value.toRadixString(16)); if (mounted) setState(() {}); }, child: CircleAvatar(radius: 9, backgroundColor: c))).toList()),
      const SizedBox(height: 18),
      Row(children: [const Text('透明度（グラス効果）', style: TextStyle(color: _muted, fontSize: 12)), Expanded(child: Slider(value: double.tryParse(widget.frontend.settings['glass_opacity'] ?? '0.60') ?? .60, activeColor: _accent, onChanged: (v) async { await widget.frontend.setSetting('glass_opacity', v.toStringAsFixed(2)); if (mounted) setState(() {}); })), Text('${(((double.tryParse(widget.frontend.settings['glass_opacity'] ?? '0.60') ?? .60) * 100).round())}%', style: const TextStyle(color: _text, fontSize: 12))]),
      const SizedBox(height: 6),
      _SettingsChoiceRow(title: 'ビデオドライバ', value: widget.frontend.settings['video_driver'] ?? 'metal', choices: const ['metal', 'opengl', 'vulkan', 'software'], onChanged: (v) async { await widget.frontend.setSetting('video_driver', v); if (mounted) setState(() {}); }),
      _SettingsChoiceRow(title: 'スケール', value: widget.frontend.settings['video_scale_mode'] ?? 'integer_fit', choices: const ['integer_fit', 'fit', 'fill', 'stretch'], onChanged: (v) async { await widget.frontend.setSetting('video_scale_mode', v); if (mounted) setState(() {}); }),
      _SettingsChoiceRow(title: 'オーディオ遅延', value: widget.frontend.settings['audio_latency_ms'] ?? '64', choices: const ['32', '64', '96', '128'], onChanged: (v) async { await widget.frontend.setSetting('audio_latency_ms', v); if (mounted) setState(() {}); }),
      SwitchListTile(contentPadding: EdgeInsets.zero, dense: true, title: const Text('入力オーバーレイ', style: TextStyle(color: _text, fontSize: 13)), subtitle: Text(widget.frontend.settings['input_overlay'] ?? '', style: const TextStyle(color: _muted, fontSize: 11), maxLines: 1, overflow: TextOverflow.ellipsis), value: (widget.frontend.settings['input_overlay_enable'] ?? 'true') == 'true', activeColor: _accent, onChanged: (v) async { await widget.frontend.setOverlayEnabled(v); if (mounted) setState(() {}); }),
      for (final row in rows.skip(7)) _SettingsRow(title: row.$1, value: row.$2),
    ]);
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

  Future<void> _pickRom() async {
    final result = await FilePicker.platform.pickFiles(allowMultiple: true);
    if (result == null) return;
    for (final file in result.files) {
      final path = file.path;
      if (path != null) await widget.frontend.importRom(path);
    }
    if (mounted) setState(() {});
  }

  Future<void> _pickRomDirectory() async {
    final path = await FilePicker.platform.getDirectoryPath();
    if (path == null) return;
    await widget.frontend.scanRoms(path);
    if (mounted) setState(() {});
  }

  Future<void> _showCoreOptions(BuildContext context) async {
    await showModalBottomSheet<void>(
      context: context,
      backgroundColor: const Color(0xFF081321),
      showDragHandle: true,
      builder: (context) => ListView(padding: const EdgeInsets.all(18), children: [
        const Text('起動中のコア設定', style: TextStyle(fontSize: 20, fontWeight: FontWeight.w900)),
        const SizedBox(height: 12),
        for (final option in _coreOptions)
          ListTile(
            title: Text(option.description),
            subtitle: Text(option.key),
            trailing: DropdownButton<String>(
              value: option.values.contains(option.value) ? option.value : option.values.firstOrNull,
              items: option.values.map((v) => DropdownMenuItem(value: v, child: Text(v))).toList(),
              onChanged: (value) async { if (value != null) await widget.frontend.setCoreOption(option.key, value); },
            ),
          ),
      ]),
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
      body: Stack(children: [
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
class _SettingsChoiceRow extends StatelessWidget { const _SettingsChoiceRow({required this.title, required this.value, required this.choices, required this.onChanged}); final String title; final String value; final List<String> choices; final ValueChanged<String> onChanged; @override Widget build(BuildContext context) => ListTile(dense: true, contentPadding: EdgeInsets.zero, title: Text(title, style: const TextStyle(color: _text, fontSize: 13)), subtitle: Text(value, style: const TextStyle(color: _muted, fontSize: 11)), trailing: DropdownButton<String>(value: choices.contains(value) ? value : choices.first, dropdownColor: _panel2, underline: const SizedBox.shrink(), items: [for (final choice in choices) DropdownMenuItem(value: choice, child: Text(choice))], onChanged: (value) { if (value != null) onChanged(value); })); }
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
