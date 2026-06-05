import Foundation

public enum CRC32 {
    private static let table: [UInt32] = (0..<256).map { i in
        var c = UInt32(i)
        for _ in 0..<8 { c = (c & 1) == 1 ? (0xedb88320 ^ (c >> 1)) : (c >> 1) }
        return c
    }

    public static func hexDigest(url: URL) throws -> String {
        let data = try Data(contentsOf: url)
        var crc: UInt32 = 0xffffffff
        for byte in data { crc = table[Int((crc ^ UInt32(byte)) & 0xff)] ^ (crc >> 8) }
        return String(format: "%08X", crc ^ 0xffffffff)
    }
}

public struct PlaylistParser: Sendable {
    public init() {}
    public func parseLPL(data: Data) throws -> [Game] {
        struct LPL: Decodable { let items: [Item]?; struct Item: Decodable { let path: String?; let label: String?; let core_name: String?; let crc32: String?; let db_name: String? } }
        let lpl = try JSONDecoder().decode(LPL.self, from: data)
        return (lpl.items ?? []).compactMap { item in
            guard let path = item.path, let label = item.label else { return nil }
            let url = URL(fileURLWithPath: path)
            let systemID = SystemCatalog.system(forExtension: url.pathExtension)?.id ?? item.db_name?.lowercased().components(separatedBy: CharacterSet.alphanumerics.inverted).first ?? "unknown"
            return Game(title: label, systemID: systemID, fileURL: url, crc32: item.crc32)
        }
    }
}

public struct CoreInfoParser: Sendable {
    public init() {}
    public func parseInfo(_ text: String, coreURL: URL) -> LibretroCore {
        var fields: [String: String] = [:]
        for line in text.split(whereSeparator: \.isNewline) {
            let parts = line.split(separator: "=", maxSplits: 1).map { $0.trimmingCharacters(in: .whitespacesAndNewlines.union(CharacterSet(charactersIn: "\""))) }
            if parts.count == 2 { fields[parts[0]] = parts[1] }
        }
        let extensions = (fields["supported_extensions"] ?? "").split(separator: "|").map(String.init)
        let systems = SystemCatalog.systems.filter { system in !Set(system.fileExtensions).intersection(extensions.map { $0.lowercased() }).isEmpty }
        return LibretroCore(displayName: fields["display_name"] ?? coreURL.deletingPathExtension().lastPathComponent, bundleIdentifier: fields["corename"] ?? coreURL.lastPathComponent, systemIDs: systems.map(\.id), path: coreURL, version: fields["display_version"], supportedExtensions: extensions, requiresFullPath: (fields["needs_fullpath"] ?? "false") == "true")
    }
}

public actor ArtworkService {
    private let artworkDirectory: URL
    public init(artworkDirectory: URL) { self.artworkDirectory = artworkDirectory }

    public func localArtwork(for game: Game) -> URL? {
        let candidates = ["png", "jpg", "jpeg", "webp"].map { artworkDirectory.appendingPathComponent(game.title).appendingPathExtension($0) }
        return candidates.first { FileManager.default.fileExists(atPath: $0.path) }
    }

    public func attachLocalArtwork(to games: [Game]) -> [Game] {
        games.map { game in var copy = game; copy.artworkURL = game.artworkURL ?? localArtwork(for: game); return copy }
    }
}

public actor BIOSManager {
    private let systemDirectory: URL
    public init(systemDirectory: URL) { self.systemDirectory = systemDirectory }
    public func audit(for systems: [EmulationSystem]) -> [BIOSRequirement] {
        systems.flatMap(\.biosFiles).filter { req in !FileManager.default.fileExists(atPath: systemDirectory.appendingPathComponent(req.fileName).path) }
    }
}
