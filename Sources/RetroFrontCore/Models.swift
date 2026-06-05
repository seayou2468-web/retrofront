import Foundation

public enum ConsoleFamily: String, Codable, CaseIterable, Identifiable, Sendable {
    case arcade, nintendo, sega, sony, nec, atari, snk, microsoft, computer, engine, unknown
    public var id: String { rawValue }
    public var displayName: String { rawValue.capitalized }
}

public struct EmulationSystem: Identifiable, Codable, Hashable, Sendable {
    public var id: String
    public var name: String
    public var shortName: String
    public var manufacturer: String
    public var family: ConsoleFamily
    public var fileExtensions: [String]
    public var biosFiles: [BIOSRequirement]

    public init(id: String, name: String, shortName: String, manufacturer: String, family: ConsoleFamily, fileExtensions: [String], biosFiles: [BIOSRequirement] = []) {
        self.id = id; self.name = name; self.shortName = shortName; self.manufacturer = manufacturer; self.family = family
        self.fileExtensions = fileExtensions.map { $0.lowercased().trimmingCharacters(in: CharacterSet(charactersIn: ".")) }
        self.biosFiles = biosFiles
    }
}

public struct BIOSRequirement: Identifiable, Codable, Hashable, Sendable {
    public var id: String { fileName }
    public var fileName: String
    public var md5: String?
    public var description: String
    public var required: Bool
    public init(fileName: String, md5: String? = nil, description: String, required: Bool = true) {
        self.fileName = fileName; self.md5 = md5; self.description = description; self.required = required
    }
}

public struct LibretroCore: Identifiable, Codable, Hashable, Sendable {
    public var id: UUID
    public var displayName: String
    public var bundleIdentifier: String
    public var systemIDs: [String]
    public var path: URL
    public var version: String?
    public var supportedExtensions: [String]
    public var requiresFullPath: Bool
    public var options: [CoreOption]

    public init(id: UUID = UUID(), displayName: String, bundleIdentifier: String, systemIDs: [String], path: URL, version: String? = nil, supportedExtensions: [String], requiresFullPath: Bool = false, options: [CoreOption] = []) {
        self.id = id; self.displayName = displayName; self.bundleIdentifier = bundleIdentifier; self.systemIDs = systemIDs; self.path = path; self.version = version
        self.supportedExtensions = supportedExtensions.map { $0.lowercased().trimmingCharacters(in: CharacterSet(charactersIn: ".")) }
        self.requiresFullPath = requiresFullPath; self.options = options
    }
}

public struct CoreOption: Identifiable, Codable, Hashable, Sendable {
    public var id: String { key }
    public var key: String
    public var title: String
    public var detail: String
    public var values: [String]
    public var defaultValue: String
}

public struct Game: Identifiable, Codable, Hashable, Sendable {
    public var id: UUID
    public var title: String
    public var systemID: String
    public var fileURL: URL
    public var artworkURL: URL?
    public var developer: String?
    public var publisher: String?
    public var genre: String?
    public var releaseYear: Int?
    public var region: String?
    public var favorite: Bool
    public var lastPlayedAt: Date?
    public var playTime: TimeInterval
    public var crc32: String?

    public init(id: UUID = UUID(), title: String, systemID: String, fileURL: URL, artworkURL: URL? = nil, developer: String? = nil, publisher: String? = nil, genre: String? = nil, releaseYear: Int? = nil, region: String? = nil, favorite: Bool = false, lastPlayedAt: Date? = nil, playTime: TimeInterval = 0, crc32: String? = nil) {
        self.id = id; self.title = title; self.systemID = systemID; self.fileURL = fileURL; self.artworkURL = artworkURL; self.developer = developer; self.publisher = publisher; self.genre = genre; self.releaseYear = releaseYear; self.region = region; self.favorite = favorite; self.lastPlayedAt = lastPlayedAt; self.playTime = playTime; self.crc32 = crc32
    }
}

public struct SaveState: Identifiable, Codable, Hashable, Sendable {
    public var id: UUID
    public var gameID: UUID
    public var coreID: UUID
    public var slot: Int
    public var createdAt: Date
    public var stateURL: URL
    public var thumbnailURL: URL?
    public var note: String
}

public struct CheatCode: Identifiable, Codable, Hashable, Sendable {
    public var id = UUID()
    public var gameID: UUID
    public var name: String
    public var code: String
    public var enabled: Bool
}

public struct NetplayRoom: Identifiable, Codable, Hashable, Sendable {
    public var id = UUID()
    public var host: String
    public var port: Int
    public var gameTitle: String
    public var coreName: String
    public var players: Int
    public var maxPlayers: Int
}

public struct FrontendSettings: Codable, Hashable, Sendable {
    public var shaderPreset: String
    public var integerScaling: Bool
    public var aspectRatio: AspectRatioMode
    public var rewindEnabled: Bool
    public var runaheadFrames: Int
    public var fastForwardRate: Double
    public var hapticsEnabled: Bool
    public var retroAchievementsUser: String
    public var iCloudSyncEnabled: Bool
    public var autoSaveOnBackground: Bool

    public init(shaderPreset: String = "LCD + subtle CRT", integerScaling: Bool = false, aspectRatio: AspectRatioMode = .coreProvided, rewindEnabled: Bool = true, runaheadFrames: Int = 0, fastForwardRate: Double = 2, hapticsEnabled: Bool = true, retroAchievementsUser: String = "", iCloudSyncEnabled: Bool = true, autoSaveOnBackground: Bool = true) {
        self.shaderPreset = shaderPreset; self.integerScaling = integerScaling; self.aspectRatio = aspectRatio; self.rewindEnabled = rewindEnabled
        self.runaheadFrames = runaheadFrames; self.fastForwardRate = fastForwardRate; self.hapticsEnabled = hapticsEnabled; self.retroAchievementsUser = retroAchievementsUser
        self.iCloudSyncEnabled = iCloudSyncEnabled; self.autoSaveOnBackground = autoSaveOnBackground
    }
}

public enum AspectRatioMode: String, Codable, CaseIterable, Identifiable, Sendable {
    case coreProvided, fourThree, squarePixels, stretch, fullscreen
    public var id: String { rawValue }
    public var title: String {
        switch self {
        case .coreProvided: "Core provided"
        case .fourThree: "4:3"
        case .squarePixels: "Square pixels"
        case .stretch: "Stretch"
        case .fullscreen: "Fullscreen"
        }
    }
}
