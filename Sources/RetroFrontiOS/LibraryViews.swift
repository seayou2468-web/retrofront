#if canImport(SwiftUI)
import SwiftUI
import RetroFrontCore

struct LibraryView: View {
    @EnvironmentObject private var model: FrontendViewModel
    @State private var importing = false
    private let columns = [GridItem(.adaptive(minimum: 150), spacing: 16)]
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                HeroHeader(games: model.games)
                if !model.games.filter(\.favorite).isEmpty { Shelf(title: "Favorites", games: model.games.filter(\.favorite)) }
                if !model.games.filter({ $0.lastPlayedAt != nil }).isEmpty { Shelf(title: "Continue Playing", games: model.games.filter { $0.lastPlayedAt != nil }.sorted { ($0.lastPlayedAt ?? .distantPast) > ($1.lastPlayedAt ?? .distantPast) }) }
                LazyVGrid(columns: columns, spacing: 16) { ForEach(model.filteredGames) { GameCard(game: $0) } }
            }.padding()
        }
        .navigationTitle("RetroFront")
        .searchable(text: $model.searchText, prompt: "Search games")
        .toolbar {
            Button { Task { await model.scanAll() } } label: { Label("Scan", systemImage: "arrow.clockwise") }
            Button { importing = true } label: { Label("Import", systemImage: "square.and.arrow.down") }
        }
        .fileImporter(isPresented: $importing, allowedContentTypes: [.data], allowsMultipleSelection: true) { result in
            Task { await importFiles(result) }
        }
        .overlay { if model.games.isEmpty { ContentUnavailableView("Import ROMs and cores", systemImage: "tray.and.arrow.down", description: Text("Place files in the app's Documents/ROMs and Documents/Cores folders or use Import.")) } }
    }

    private func importFiles(_ result: Result<[URL], Error>) async {
        guard let store = model.store else { return }
        do {
            let urls = try result.get()
            _ = try await LibraryScanner().importFiles(urls, into: await store.directories)
            await model.scanAll()
        } catch { model.alertMessage = error.localizedDescription }
    }
}

struct HeroHeader: View {
    let games: [Game]
    var body: some View {
        ZStack(alignment: .bottomLeading) {
            LinearGradient(colors: [.purple, .blue, .black], startPoint: .topLeading, endPoint: .bottomTrailing)
            VStack(alignment: .leading, spacing: 8) {
                Text("Multi-system iOS frontend").font(.caption).textCase(.uppercase).foregroundStyle(.white.opacity(0.75))
                Text("Your retro library, saves, overlays, shaders, achievements and netplay in one place.").font(.title2.bold()).foregroundStyle(.white)
                Text("\(games.count) games · \(Set(games.map(\.systemID)).count) systems").foregroundStyle(.white.opacity(0.85))
            }.padding()
        }.frame(height: 180).clipShape(RoundedRectangle(cornerRadius: 28, style: .continuous))
    }
}

struct Shelf: View {
    let title: String
    let games: [Game]
    var body: some View {
        VStack(alignment: .leading) {
            Text(title).font(.headline)
            ScrollView(.horizontal, showsIndicators: false) { HStack { ForEach(games) { GameCard(game: $0).frame(width: 150) } } }
        }
    }
}

struct GameCard: View {
    @EnvironmentObject private var model: FrontendViewModel
    let game: Game
    var body: some View {
        NavigationLink { GameDetailView(game: game) } label: {
            VStack(alignment: .leading, spacing: 10) {
                ArtworkView(url: game.artworkURL, title: game.title).frame(height: 205)
                Text(game.title).font(.headline).lineLimit(2).foregroundStyle(.primary)
                HStack { Text(SystemCatalog.system(id: game.systemID)?.shortName ?? game.systemID).font(.caption).foregroundStyle(.secondary); Spacer(); if game.favorite { Image(systemName: "star.fill").foregroundStyle(.yellow) } }
            }.padding(10).background(.thinMaterial).clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
        }.buttonStyle(.plain).contextMenu { Button("Favorite", systemImage: "star") { Task { await model.toggleFavorite(game) } } }
    }
}

struct ArtworkView: View {
    let url: URL?; let title: String
    var body: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 14).fill(LinearGradient(colors: [.indigo.opacity(0.7), .cyan.opacity(0.5)], startPoint: .topLeading, endPoint: .bottomTrailing))
            if let url, let image = UIImage(contentsOfFile: url.path) { Image(uiImage: image).resizable().scaledToFill() }
            else { VStack { Image(systemName: "gamecontroller.fill").font(.largeTitle); Text(title.prefix(2).uppercased()).font(.title.bold()) }.foregroundStyle(.white) }
        }.clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
    }
}
#endif
