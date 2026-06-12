import SwiftUI
import RetrofrontSwift

struct LibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let onImport: () -> Void

    var body: some View {
        AppScreen(title: "すべてのゲーム", subtitle: "\(runtime.availableGames.count)ゲーム") {
            HStack(spacing: 12) {
                PrimaryAction(title: "ROMをインポート", subtitle: "ドラッグ＆ドロップ相当", icon: "square.and.arrow.down.fill", tint: RetroArchMenuPalette.driver("materialui").accent) {
                    onImport()
                }
                PrimaryAction(title: "スキャン", subtitle: "ライブラリを更新", icon: "arrow.clockwise", tint: RetroArchMenuPalette.teal) {
                    runtime.rescanLibrary()
                }
            }

            StatusPill(message: runtime.statusMessage)

            HStack(spacing: 10) {
                LibraryStatCard(title: "ROMs", value: "\(runtime.availableGames.count)", icon: "gamecontroller.fill", tint: RetroArchMenuPalette.driver("materialui").accent)
                LibraryStatCard(title: "Types", value: "\(runtime.libraryRomTypeCount)", icon: "doc.fill", tint: RetroArchMenuPalette.violet)
            }

            ContentSection(title: "タイトル", count: runtime.availableGames.count) {
                if runtime.availableGames.isEmpty {
                    EmptyPanel(icon: "tray", title: "No ROMs", message: "Importから.gbaなどのROMだけを追加してください。coresやinfoはライブラリに表示されません。")
                } else {
                    VStack(spacing: 10) {
                        ForEach(runtime.availableGames, id: \.path) { game in
                            GameRow(
                                game: game,
                                details: runtime.romDetails(for: game),
                                compatibility: runtime.compatibleCoreSummary(for: game)
                            ) {
                                runtime.loadGame(at: URL(fileURLWithPath: game.path))
                            }
                        }
                    }
                }
            }
        }
    }
}

struct PlaylistSummary: Identifiable {
    let id: String
    let title: String
    let count: String
    let icon: String
    let tint: Color
}

