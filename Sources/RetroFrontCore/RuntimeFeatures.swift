import Foundation

public struct SoftPatchResult: Sendable, Hashable {
    public var patchedURL: URL
    public var appliedPatchURL: URL?
    public var messages: [String]
}

public struct SoftPatchManager: Sendable {
    public init() {}

    public func preparedContentURL(for game: Game, workDirectory: URL) throws -> SoftPatchResult {
        let fm = FileManager.default
        try fm.createDirectory(at: workDirectory, withIntermediateDirectories: true)
        let base = game.fileURL.deletingPathExtension()
        let candidates = ["ips", "IPS"].map { base.appendingPathExtension($0) }
        guard let patch = candidates.first(where: { fm.fileExists(atPath: $0.path) }) else {
            return SoftPatchResult(patchedURL: game.fileURL, appliedPatchURL: nil, messages: ["No IPS soft patch found next to ROM."])
        }
        var rom = try Data(contentsOf: game.fileURL)
        try applyIPS(patchURL: patch, to: &rom)
        let destination = workDirectory.appendingPathComponent(game.fileURL.deletingPathExtension().lastPathComponent + ".patched." + game.fileURL.pathExtension)
        try rom.write(to: destination, options: .atomic)
        return SoftPatchResult(patchedURL: destination, appliedPatchURL: patch, messages: ["Applied IPS soft patch: \(patch.lastPathComponent)"])
    }

    public func applyIPS(patchURL: URL, to rom: inout Data) throws {
        let patch = try Data(contentsOf: patchURL)
        guard patch.count >= 8, String(data: patch.prefix(5), encoding: .ascii) == "PATCH" else { throw PatchError.invalidHeader }
        var offset = 5
        while offset + 3 <= patch.count {
            if patch[offset] == 0x45, patch[offset + 1] == 0x4F, patch[offset + 2] == 0x46 { return }
            let target = (Int(patch[offset]) << 16) | (Int(patch[offset + 1]) << 8) | Int(patch[offset + 2])
            offset += 3
            guard offset + 2 <= patch.count else { throw PatchError.truncated }
            let size = (Int(patch[offset]) << 8) | Int(patch[offset + 1])
            offset += 2
            if size == 0 {
                guard offset + 3 <= patch.count else { throw PatchError.truncated }
                let rleSize = (Int(patch[offset]) << 8) | Int(patch[offset + 1])
                let value = patch[offset + 2]
                offset += 3
                ensureSize(target + rleSize, data: &rom)
                rom.replaceSubrange(target..<(target + rleSize), with: repeatElement(value, count: rleSize))
            } else {
                guard offset + size <= patch.count else { throw PatchError.truncated }
                ensureSize(target + size, data: &rom)
                rom.replaceSubrange(target..<(target + size), with: patch[offset..<(offset + size)])
                offset += size
            }
        }
        throw PatchError.missingEOF
    }

    private func ensureSize(_ size: Int, data: inout Data) {
        if data.count < size { data.append(contentsOf: repeatElement(0, count: size - data.count)) }
    }

    public enum PatchError: Error, LocalizedError, Sendable {
        case invalidHeader, truncated, missingEOF
        public var errorDescription: String? {
            switch self {
            case .invalidHeader: "IPS patch has an invalid PATCH header."
            case .truncated: "IPS patch is truncated."
            case .missingEOF: "IPS patch is missing EOF marker."
            }
        }
    }
}

public struct NetplayConfiguration: Codable, Hashable, Sendable {
    public var host: String
    public var port: Int
    public var nickname: String

    public init(host: String = "", port: Int = 55435, nickname: String = "Player") {
        self.host = host
        self.port = port
        self.nickname = nickname
    }
}

public enum FrontendPresetCatalog {
    public static let shaderPresets = ["LCD + subtle CRT", "Sharp pixels", "CRT Royale style", "Scanlines", "No shader"]
    public static let overlayPresets = ["Auto", "Game Boy", "GBA", "SNES", "Genesis", "Arcade", "Minimal"]
}
