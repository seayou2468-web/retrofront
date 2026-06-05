#if canImport(SwiftUI)
import SwiftUI
import RetroFrontCore

@main
public struct RetroFrontApp: App {
    @StateObject private var model = FrontendViewModel()
    public init() {}
    public var body: some Scene {
        WindowGroup { RootView().environmentObject(model).task { await model.bootstrap() } }
    }
}

@MainActor
public final class FrontendViewModel: ObservableObject {
    @Published public var games: [Game] = []
    @Published public var cores: [LibretroCore] = []
    @Published public var saveStates: [SaveState] = []
    @Published public var settings = FrontendSettings()
    @Published public var searchText = ""
    @Published public var selectedSystemID: String?
    @Published public var alertMessage: String?
    public private(set) var store: LibraryStore?

    public var filteredGames: [Game] {
        games.filter { game in
            (selectedSystemID == nil || game.systemID == selectedSystemID) && (searchText.isEmpty || game.title.localizedCaseInsensitiveContains(searchText))
        }
    }

    public func bootstrap() async {
        let store = await LibraryStore()
        self.store = store
        await refresh()
        await scanAll()
    }

    public func refresh() async {
        guard let store else { return }
        games = await store.games; cores = await store.cores; saveStates = await store.saveStates; settings = await store.settings
    }

    public func scanAll() async {
        guard let store else { return }
        do {
            let dirs = await store.directories
            let scanner = LibraryScanner()
            let report = try await scanner.scanROMs(at: dirs.roms, knownGames: await store.games)
            let artwork = await ArtworkService(artworkDirectory: dirs.artwork).attachLocalArtwork(to: report.discoveredGames)
            for game in artwork { try await store.upsert(game: game) }
            for core in try await scanner.scanCores(at: dirs.cores) { try await store.upsert(core: core) }
            await refresh()
        } catch { alertMessage = error.localizedDescription }
    }

    public func toggleFavorite(_ game: Game) async {
        guard let store else { return }
        var copy = game; copy.favorite.toggle()
        try? await store.upsert(game: copy)
        await refresh()
    }

    public func update(settings: FrontendSettings) async {
        guard let store else { return }
        do { try await store.update(settings: settings); await refresh() } catch { alertMessage = error.localizedDescription }
    }
}

public struct RootView: View {
    @EnvironmentObject private var model: FrontendViewModel
    public init() {}
    public var body: some View {
        TabView {
            NavigationStack { LibraryView() }.tabItem { Label("Library", systemImage: "square.grid.2x2") }
            NavigationStack { SystemsView() }.tabItem { Label("Systems", systemImage: "gamecontroller") }
            NavigationStack { CoresView() }.tabItem { Label("Cores", systemImage: "cpu") }
            NavigationStack { SavesView() }.tabItem { Label("Saves", systemImage: "tray.full") }
            NavigationStack { SettingsView() }.tabItem { Label("Settings", systemImage: "gearshape") }
        }
        .alert("RetroFront", isPresented: Binding(get: { model.alertMessage != nil }, set: { if !$0 { model.alertMessage = nil } })) { Button("OK", role: .cancel) {} } message: { Text(model.alertMessage ?? "") }
    }
}
#endif
