# RetroArch と同じ UI を Retrofront で完全動作させるための最小作業計画

## 0. この計画の前提

この計画は、`Retrofront/frontend/menu/` がすでに `reference/RetroArch/` の UI ソースに追従していることを前提にする。したがって、`Retrofront/frontend/menu/` の更新、upstream 差分調査、追従運用、RetroArch 本体機能の移植計画はここには含めない。

現在必要なのは「完全な UI が動くこと」だけである。core 起動、実 playlist 管理、実 download、実 database scan、実 netplay、実 achievements、実 shader 編集保存などの本機能は、UI が完全に表示・遷移・操作できるようになってから実装する。この段階では、UI が必要とする値と成功応答を Rust 側の UI runtime が返し、画面・入力・アニメーション・テーマ・フォント・ダミーデータ・無効状態表示を破綻なく動かすことをゴールにする。

描画 backend は `wgpu` を使う。shader 連携は `librashader` を使うが、`librashader` の wgpu runtime は使わない。`wgpu` から Vulkan/Metal/D3D12 等の raw handle を取り出し、その raw handle を `librashader` の通常 runtime に渡す方針にする。

## 1. ゴール

`Retrofront/` で RetroArch と同じ UI を、実機能なしでも最後まで操作できる状態にする。

具体的な完成条件は次の通り。

- `xmb`、`ozone`、`rgui`、`materialui` の menu driver が起動できる。
- 各 driver でトップメニュー、設定、履歴、playlist、core 一覧、shader、オンライン更新、情報、終了確認などの主要画面に遷移できる。
- 実機能が未実装の項目は、クラッシュせず、空一覧・ダミー一覧・disabled 表示・「未実装」message のいずれかで UI として成立する。
- font、icon、wallpaper、thumbnail placeholder、theme color、animation、scroll、cursor、breadcrumb、sub-label、help text が表示される。
- keyboard、gamepad、mouse、touch の UI 操作が動く。
- window resize、DPI scale、safe area、orientation change に UI が追従する。
- `wgpu` renderer 上で menu の描画 command が安定して表示される。
- `librashader` は raw handle 経由で接続可能な設計にしておき、UI 上の shader 画面や preview が破綻しない。

## 2. 実装しないもの

UI 完全動作前には次を実装しない。

- libretro core の実起動。
- 実 ROM/content load。
- 実 playlist scan。
- 実 database query。
- 実 thumbnail download。
- 実 online updater。
- 実 netplay。
- 実 achievements login/sync。
- 実 shader preset の恒久保存。
- 実 save/state/screenshot/rewind。
- 実 audio mixer。
- 実 cloud/smb/network。

ただし、これらの画面を UI として表示するための dummy data、placeholder、disabled action、success/failure mock、アセットデータ読み込みは実装する。

## 3. 全体構成

```text
Retrofront/
  frontend/
    menu/                         # 既存の RetroArch UI 追従ソース。更新しない。
    menu/retrofront_menu_bridge.* # C UI から Rust UI runtime へ渡す橋渡し。
    renderer/                     # C 側の薄い描画 glue。
    rust/
      retrofront-core/
        src/
          ui_runtime/             # UI 完全動作用の状態・mock・設定・一覧。
          renderer/               # wgpu renderer。
          shader/                 # raw handle 経由 librashader 接続点。
          input/                  # UI 入力変換。
          assets/                 # UI asset/font/icon/thumbnail placeholder。
          platform/               # window/surface/raw handle 抽出。
```

責務は次のように分ける。

| 領域 | 役割 |
| --- | --- |
| `Retrofront/frontend/menu/` | UI の C ソース本体。既存追従済みとして扱い、この計画では更新しない。 |
| `retrofront_menu_bridge.*` | UI が必要とする RetroArch  API を受け、Rust UI runtime の mock/状態へ転送する。 |
| `ui_runtime` | UI 表示に必要な設定、一覧、ダミーデータ、画面遷移結果、message を保持する。 |
| `renderer` | `wgpu` で font、icon、quad、texture、clip、animation frame を描画する。 |
| `shader` | `wgpu` raw handle を取り出して `librashader` 通常 runtime に接続する境界を持つ。 |
| `input` | platform input を menu action に変換する。 |
| `assets` | RetroArch UI 互換の asset path 解決、font load、placeholder texture を提供する。 |

## 4. 作業 1: UI runtime の骨格を作る

1. Rust 側に `ui_runtime` module を作る。
2. `UiRuntime` struct を作る。
3. `UiRuntime` に現在の menu driver、window size、DPI scale、safe area、theme、language、selected index、navigation stack を持たせる。
4. `UiRuntime` に UI 専用の mock settings を持たせる。
5. `UiRuntime` に UI 専用の mock content data を持たせる。
6. `UiRuntime` に UI 専用の notification queue を持たせる。
7. C から使う opaque handle を作る。
8. `retrofront_ui_runtime_create` を作る。
9. `retrofront_ui_runtime_destroy` を作る。
10. `retrofront_ui_runtime_begin_frame` を作る。
11. `retrofront_ui_runtime_end_frame` を作る。
12. panic が C ABI を越えないようにする。
13. C へ返す文字列の lifetime policy を固定する。

## 5. 作業 2: C bridge を UI 専用 API に寄せる

1. `retrofront_menu_bridge.c` から Rust UI runtime を呼ぶ関数を集約する。
2. C UI が要求する設定 getter を Rust mock settings に接続する。
3. C UI が要求する設定 setter を Rust mock settings に接続する。
4. C UI が要求する path getter を Rust asset/path resolver に接続する。
5. C UI が要求する message push を Rust notification queue に接続する。
6. C UI が要求する list source を Rust mock list provider に接続する。
7. 実機能へ進もうとする action は UI runtime で受け止める。
8. 未実装 action は `未実装` message を出し、UI stack は壊さない。
9. bridge に本機能の実装を入れない。
10. bridge の関数は薄く保ち、状態は Rust に置く。

## 6. 作業 3: wgpu renderer を UI 表示専用で完成させる

1. `wgpu` instance、adapter、device、queue、surface を初期化する。
2. surface format、present mode、alpha mode を決める。
3. window resize 時に surface config を再作成する。
4. UI 用 orthographic projection を作る。
5. colored rectangle pipeline を作る。
6. textured rectangle pipeline を作る。
7. scissor/clip rect を command に含める。
8. alpha blend を RetroArch UI に近い見た目で設定する。
9. vertex buffer ring を作る。
10. index buffer ring を作る。
11. texture bind group cache を作る。
12. 1 frame の draw command list を Rust 側で受け取る。
13. command list を batch 化する。
14. present 前後の error handling を入れる。
15. lost/outdated surface 時に renderer を復旧する。
16. headless では command validation test を動かせるようにする。

## 7. 作業 4: C UI 描画 command を Rust renderer に渡す

1. C 側に `RetrofrontDrawCommand` enum を作る。
2. command 種別は `FillRect`、`DrawTexture`、`DrawText`、`SetClip`、`ClearClip`、`PushTransform`、`PopTransform` から始める。
3. C UI の既存描画呼び出しを command 生成へ変換する。
4. command buffer は Rust が所有し、C は append だけ行う。
5. command buffer overflow 時は frame を壊さず warning を出す。
6. text draw は最初 command として積み、Rust font renderer で処理する。
7. texture handle は Rust 発行の integer ID にする。
8. C pointer を renderer 内部 resource にしない。
9. frame 終端で Rust renderer が command buffer を consume する。
10. 同じ frame を screenshot できる debug path を用意する。

## 8. 作業 5: font と text を完成させる

1. RetroArch UI と同じ font asset を読み込める resolver を作る。
2. Latin、日本語、記号、絵文字 fallback の順序を決める。
3. glyph rasterizer を Rust 側に置く。
4. glyph atlas texture を `wgpu` texture として管理する。
5. text width API を C UI へ返す。
6. text height API を C UI へ返す。
7. line spacing と baseline を menu driver ごとに崩れない値にする。
8. ellipsis、marquee、scrolling text を破綻なく描画する。
9. sub-label の折り返しを検証する。
10. 右寄せ/中央寄せ/左寄せを検証する。
11. 日本語 UI 文字列で clipping しないか確認する。
12. font atlas overflow 時に atlas を拡張または再生成する。

## 9. 作業 6: assets と placeholder を揃える

1. RetroArch UI 互換の asset root を Rust で解決する。
2. menu driver ごとの theme directory を解決する。
3. icon texture を読み込む。
4. wallpaper texture を読み込む。
5. menu thumbnail placeholder を用意する。
6. missing icon placeholder を用意する。
7. missing thumbnail placeholder を用意する。
8. texture decode は Rust 側で行う。
9. texture upload は `wgpu` queue で行う。
10. asset reload を frame 境界で安全に行う。
11. theme switch 後に古い texture を破棄する。
12. asset missing 時は UI を止めず placeholder に差し替える。

## 10. 作業 7: input を UI 操作として完成させる

1. keyboard event を Rust input module に接続する。
2. gamepad event を Rust input module に接続する。
3. mouse event を Rust input module に接続する。
4. touch event を Rust input module に接続する。
5. 上下左右、決定、戻る、メニュー、検索、タブ切替を menu action に変換する。
6. key repeat を実装する。
7. analog stick threshold を実装する。
8. trigger/button repeat を実装する。
9. mouse hover と click を menu selection に反映する。
10. touch scroll、tap、long press、fling を実装する。
11. materialui 用の safe area/touch hit target を調整する。
12. input replay file を読み、同じ UI 操作をできるようにする。

## 11. 作業 8: UI 表示用 mock settings を作る

1. video、audio、input、user、directory、playlist、network、shader、menu の setting category を作る。
2. 各 category に UI 表示用 key/value を用意する。
3. bool setting を変更できるようにする。
4. int setting を左右キーで変更できるようにする。
5. enum setting を左右キーで変更できるようにする。
6. string/path setting は編集画面へ入れるが保存は mock に留める。
7. restart required 表示を出せるようにする。
8. disabled setting 表示を出せるようにする。
9. dependency により灰色になる setting を mock で再現する。
10. UI 操作中だけ値が保持される in-memory store にする。
11. 必要なら debug build だけ JSON snapshot を吐く。
12. 本物の設定ファイル保存は行わない。

## 12. 作業 9: UI 表示用 mock list provider を作る

1. main menu の項目を返す。
2. settings menu の階層を返す。
3. playlist 一覧の mock を返す。
4. playlist entry の mock を返す。
5. history の mock を返す。
6. favorites の mock を返す。
7. core 一覧の mock を返す。
8. contentless core の mock を返す。
9. shader preset 一覧の mock を返す。
10. online updater の mock 項目を返す。
11. information の mock 項目を返す。
12. directory browser の mock directory/file を返す。
13. empty state を返す mode を用意する。
14. large list を返す mode を用意する。
15. long label、long sub-label、日本語 label を含める。
16. thumbnail あり/なしの両方を含める。
17. disabled action を含める。
18. action 実行時は画面遷移、message、disabled のいずれかに変換する。

## 13. 作業 10: menu driver 4 種を順番に通す

### 13.1 rgui

1. 最初に `rgui` を起動する。
2. font/text/rect/list navigation を確認する。
3. 最小 command set で全階層を巡回する。
4. resize 後も崩れないことを確認する。

### 13.2 ozone

1. `ozone` を起動する。
2. sidebar、header、footer、thumbnail area を表示する。
3. long list scroll を確認する。
4. icon と thumbnail placeholder を確認する。

### 13.3 xmb

1. `xmb` を起動する。
2. horizontal category navigation を確認する。
3. icon animation と alpha を確認する。
4. wallpaper と theme color を確認する。

### 13.4 materialui

1. `materialui` を起動する。
2. touch 操作を確認する。
3. safe area と DPI scale を確認する。
4. portrait/landscape 相当の resize を確認する。

## 14. 作業 11: librashader 接続点を作る

1. `wgpu` device/queue/surface から backend raw handle を取得する設計にする。
2. Vulkan backend では instance/device/physical device/queue/command buffer/image view 等の必要 handle を整理する。
3. Metal backend では device/command queue/texture 等の必要 handle を整理する。
4. D3D12 backend では device/queue/resource handle 等の必要 handle を整理する。
5. `librashader` の wgpu runtime は使わない。
6. raw handle を `librashader` の通常 runtime に渡す wrapper を `shader` module に置く。
7. UI 完全動作段階では shader 実適用を必須にしない。
8. shader 画面では preset 一覧、parameter mock、preview placeholder を表示する。
9. raw handle 取得不能な環境では shader preview を disabled 表示にする。
10. renderer と shader の lifetime を分け、surface 再作成時に安全に再接続できるようにする。

## 15. 作業 12: 未実装機能を UI として成立させる

1. core 起動 action は `Core launch is not implemented yet` message にする。
2. content load action は file browser mock へ遷移する。
3. online updater action は progress mock を表示して完了 message を出す。
4. playlist scan action は progress mock を表示して完了 message を出す。
5. thumbnail download action は progress mock を表示して完了 message を出す。
6. achievements login action は disabled または mock error にする。
7. netplay action は disabled または mock error にする。
8. shader apply action は UI 上の selected state だけ更新する。
9. save/state/screenshot action は未実装 message にする。
10. exit action は confirmation UI だけ表示する。
11. どの未実装 action でも crash、無限 loop、stack 破壊を起こさない。

## 16. 作業 13: UI 自動巡回テストを作る

1. `rgui` 用 input replay を作る。
2. `ozone` 用 input replay を作る。
3. `xmb` 用 input replay を作る。
4. `materialui` 用 input replay を作る。
5. 各 replay で main menu 全 category を開く。
6. settings 階層を深さ 3 以上まで開く。
7. playlist mock を開く。
8. shader mock を開く。
9. online updater mock を開く。
10. information mock を開く。
11. disabled action を押す。
12. back navigation で root へ戻る。
13. replay 中に panic しないことを確認する。
14. replay 終了時に navigation stack が正常であることを確認する。

## 17. 作業 14: screenshot regression を作る

1. 各 menu driver の root 画面 screenshot を撮る。
2. settings 画面 screenshot を撮る。
3. playlist 画面 screenshot を撮る。
4. shader 画面 screenshot を撮る。
5. long list scroll 中 screenshot を撮る。
6. missing asset fallback screenshot を撮る。
7. Japanese label screenshot を撮る。
8. resize 後 screenshot を撮る。
9. touch layout screenshot を撮る。
10. baseline との差分を出す。
11. 初期段階は threshold を緩めにし、明らかな崩れだけ検出する。
12. UI が固まったら threshold を厳しくする。

## 18. 作業 15: 完了判定

UI 完全動作段階の完了条件は次の通り。

- 4 menu driver が起動する。
- 4 menu driver で root から主要画面へ遷移できる。
- mock settings を変更して UI 表示が更新される。
- mock list の empty/large/long label/Japanese label が表示できる。
- 未実装 action がすべて安全に message、disabled、mock progress のいずれかへ変換される。
- keyboard/gamepad/mouse/touch の操作が最低 1 driver 以上で確認され、driver 固有操作も確認される。
- `wgpu` renderer が resize、surface lost/outdated に耐える。
- font/icon/wallpaper/placeholder が missing 時も UI が止まらない。
- `librashader` raw handle 接続点が module として存在し、使わない環境では disabled fallback になる。
- UI replay test が panic なしで完走する。
- screenshot regression で主要画面の崩れを検出できる。

## 19. 作業順の最短ルート

1. `UiRuntime` と C opaque handle を作る。
2. C bridge を Rust UI runtime に接続する。
3. `wgpu` renderer を初期化する。
4. rect/texture/text の draw command を通す。
5. font atlas と text metrics を完成させる。
6. asset resolver と placeholder texture を完成させる。
7. keyboard/gamepad 入力を通す。
8. mock settings を返す。
9. mock list provider を返す。
10. `rgui` を完走させる。
11. `ozone` を完走させる。
12. `xmb` を完走させる。
13. `materialui` を完走させる。
14. mouse/touch を完成させる。
15. 未実装 action の message/disabled/mock progress を揃える。
16. `librashader` raw handle 接続点を作る。
17. UI replay test を作る。
18. screenshot regression を作る。
19. resize/DPI/safe area を固める。
20. UI 完全動作完了として、次段階の実機能実装へ進む。

## 20. まとめ

今の段階で必要なのは、RetroArch 本体機能を Rust に移植することではなく、既に追従している `Retrofront/frontend/menu/` を使って UI を完全に表示・遷移・操作できるようにすること。したがって作業は、`wgpu` renderer、font、assets、input、mock settings、mock list、未実装 action の安全な受け止め、4 menu driver の巡回、UI replay、screenshot regression に集中する。実 core 起動や実 playlist scan などは、UI が完全に動くことを確認してから次段階で実装する。
