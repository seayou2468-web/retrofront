import Foundation

public actor LibraryStore {
    public private(set) var games: [Game]
    public private(set) var cores: [LibretroCore]
    public private(set) var saveStates: [SaveState]
    public private(set) var cheats: [CheatCode]
    public private(set) var settings: FrontendSettings

    private let root: URL
    private let encoder: JSONEncoder
    private let decoder: JSONDecoder

    public init(root: URL = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first ?? URL(fileURLWithPath: ".retrofront")) async {
        self.root = root
        self.encoder = JSONEncoder(); self.decoder = JSONDecoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        encoder.dateEncodingStrategy = .iso8601; decoder.dateDecodingStrategy = .iso8601
        self.games = []; self.cores = []; self.saveStates = []; self.cheats = []; self.settings = .init()
        await load()
    }

    public var directories: FrontendDirectories { FrontendDirectories(root: root) }

    public func load() async {
        try? FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        for url in directories.all { try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true) }
        games = (try? read([Game].self, from: root.appendingPathComponent("games.json"))) ?? []
        cores = (try? read([LibretroCore].self, from: root.appendingPathComponent("cores.json"))) ?? []
        saveStates = (try? read([SaveState].self, from: root.appendingPathComponent("states.json"))) ?? []
        cheats = (try? read([CheatCode].self, from: root.appendingPathComponent("cheats.json"))) ?? []
        settings = (try? read(FrontendSettings.self, from: root.appendingPathComponent("settings.json"))) ?? .init()
    }

    public func save() async throws {
        try write(games, to: root.appendingPathComponent("games.json"))
        try write(cores, to: root.appendingPathComponent("cores.json"))
        try write(saveStates, to: root.appendingPathComponent("states.json"))
        try write(cheats, to: root.appendingPathComponent("cheats.json"))
        try write(settings, to: root.appendingPathComponent("settings.json"))
    }

    public func upsert(game: Game) async throws {
        if let index = games.firstIndex(where: { $0.id == game.id || $0.fileURL == game.fileURL }) { games[index] = game } else { games.append(game) }
        try await save()
    }

    public func upsert(core: LibretroCore) async throws {
        if let index = cores.firstIndex(where: { $0.id == core.id || $0.path == core.path }) { cores[index] = core } else { cores.append(core) }
        try await save()
    }

    public func update(settings newValue: FrontendSettings) async throws { settings = newValue; try await save() }

    public func add(saveState: SaveState) async throws { saveStates.removeAll { $0.gameID == saveState.gameID && $0.slot == saveState.slot }; saveStates.append(saveState); try await save() }
    public func add(cheat: CheatCode) async throws { cheats.append(cheat); try await save() }

    private func read<T: Decodable>(_ type: T.Type, from url: URL) throws -> T { try decoder.decode(T.self, from: Data(contentsOf: url)) }
    private func write<T: Encodable>(_ value: T, to url: URL) throws { try encoder.encode(value).write(to: url, options: [.atomic]) }
}

public struct FrontendDirectories: Sendable {
    public let root: URL
    public var roms: URL { root.appendingPathComponent("ROMs") }
    public var cores: URL { root.appendingPathComponent("Cores") }
    public var saves: URL { root.appendingPathComponent("Saves") }
    public var states: URL { root.appendingPathComponent("States") }
    public var system: URL { root.appendingPathComponent("System") }
    public var artwork: URL { root.appendingPathComponent("Artwork") }
    public var playlists: URL { root.appendingPathComponent("Playlists") }
    public var shaders: URL { root.appendingPathComponent("Shaders") }
    public var overlays: URL { root.appendingPathComponent("Overlays") }
    public var imports: URL { root.appendingPathComponent("Import Inbox") }
    public var all: [URL] { [roms, cores, saves, states, system, artwork, playlists, shaders, overlays, imports] }
}
