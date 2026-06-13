# RetroArch UI をそのまま使い、UI 以外を Rust に置き換えるためのロードマップ

## 0. ゴール定義

`reference/RetroArch/` の RetroArch UI 全域を可能な限り無改変で `Retrofront/` に取り込み、UI が期待する RetroArch 側のグローバル状態・設定・描画・入力・ファイルシステム・タスク・ネットワーク・libretro 管理などを Rust 実装で提供する。最終状態は「C の UI ソースは RetroArch 追従可能な薄い移植層として残し、実体のアプリケーション状態と副作用は Rust が所有する」構成にする。

重要な前提は次の通り。

- UI ソースは「仕様」として扱い、UI ロジックそのものの Rust 再実装を最初のゴールにしない。
- `reference/RetroArch/menu/` だけでは UI 全域ではない。メニューは `menu/`、描画は `gfx/` と `gfx/font/`、アセット/翻訳/シェーダ/設定/タスク/入力/音声/ネットワーク/playlist/database などに広く依存する。
- 依存先を一気に全置換しない。まず C UI をビルド可能にし、未実装依存を Rust 側の最小 stub で満たし、その後に stub を本実装へ置き換える。
- C と Rust の境界は C ABI に固定する。Rust の内部設計を C UI に漏らさず、C UI から見える関数・構造体・列挙値は RetroArch 互換の adapter 層で吸収する。
- ライセンス、著作権表示、RetroArch 追従手順を最初から運用に含める。

## 1. 成果物の完成形

### 1.1 ディレクトリ構成の目標

```text
Retrofront/
  frontend/
    menu/                 # RetroArch menu/ の追従コピー。原則無改変。
    ui_compat/            # RetroArch UI が include/call する互換 C shim。
    renderer/             # C から見える最小描画 ABI と必要な glue。
    rust/
      retrofront-core/    # Rust が所有する状態・描画・入力・FS・task・libretro。
reference/
  RetroArch/              # upstream 参照。直接編集しない。
docs/
  retroarch-ui-rust-replacement-roadmap.md
```

### 1.2 レイヤー境界

| レイヤー | 言語 | 役割 | 変更方針 |
| --- | --- | --- | --- |
| RetroArch UI source | C/C++/ObjC | menu drivers、menu callbacks、表示リスト、UI 状態遷移 | 原則 upstream 追従。局所 patch は `ui_compat` へ逃がす。 |
| Compatibility shim | C | RetroArch の関数名/型/ヘッダを満たし Rust ABI へ転送 | 小さく保つ。UI から見える互換面だけ実装。 |
| Rust core | Rust | アプリ状態、副作用、描画 backend、入力、FS、設定、task、libretro | 実装本体。テストを書く。 |
| Platform host | Swift/C/Rust | iOS/Linux window、イベントループ、surface 提供 | Rust core と C UI を起動する。 |

## 2. Phase 0: 現状調査と固定化

### 2.1 upstream 差分を確認する

1. `reference/RetroArch/` のコミット hash または取得元 tag を記録する。
2. `reference/RetroArch/menu/` と `Retrofront/frontend/menu/` の差分を取る。
3. 差分を次の 4 種に分類する。
   - 単純コピー漏れ。
   - Retrofront 用の意図的変更。
   - build を通すための一時 stub。
   - upstream 更新で解消すべき古い差分。
4. 差分分類表を `docs/retroarch-ui-upstream-diff.md` に作る。
5. 以後、UI ソース変更時は必ず差分分類表を更新する。

### 2.2 UI 全域の対象範囲を確定する

最低限対象に入れるもの。

- `reference/RetroArch/menu/`
  - `menu_driver.c`
  - `menu_displaylist.c`
  - `menu_setting.c`
  - `menu_shader.c`
  - `menu_explore.c`
  - `menu_contentless_cores.c`
  - `menu_screensaver.c`
  - `menu/cbs/*.c`
  - `menu/drivers/materialui.c`
  - `menu/drivers/ozone.c`
  - `menu/drivers/rgui.c`
  - `menu/drivers/xmb.c`
- `reference/RetroArch/gfx/` のうち UI 描画に必要な display/font/texture/video context API。
- `reference/RetroArch/assets/` 相当の theme/font/icon/wallpaper。
- `reference/RetroArch/intl/` 相当の翻訳文字列。
- `reference/RetroArch/configuration.*`、`settings.*`、`paths.*`、`file_path_special.*` 相当の設定/パス API。
- `reference/RetroArch/tasks/` 相当の非同期タスク API。
- playlist、database、favorites、history、core info、shader preset、cheevos 表示など UI が列挙するデータ source。

### 2.3 「UI 以外」の Rust 所有物を明文化する

Rust が所有するものを固定する。

- Window/surface/device/swapchain/context。
- GPU resources、texture upload、font atlas、shader pipeline。
- Input event、gamepad/touch/keyboard mapping、menu action 変換。
- Settings store、config load/save、runtime overrides。
- Path resolution、VFS、archive、network path、content scan。
- Task queue、download、scan、thumbnail load、database query。
- Playlist/history/favorites/contentless core list。
- libretro core load/unload、environment callback、audio/video/input callbacks。
- Audio mixer と menu sound。
- Logging、metrics、crash boundary。

## 3. Phase 1: ビルド境界を作る

### 3.1 C UI を独立 static library 化する

1. `Retrofront/frontend/menu/` を `retrofront-ui-c` という C static library として扱う。
2. C compiler flags を upstream に寄せる。
3. platform 固有 macro を最小化する。
4. 使用 driver を最初は 1 つに絞る。推奨順は `rgui`、`ozone`、`xmb`、`materialui`。
   - `rgui`: 依存が少なく最初の疎通向き。
   - `ozone`: 現代的 UI 検証向き。
   - `xmb`: assets と animation 依存が多い。
   - `materialui`: touch/mobile 検証向き。
5. `menu/drivers/*.c` は全て残すが、build feature で有効 driver を切り替える。

### 3.2 互換 header セットを作る

1. `Retrofront/frontend/ui_compat/include/` を作る。
2. RetroArch UI が include する header 名をそのまま配置する。
3. header の中身は次のどちらかにする。
   - upstream と ABI 互換が必要な struct/enum 定義。
   - Rust ABI へ転送する関数宣言。
4. `reference/RetroArch/` の巨大 header を無条件に include しない。
5. 依存が増えたら header ごとに owner を決め、stub か本実装かを台帳化する。

### 3.3 Rust ABI crate を固定する

1. `retrofront-core` に `crate-type = ["staticlib", "cdylib", "rlib"]` を設定する。
2. `retrofront_rust.h` を C 側唯一の Rust 入口にする。
3. ABI 関数名は `retrofront_` prefix に統一する。
4. C から Rust へ渡す pointer の lifetime を全て文書化する。
5. Rust panic は ABI を越えないように `catch_unwind` 境界または panic abort 方針を決める。
6. error は整数 code + thread-local/handle-based message で返す。

## 4. Phase 2: 依存関係の完全棚卸し

### 4.1 symbol 収集

1. C UI library を未解決 symbol を許す形で compile する。
2. `nm -u` または linker map で未解決 symbol 一覧を作る。
3. symbol を次の category に分ける。
   - settings/config。
   - paths/filesystem/VFS。
   - menu state/list/entries。
   - video/display/font/texture。
   - input。
   - task/thread/timer。
   - playlist/database/core info。
   - shader。
   - network/download。
   - audio/menu sound。
   - logging/message。
   - platform/window。
   - memory/string/list utility。
4. 依存ごとに「Rust 本実装」「C shim」「upstream common を残す」「削除」の方針を決める。

### 4.2 include 依存収集

1. `clang -M` または `cc -MMD` で include graph を生成する。
2. `reference/RetroArch/libretro-common/` へ依存する箇所を抽出する。
3. C utility として残してよいものを選ぶ。
   - 文字列 utility。
   - list/vector utility。
   - path utility の pure 関数。
   - hash/encoding の pure 関数。
4. 副作用を持つものは Rust へ移す。
   - file IO。
   - thread。
   - network。
   - platform API。
   - config 永続化。

### 4.3 dependency ledger を作る

`docs/retroarch-ui-dependency-ledger.md` に次の列で表を作る。

| UI symbol/header | upstream path | 用途 | owner | Rust module | 実装状態 | test |
| --- | --- | --- | --- | --- | --- | --- |

状態は `not_started`、`stub`、`partial`、`complete`、`upstream_c_kept` の 5 段階にする。

## 5. Phase 3: 最小起動ループ

### 5.1 Rust 側 AppState を作る

最初に必要な状態。

- `MenuRuntime`。
- `SettingsStore`。
- `PathStore`。
- `AssetStore`。
- `InputState`。
- `RendererHandle`。
- `TaskRuntime`。
- `PlaylistStore`。
- `CoreInfoStore`。

C には opaque handle だけ渡す。

### 5.2 起動 sequence

1. platform host が window/surface を作る。
2. Rust core を初期化する。
3. Rust core が renderer を初期化する。
4. C UI を初期化する。
5. menu driver を選択する。
6. assets path と config path を C UI に見せる。
7. main loop を開始する。
8. 1 frame ごとに次を実行する。
   - platform event poll。
   - Rust input update。
   - C UI menu iterate。
   - C UI が描画 command を出す。
   - C shim が Rust renderer へ command を転送する。
   - Rust renderer が present する。

### 5.3 最小 acceptance criteria

- 空の playlist でもメニュー画面が出る。
- 上下左右/決定/戻るが動く。
- 設定画面を開ける。
- 1 つの setting 値を変更できる。
- アプリ再起動後に変更が残る。
- 画面 resize に追従する。
- 終了時に leak sanitizer または Rust drop log で大きな leak がない。

## 6. Phase 4: 描画を Rust へ置き換える

### 6.1 C UI から見た描画 API を保つ

UI driver が期待する概念を維持する。

- viewport。
- scissor。
- font draw。
- text metrics。
- texture load/free/update。
- colored quad。
- image quad。
- icon atlas。
- menu animation alpha/transform。
- video frame background。
- shader preview/preset 表示。

### 6.2 Rust renderer に変換する

1. C 側 draw call を immediate に GPU 実行しない。
2. C shim で `MenuDrawCommand` に詰める。
3. Rust が frame 終端で command list を受け取る。
4. Rust が batching、texture bind、font atlas、pipeline 選択を行う。
5. backend は最初 `wgpu` に統一する。
6. iOS では Metal surface、Linux では Vulkan/Wayland/X11 surface を使う。
7. raw handle が必要な shader integration は Rust renderer 内部に閉じ込める。

### 6.3 font/text

1. RetroArch と同じ font file と fallback 順を使う。
2. glyph metrics の差が UI 崩れの最大原因なので snapshot test を作る。
3. HarfBuzz 等の shaping が必要な言語を後回しにしない。
4. C UI には text width/height API だけ見せる。
5. Rust 側で atlas eviction 方針を決める。

### 6.4 texture/assets

1. assets zip または directory の layout を RetroArch 互換にする。
2. icon 名から asset path への解決は Rust が行う。
3. PNG/JPEG/TGA decode は Rust crate に寄せる。
4. C UI の texture handle は整数 ID にする。
5. reload、theme switch、DPI scale change を test する。

## 7. Phase 5: 入力を Rust へ置き換える

1. platform event を Rust が受け取る。
2. keyboard/gamepad/touch/mouse を `InputEvent` に正規化する。
3. `InputEvent` を RetroArch menu action に変換する。
4. key repeat、hold acceleration、analog threshold を設定化する。
5. touch gesture は `materialui` 用に tap/long press/scroll/fling を実装する。
6. C UI には `menu_input_state` 相当の問い合わせ結果だけ返す。
7. hotkey と menu navigation の優先順位を決める。
8. input recording/replay test を作る。

## 8. Phase 6: 設定とパスを Rust へ置き換える

### 8.1 設定 store

1. RetroArch の setting key を Rust の typed schema に写像する。
2. unknown key を落とさず保存できる escape hatch を用意する。
3. UI の callback が参照する setting は全て ledger に登録する。
4. bool/int/float/string/path/enum の getter/setter ABI を作る。
5. setting change event を Rust core 内で publish する。
6. C UI から直接 global settings struct を変更させない。

### 8.2 path store

1. config、assets、cores、system、saves、states、playlists、thumbnails、shaders、logs を定義する。
2. iOS sandbox と Linux XDG の path policy を分ける。
3. relative path と absolute path の normalize を Rust に統一する。
4. content path display 用の短縮表示 API を用意する。
5. permission error を UI message に変換する。

## 9. Phase 7: メニュー data source を Rust へ置き換える

### 9.1 menu display list

1. C UI は display list を組み立てるが、元データは Rust が提供する。
2. cores、playlists、history、favorites、settings、directories を Rust API で列挙する。
3. C 側には `label`、`sublabel`、`path`、`type`、`icon`、`flags` の配列として渡す。
4. 文字列 lifetime は「呼び出し中だけ有効」か「handle 解放まで有効」か統一する。
5. 大量 playlist では pagination/lazy loading を実装する。

### 9.2 playlist/history/favorites

1. RetroArch playlist format の read/write を Rust で実装する。
2. 既存 playlist を round-trip test する。
3. duplicate detection を設定化する。
4. thumbnail lookup と playlist entry を結びつける。
5. 最近使った項目は atomic write する。

### 9.3 core info

1. core directory を scan する。
2. `.info` parser を Rust で実装する。
3. supported extensions、display name、firmware requirements を expose する。
4. contentless core を list する。
5. core missing/incompatible を UI に表示する。

## 10. Phase 8: task system を Rust へ置き換える

1. UI から task enqueue する ABI を作る。
2. Rust で async runtime または thread pool を選ぶ。
3. task type を enum 化する。
   - directory scan。
   - playlist scan。
   - thumbnail load/download。
   - core info refresh。
   - shader load。
   - archive listing。
   - network update。
4. progress、cancel、complete、error を pollable にする。
5. C UI の notification/message queue へ completion を戻す。
6. task 中の UI freeze がないことを frame time test で確認する。

## 11. Phase 9: shader と video preview

1. menu shader UI が参照する preset list を Rust が列挙する。
2. preset parse/validate を Rust に寄せる。
3. preview thumbnail または current core frame background を Rust renderer が提供する。
4. shader parameter setting を typed schema と同期する。
5. invalid shader は fallback 表示にする。
6. Vulkan/Metal の差は Rust renderer 内に閉じる。

## 12. Phase 10: libretro core 実行との統合

1. Rust が core lifecycle を所有する。
2. C UI からは「content load」「core unload」「run state change」だけ要求できるようにする。
3. libretro environment callback は Rust で実装する。
4. menu が必要とする core metadata を Rust が返す。
5. content launch 後も menu overlay を出せるよう renderer composition を作る。
6. save/state/screenshot/rewind/netplay 等の UI 項目は未実装時も明示的 disabled にする。

## 13. Phase 11: 翻訳・アクセシビリティ・テーマ完全化

1. `intl` の message hash と翻訳 table を Rust で load する。
2. fallback language を `en` にする。
3. missing translation を log する。
4. DPI scale、safe area、reduced motion、high contrast を Rust setting にする。
5. screen reader 連携が必要な platform では C UI の focus item を Rust host へ通知する。

## 14. Phase 12: upstream 追従運用

1. `reference/RetroArch/` を更新する手順を script 化する。
2. `menu/` 差分を自動生成する。
3. C UI に直接当てた patch は最小化し、必ず理由を書く。
4. dependency ledger を再生成する。
5. visual regression を走らせる。
6. 破壊的変更があれば Rust shim を更新する。
7. 追従 PR では「upstream 変更」「compat 変更」「Rust 実装変更」を commit 分割する。

## 15. Phase 13: テスト計画

### 15.1 unit test

- settings schema round-trip。
- path normalization。
- playlist parser/writer。
- `.info` parser。
- asset resolver。
- input mapping。
- task state machine。
- C ABI string/handle lifetime。

### 15.2 integration test

- C UI static library link test。
- Rust core + C UI init/shutdown test。
- menu navigation replay test。
- setting change persistence test。
- playlist large data test。
- renderer command list validation。

### 15.3 visual regression

1. 決まった window size で各 menu driver を起動する。
2. 同じ input replay を流す。
3. screenshot を保存する。
4. reference image と pixel diff する。
5. font rendering 差を許容する threshold を決める。
6. iOS と Linux の baseline を別に持つ。

### 15.4 performance test

- cold start time。
- first menu frame time。
- large playlist scroll frame time。
- thumbnail load 中の frame drop。
- shader list 表示時間。
- memory peak。
- texture atlas 使用量。

## 16. Phase 14: リスクと対策

| リスク | 原因 | 対策 |
| --- | --- | --- |
| UI だけのつもりが RetroArch 全体を移植する規模になる | menu が global state と subsystems に強く依存する | dependency ledger で owner を決め、stub から段階移行する。 |
| upstream 追従不能になる | C UI を直接改変する | `ui_compat` へ変更を逃がし、menu 差分を常時監視する。 |
| 描画が微妙に違う | font metrics、texture sampling、DPI、animation timer 差 | visual regression と metrics test を早期導入する。 |
| Rust/C 境界でクラッシュする | lifetime、ownership、panic、thread 境界不備 | opaque handle、明示 free、ABI test、panic 方針を固定する。 |
| iOS で動かない | dynamic loading、sandbox、Metal surface、署名制約 | platform policy を Rust host に閉じ、iOS 実機 CI を用意する。 |
| 設定項目が多すぎる | RetroArch settings が巨大 | UI が実際に読む key から優先実装し、unknown key 保持を入れる。 |
| assets が揃わない | theme/icon/font path の違い | RetroArch 互換 layout を採用し、asset resolver test を作る。 |

## 17. 実装順チェックリスト

### Milestone A: 調査完了

- [ ] upstream 参照 commit を記録する。
- [ ] `menu/` 差分表を作る。
- [ ] include graph を作る。
- [ ] unresolved symbol 一覧を作る。
- [ ] dependency ledger を作る。

### Milestone B: C UI が link する

- [ ] C UI static library を作る。
- [ ] `ui_compat/include` を作る。
- [ ] Rust ABI header を固定する。
- [ ] 全未解決 symbol を stub で埋める。
- [ ] init/shutdown test を通す。

### Milestone C: 最小メニュー表示

- [ ] Rust renderer command list を作る。
- [ ] font/text metrics API を作る。
- [ ] asset resolver を作る。
- [ ] 1 driver でメニューを表示する。
- [ ] keyboard navigation を動かす。

### Milestone D: 実用メニュー

- [ ] settings store を実装する。
- [ ] path store を実装する。
- [ ] playlist/core info を実装する。
- [ ] task runtime を実装する。
- [ ] thumbnails を表示する。
- [ ] shader list を表示する。

### Milestone E: core 起動統合

- [ ] libretro core loader を Rust 所有にする。
- [ ] content launch を menu から要求できるようにする。
- [ ] core 実行画面と menu overlay を合成する。
- [ ] save/state/screenshot の UI 項目を接続する。

### Milestone F: 品質固定

- [ ] visual regression を CI に入れる。
- [ ] input replay test を CI に入れる。
- [ ] sanitizer または leak check を定期実行する。
- [ ] iOS 実機 build を確認する。
- [ ] upstream 追従 script を用意する。

## 18. 最初の 10 作業

1. `docs/retroarch-ui-dependency-ledger.md` の雛形を作る。
2. `reference/RetroArch/menu/` と `Retrofront/frontend/menu/` の差分を生成する。
3. `Retrofront/frontend/ui_compat/include/` を追加する。
4. C UI library の build target を追加する。
5. `rgui` だけを有効化して link error を収集する。
6. link error を category 分けして ledger に登録する。
7. Rust ABI の `retrofront_menu_runtime_create/destroy` を作る。
8. settings/path/logging の stub を Rust に作る。
9. renderer command list の型だけ作る。
10. init/shutdown integration test を追加する。

## 19. 判断基準

この方針を続けるかどうかは、Milestone C の時点で判断する。

続行してよい条件。

- C UI への直接 patch が小さい。
- 依存 ledger の 60% 以上が Rust owner として分類済み。
- 最小 driver が 60 FPS 近くで描画できる。
- font/asset 差分が visual regression で追える。
- Rust/C ABI の crash が再現 test で潰せる。

方針転換を検討する条件。

- menu source への直接改変が増え続ける。
- RetroArch global state の再現が UI 以外の本体移植に近づく。
- upstream 追従のたびに大規模 conflict が出る。
- 描画差分の原因が C UI 側 assumptions にあり shim で吸収できない。

## 20. まとめ

最短ルートは「RetroArch UI を C のまま固定して先に link し、未実装依存を Rust stub で全部埋め、画面を出してから stub を実装へ置き換える」こと。最初から全依存を完璧に Rust 化しようとすると、UI が起動する前に作業量が膨らむ。必ず dependency ledger、ABI 境界、visual regression、upstream 差分管理を先に作り、以後の移植を機械的に進められる状態にする。
