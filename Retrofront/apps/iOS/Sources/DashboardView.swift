import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers
import UIKit

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var selectedTab = 0
    @State private var isPlayViewActive = false
    @State private var isImporterPresented = false

    var body: some View {
        TabView(selection: $selectedTab) {
            LibraryView {
                isImporterPresented = true
            }
                .tabItem { Label("ライブラリ", systemImage: "rectangle.stack.fill") }
                .tag(0)

            PlaylistDashboardView()
                .tabItem { Label("プレイリスト", systemImage: "list.bullet.rectangle.fill") }
                .tag(1)

            CoreDashboardView()
                .tabItem { Label("コア", systemImage: "cpu.fill") }
                .tag(2)

            SettingsView()
                .tabItem { Label("設定", systemImage: "gearshape.fill") }
                .tag(3)
        }
        .accentColor(OneUI.accent)
        .preferredColorScheme(.dark)
        .toolbarBackground(OneUI.surface, for: .tabBar)
        .toolbarBackground(.visible, for: .tabBar)
        .background(OneUI.background.ignoresSafeArea())
        .fullScreenCover(isPresented: $isPlayViewActive) { PlayView() }
        .fileImporter(isPresented: $isImporterPresented, allowedContentTypes: [.data]) { result in
            switch result {
            case .success(let url):
                runtime.importFile(at: url)
            case .failure(let error):
                runtime.statusMessage = "Import failed: \(error)"
            }
        }
        .sheet(isPresented: Binding(get: { runtime.pendingContentURL != nil }, set: { if !$0 { runtime.cancelCoreChoice() } })) {
            CoreChoiceSheet()
        }
        .onReceive(runtime.$launchToken) { token in
            if token > 0 { isPlayViewActive = true }
        }
    }
}

enum OneUI {
    static let background = Color(red: 0.030, green: 0.036, blue: 0.055)
    static let surface = Color(red: 0.075, green: 0.088, blue: 0.125)
    static let elevated = Color(red: 0.105, green: 0.122, blue: 0.170)
    static let ink = Color(red: 0.930, green: 0.950, blue: 0.990)
    static let secondary = Color(red: 0.650, green: 0.700, blue: 0.790)
    static let muted = Color(red: 0.430, green: 0.490, blue: 0.610)
    static let accent = Color(red: 0.250, green: 0.600, blue: 1.000)
    static let teal = Color(red: 0.130, green: 0.830, blue: 0.830)
    static let violet = Color(red: 0.620, green: 0.500, blue: 1.000)
    static let pink = Color(red: 0.930, green: 0.280, blue: 0.520)
    static let amber = Color(red: 0.950, green: 0.650, blue: 0.120)
    static let radius: CGFloat = 24
    static let compactRadius: CGFloat = 18

    static var ashChromeBackground: some View {
        ZStack {
            Color(red: 0.055, green: 0.060, blue: 0.070)
            LinearGradient(colors: [Color.white.opacity(0.10), Color.black.opacity(0.35)], startPoint: .top, endPoint: .bottom)
            RadialGradient(colors: [Color(red: 0.62, green: 0.68, blue: 0.78).opacity(0.22), .clear], center: .topLeading, startRadius: 20, endRadius: 440)
            RadialGradient(colors: [Color(red: 0.22, green: 0.26, blue: 0.32).opacity(0.42), .clear], center: .bottomTrailing, startRadius: 60, endRadius: 380)
        }
    }

    static var auroraBackground: some View {
        ZStack {
            background
            RadialGradient(colors: [violet.opacity(0.36), .clear], center: .topLeading, startRadius: 20, endRadius: 420)
            RadialGradient(colors: [teal.opacity(0.28), .clear], center: .bottomTrailing, startRadius: 40, endRadius: 360)
        }
    }
}

struct AppScreen<Content: View>: View {
    let title: String
    let subtitle: String
    @ViewBuilder var content: Content

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(title)
                            .font(.system(size: 34, weight: .bold, design: .default))
                            .foregroundColor(OneUI.ink)
                        Text(subtitle)
                            .font(.subheadline.weight(.medium))
                            .foregroundColor(OneUI.secondary)
                    }
                    .padding(.top, 16)

                    content
                }
                .padding(.horizontal, 18)
                .padding(.bottom, 28)
            }
            .background(OneUI.auroraBackground.ignoresSafeArea())
            .toolbar(.hidden, for: .navigationBar)
        }
    }
}

struct LibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let onImport: () -> Void

    var body: some View {
        AppScreen(title: "すべてのゲーム", subtitle: "\(runtime.availableGames.count)ゲーム") {
            HStack(spacing: 12) {
                PrimaryAction(title: "ROMをインポート", subtitle: "ドラッグ＆ドロップ相当", icon: "square.and.arrow.down.fill", tint: OneUI.accent) {
                    onImport()
                }
                PrimaryAction(title: "スキャン", subtitle: "ライブラリを更新", icon: "arrow.clockwise", tint: OneUI.teal) {
                    runtime.rescanLibrary()
                }
            }

            StatusPill(message: runtime.statusMessage)

            HStack(spacing: 10) {
                LibraryStatCard(title: "ROMs", value: "\(runtime.availableGames.count)", icon: "gamecontroller.fill", tint: OneUI.accent)
                LibraryStatCard(title: "Types", value: "\(runtime.libraryRomTypeCount)", icon: "doc.fill", tint: OneUI.violet)
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

struct NowPlayingView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPlayViewActive: Bool

    var body: some View {
        AppScreen(title: "Play", subtitle: "現在のゲームセッション") {
            VStack(alignment: .leading, spacing: 18) {
                ZStack {
                    RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous)
                        .fill(Color.black)
                        .aspectRatio(16.0 / 10.0, contentMode: .fit)
                    if let image = runtime.displayImage {
                        Image(uiImage: image)
                            .resizable()
                            .interpolation(.none)
                            .scaledToFit()
                            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
                    } else {
                        VStack(spacing: 10) {
                            Image(systemName: "display")
                                .font(.system(size: 34, weight: .semibold))
                            Text(runtime.loadedGameURL == nil ? "No game loaded" : "Ready to render")
                                .font(.headline)
                        }
                        .foregroundColor(.white.opacity(0.78))
                    }
                }

                VStack(alignment: .leading, spacing: 6) {
                    Text(runtime.loadedGameURL?.lastPathComponent ?? "ゲーム未選択")
                        .font(.title3.bold())
                        .foregroundColor(OneUI.ink)
                    Text(runtime.systemInfo?.libraryName ?? "Libraryからゲームを選択すると起動します。")
                        .font(.subheadline)
                        .foregroundColor(OneUI.secondary)
                }

                Button {
                    if runtime.loadedGameURL != nil { isPlayViewActive = true }
                } label: {
                    Label(runtime.loadedGameURL == nil ? "Select a game from Library" : "Open Player", systemImage: "play.circle.fill")
                        .font(.headline)
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 15)
                        .background(Capsule().fill(runtime.loadedGameURL == nil ? OneUI.muted : OneUI.accent))
                }
                .buttonStyle(.plain)
                .disabled(runtime.loadedGameURL == nil)
            }
            .padding(18)
            .background(OneUI.surface)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
            .shadow(color: .black.opacity(0.05), radius: 18, y: 10)
        }
    }
}

struct CoreChoiceSheet: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.presentationMode) private var presentationMode

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 12) {
                    ForEach(runtime.pendingCoreChoices, id: \.path) { core in
                        Button {
                            runtime.launchPendingContent(with: core)
                            presentationMode.wrappedValue.dismiss()
                        } label: {
                            CoreRow(core: core)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(18)
            }
            .background(OneUI.background)
            .navigationBarTitle("Select Core", displayMode: .inline)
            .navigationBarItems(leading: Button("Cancel") { runtime.cancelCoreChoice() })
        }
        .navigationViewStyle(.stack)
    }
}

struct PrimaryAction: View {
    let title: String
    let subtitle: String
    let icon: String
    let tint: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: icon)
                    .font(.system(size: 17, weight: .bold))
                    .foregroundColor(.white)
                    .frame(width: 38, height: 38)
                    .background(Circle().fill(tint))
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(.headline).foregroundColor(OneUI.ink)
                    Text(subtitle).font(.caption.weight(.medium)).foregroundColor(OneUI.secondary)
                }
                Spacer(minLength: 0)
            }
            .padding(14)
            .frame(maxWidth: .infinity, minHeight: 76)
            .background(OneUI.surface)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
            .shadow(color: .black.opacity(0.05), radius: 14, y: 8)
        }
        .buttonStyle(.plain)
    }
}

struct StatusPill: View {
    let message: String

    var body: some View {
        Label(message, systemImage: "info.circle.fill")
            .font(.footnote.weight(.medium))
            .foregroundColor(OneUI.secondary)
            .lineLimit(2)
            .padding(.horizontal, 14)
            .padding(.vertical, 11)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Capsule().fill(OneUI.surface))
    }
}

struct ContentSection<Content: View>: View {
    let title: String
    let count: Int
    @ViewBuilder var content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(title)
                    .font(.title3.bold())
                    .foregroundColor(OneUI.ink)
                Spacer()
                Text("\(count)")
                    .font(.caption.bold())
                    .foregroundColor(OneUI.secondary)
                    .padding(.horizontal, 9)
                    .padding(.vertical, 5)
                    .background(Capsule().fill(OneUI.surface))
            }
            content
        }
    }
}

struct GameRow: View {
    let game: GameEntrySwift
    let details: String
    let compatibility: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 14) {
                RoundedIcon(systemName: "play.fill", tint: OneUI.accent)
                VStack(alignment: .leading, spacing: 4) {
                    Text(game.label)
                        .font(.headline)
                        .foregroundColor(OneUI.ink)
                        .lineLimit(1)
                    Text(details)
                        .font(.caption)
                        .foregroundColor(OneUI.secondary)
                        .lineLimit(1)
                    Text(compatibility)
                        .font(.caption2.weight(.semibold))
                        .foregroundColor(compatibility.hasPrefix("No") ? .orange : OneUI.teal)
                        .lineLimit(1)
                }
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.caption.bold())
                    .foregroundColor(OneUI.muted)
            }
            .padding(15)
            .background(OneUI.surface)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
        }
        .buttonStyle(.plain)
    }
}

struct CoreRow: View {
    let core: CoreInfo

    var body: some View {
        HStack(spacing: 14) {
            RoundedIcon(systemName: "cpu.fill", tint: OneUI.violet)
            VStack(alignment: .leading, spacing: 4) {
                Text(core.displayName)
                    .font(.headline)
                    .foregroundColor(OneUI.ink)
                    .lineLimit(1)
                Text([core.systemName, core.supportedExtensions.joined(separator: ", ")].filter { !$0.isEmpty }.joined(separator: " • "))
                    .font(.caption)
                    .foregroundColor(OneUI.secondary)
                    .lineLimit(2)
            }
            Spacer(minLength: 0)
        }
        .padding(15)
        .background(OneUI.surface)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
    }
}

struct LibraryStatCard: View {
    let title: String
    let value: String
    let icon: String
    let tint: Color

    var body: some View {
        HStack(spacing: 10) {
            RoundedIcon(systemName: icon, tint: tint)
            VStack(alignment: .leading, spacing: 2) {
                Text(value).font(.title3.bold()).foregroundColor(OneUI.ink)
                Text(title).font(.caption.weight(.semibold)).foregroundColor(OneUI.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
        .frame(maxWidth: .infinity)
        .background(OneUI.surface)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.compactRadius, style: .continuous))
    }
}

struct RoundedIcon: View {
    let systemName: String
    let tint: Color

    var body: some View {
        Image(systemName: systemName)
            .font(.system(size: 16, weight: .bold))
            .foregroundColor(tint)
            .frame(width: 42, height: 42)
            .background(RoundedRectangle(cornerRadius: 14, style: .continuous).fill(tint.opacity(0.12)))
    }
}

struct EmptyPanel: View {
    let icon: String
    let title: String
    let message: String

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 28, weight: .semibold))
                .foregroundColor(OneUI.muted)
            Text(title)
                .font(.headline)
                .foregroundColor(OneUI.ink)
            Text(message)
                .font(.subheadline)
                .multilineTextAlignment(.center)
                .foregroundColor(OneUI.secondary)
        }
        .frame(maxWidth: .infinity)
        .padding(24)
        .background(OneUI.surface)
        .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
    }
}
