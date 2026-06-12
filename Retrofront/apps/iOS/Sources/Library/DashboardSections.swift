import SwiftUI
import RetrofrontSwift

struct PlaylistDashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    private var playlists: [PlaylistSummary] {
        [
            PlaylistSummary(id: "favorites", title: "お気に入り", count: "\(max(runtime.availableGames.count / 3, 0))ゲーム", icon: "heart.fill", tint: RetroArchMenuPalette.pink),
            PlaylistSummary(id: "recent", title: "最近プレイ", count: "\(min(runtime.availableGames.count, 15))ゲーム", icon: "clock.fill", tint: RetroArchMenuPalette.teal),
            PlaylistSummary(id: "completed", title: "クリア済み", count: "\(max(runtime.availableGames.count / 4, 0))ゲーム", icon: "checkmark.seal.fill", tint: RetroArchMenuPalette.violet),
            PlaylistSummary(id: "arcade", title: "アーケード", count: "\(runtime.availableGames.filter { $0.label.localizedCaseInsensitiveContains("arcade") }.count)ゲーム", icon: "gamecontroller.fill", tint: RetroArchMenuPalette.driver("materialui").accent),
            PlaylistSummary(id: "rpg", title: "RPG", count: "\(runtime.availableGames.filter { $0.label.localizedCaseInsensitiveContains("rpg") }.count)ゲーム", icon: "sparkles", tint: RetroArchMenuPalette.amber),
            PlaylistSummary(id: "retro", title: "レトロアクション", count: "\(runtime.availableGames.count)ゲーム", icon: "flag.checkered", tint: RetroArchMenuPalette.driver("materialui").muted),
        ]
    }

    var body: some View {
        AppScreen(title: "プレイリスト", subtitle: "遊び方ごとにライブラリを整理") {
            VStack(spacing: 12) {
                ForEach(playlists) { playlist in
                    HStack(spacing: 14) {
                        RoundedIcon(systemName: playlist.icon, tint: playlist.tint)
                        VStack(alignment: .leading, spacing: 4) {
                            Text(playlist.title).font(.headline).foregroundColor(RetroArchMenuPalette.driver("materialui").ink)
                            Text(playlist.count).font(.caption.weight(.medium)).foregroundColor(RetroArchMenuPalette.driver("materialui").secondary)
                        }
                        Spacer()
                        Image(systemName: "chevron.right").font(.caption.bold()).foregroundColor(RetroArchMenuPalette.driver("materialui").muted)
                    }
                    .padding(15)
                    .background(RetroArchMenuPalette.driver("materialui").surface.opacity(0.92))
                    .overlay(RoundedRectangle(cornerRadius: RetroArchMenuMetrics.compactRadius, style: .continuous).stroke(Color.white.opacity(0.06)))
                    .clipShape(RoundedRectangle(cornerRadius: RetroArchMenuMetrics.compactRadius, style: .continuous))
                }
            }
        }
    }
}

struct CoreDashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        AppScreen(title: "コア", subtitle: "wgpuレンダリングで利用するエミュレータコア") {
            StatusPill(message: "software / Metal / OpenGL / Vulkan / MoltenVK はwgpuファミリとして維持されます")
            ContentSection(title: "ロード可能なコア", count: runtime.availableCores.count) {
                if runtime.availableCores.isEmpty {
                    EmptyPanel(icon: "cpu", title: "コア未検出", message: "同梱コアをインストール、または設定から再スキャンしてください。")
                } else {
                    VStack(spacing: 10) {
                        ForEach(runtime.availableCores, id: \.path) { core in
                            CoreRow(core: core)
                        }
                    }
                }
            }
        }
    }
}

