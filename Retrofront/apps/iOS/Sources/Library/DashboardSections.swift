import SwiftUI
import RetrofrontSwift

struct PlaylistDashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    private var playlists: [PlaylistSummary] {
        [
            PlaylistSummary(id: "favorites", title: "お気に入り", count: "\(max(runtime.availableGames.count / 3, 0))ゲーム", icon: "heart.fill", tint: OneUI.pink),
            PlaylistSummary(id: "recent", title: "最近プレイ", count: "\(min(runtime.availableGames.count, 15))ゲーム", icon: "clock.fill", tint: OneUI.teal),
            PlaylistSummary(id: "completed", title: "クリア済み", count: "\(max(runtime.availableGames.count / 4, 0))ゲーム", icon: "checkmark.seal.fill", tint: OneUI.violet),
            PlaylistSummary(id: "arcade", title: "アーケード", count: "\(runtime.availableGames.filter { $0.label.localizedCaseInsensitiveContains("arcade") }.count)ゲーム", icon: "gamecontroller.fill", tint: OneUI.accent),
            PlaylistSummary(id: "rpg", title: "RPG", count: "\(runtime.availableGames.filter { $0.label.localizedCaseInsensitiveContains("rpg") }.count)ゲーム", icon: "sparkles", tint: OneUI.amber),
            PlaylistSummary(id: "retro", title: "レトロアクション", count: "\(runtime.availableGames.count)ゲーム", icon: "flag.checkered", tint: OneUI.muted),
        ]
    }

    var body: some View {
        AppScreen(title: "プレイリスト", subtitle: "遊び方ごとにライブラリを整理") {
            VStack(spacing: 12) {
                ForEach(playlists) { playlist in
                    HStack(spacing: 14) {
                        RoundedIcon(systemName: playlist.icon, tint: playlist.tint)
                        VStack(alignment: .leading, spacing: 4) {
                            Text(playlist.title).font(.headline).foregroundColor(OneUI.ink)
                            Text(playlist.count).font(.caption.weight(.medium)).foregroundColor(OneUI.secondary)
                        }
                        Spacer()
                        Image(systemName: "chevron.right").font(.caption.bold()).foregroundColor(OneUI.muted)
                    }
                    .padding(15)
                    .background(OneUI.surface.opacity(0.92))
                    .overlay(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous).stroke(Color.white.opacity(0.06)))
                    .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
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

