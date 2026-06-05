import Foundation

public struct ScanReport: Sendable, Codable, Hashable {
    public var discoveredGames: [Game]
    public var ignoredFiles: [URL]
    public var missingBIOS: [BIOSRequirement]
}

public struct LibraryScanner: Sendable {
    public init() {}

    public func scanROMs(at root: URL, knownGames: [Game] = []) async throws -> ScanReport {
        let fm = FileManager.default
        guard let enumerator = fm.enumerator(at: root, includingPropertiesForKeys: [.isRegularFileKey], options: [.skipsHiddenFiles]) else {
            return ScanReport(discoveredGames: [], ignoredFiles: [], missingBIOS: [])
        }
        var games = knownGames
        var ignored: [URL] = []
        for case let file as URL in enumerator {
            let ext = file.pathExtension.lowercased()
            guard let system = SystemCatalog.system(forExtension: ext) else { ignored.append(file); continue }
            if games.contains(where: { $0.fileURL == file }) { continue }
            var title = file.deletingPathExtension().lastPathComponent
            title = title.replacingOccurrences(of: "_", with: " ").replacingOccurrences(of: ".", with: " ")
            let crc = try? CRC32.hexDigest(url: file)
            games.append(Game(title: title, systemID: system.id, fileURL: file, crc32: crc))
        }
        return ScanReport(discoveredGames: games.sorted { $0.title.localizedCaseInsensitiveCompare($1.title) == .orderedAscending }, ignoredFiles: ignored, missingBIOS: [])
    }

    public func scanCores(at root: URL) async throws -> [LibretroCore] {
        let fm = FileManager.default
        guard let enumerator = fm.enumerator(at: root, includingPropertiesForKeys: [.isRegularFileKey], options: [.skipsHiddenFiles]) else { return [] }
        var cores: [LibretroCore] = []
        let coreExtensions = ["dylib", "framework", "so"]
        for case let file as URL in enumerator where coreExtensions.contains(file.pathExtension.lowercased()) {
            let name = file.deletingPathExtension().lastPathComponent
            let inferred = inferSystems(fromCoreName: name)
            cores.append(LibretroCore(displayName: prettifyCoreName(name), bundleIdentifier: name, systemIDs: inferred.map(\.id), path: file, supportedExtensions: Array(Set(inferred.flatMap(\.fileExtensions))).sorted()))
        }
        return cores.sorted { $0.displayName < $1.displayName }
    }

    public func importFiles(_ files: [URL], into directories: FrontendDirectories) async throws -> ScanReport {
        let fm = FileManager.default
        var imported: [URL] = []
        for file in files {
            let ext = file.pathExtension.lowercased()
            let destinationRoot: URL
            if ["dylib", "framework", "so"].contains(ext) { destinationRoot = directories.cores }
            else if ["png", "jpg", "jpeg", "webp"].contains(ext) { destinationRoot = directories.artwork }
            else { destinationRoot = directories.roms }
            let destination = destinationRoot.appendingPathComponent(file.lastPathComponent)
            if fm.fileExists(atPath: destination.path) { try fm.removeItem(at: destination) }
            try fm.copyItem(at: file, to: destination)
            imported.append(destination)
        }
        return try await scanROMs(at: directories.roms)
    }

    private func inferSystems(fromCoreName name: String) -> [EmulationSystem] {
        let lower = name.lowercased()
        let matched = SystemCatalog.systems.filter { system in
            lower.contains(system.id) || lower.contains(system.shortName.lowercased()) || lower.contains(system.name.lowercased().split(separator: " ").first.map(String.init) ?? "")
        }
        if lower.contains("snes9x") { return SystemCatalog.systems.filter { $0.id == "snes" } }
        if lower.contains("mgba") || lower.contains("vba") { return SystemCatalog.systems.filter { ["gba", "gb"].contains($0.id) } }
        if lower.contains("genesis") || lower.contains("picodrive") { return SystemCatalog.systems.filter { ["genesis", "sms"].contains($0.id) } }
        if lower.contains("pcsx") || lower.contains("beetle_psx") { return SystemCatalog.systems.filter { $0.id == "psx" } }
        return matched.isEmpty ? SystemCatalog.systems : matched
    }

    private func prettifyCoreName(_ name: String) -> String { name.replacingOccurrences(of: "_libretro", with: "").replacingOccurrences(of: "_", with: " ").capitalized }
}
