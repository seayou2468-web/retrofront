import Foundation

public enum SystemCatalog {
    public static let systems: [EmulationSystem] = [
        .init(id: "nes", name: "Nintendo Entertainment System", shortName: "NES", manufacturer: "Nintendo", family: .nintendo, fileExtensions: ["nes", "fds", "unf", "unif"]),
        .init(id: "snes", name: "Super Nintendo Entertainment System", shortName: "SNES", manufacturer: "Nintendo", family: .nintendo, fileExtensions: ["sfc", "smc", "fig", "bs", "st"]),
        .init(id: "gb", name: "Game Boy / Game Boy Color", shortName: "GB/GBC", manufacturer: "Nintendo", family: .nintendo, fileExtensions: ["gb", "gbc", "sgb"]),
        .init(id: "gba", name: "Game Boy Advance", shortName: "GBA", manufacturer: "Nintendo", family: .nintendo, fileExtensions: ["gba"]),
        .init(id: "n64", name: "Nintendo 64", shortName: "N64", manufacturer: "Nintendo", family: .nintendo, fileExtensions: ["n64", "z64", "v64"]),
        .init(id: "nds", name: "Nintendo DS", shortName: "DS", manufacturer: "Nintendo", family: .nintendo, fileExtensions: ["nds", "dsi"]),
        .init(id: "sms", name: "Sega Master System / Game Gear", shortName: "SMS/GG", manufacturer: "Sega", family: .sega, fileExtensions: ["sms", "gg", "sg"]),
        .init(id: "genesis", name: "Sega Genesis / Mega Drive", shortName: "Genesis", manufacturer: "Sega", family: .sega, fileExtensions: ["md", "gen", "smd", "bin", "32x"]),
        .init(id: "saturn", name: "Sega Saturn", shortName: "Saturn", manufacturer: "Sega", family: .sega, fileExtensions: ["cue", "chd", "iso", "ccd", "mds"], biosFiles: [.init(fileName: "saturn_bios.bin", description: "Saturn boot ROM")]),
        .init(id: "psx", name: "Sony PlayStation", shortName: "PS1", manufacturer: "Sony", family: .sony, fileExtensions: ["cue", "chd", "pbp", "iso", "img", "m3u"], biosFiles: [.init(fileName: "scph5501.bin", md5: "490f666e1afb15b7362b406ed1cea246", description: "US PlayStation BIOS")]),
        .init(id: "psp", name: "PlayStation Portable", shortName: "PSP", manufacturer: "Sony", family: .sony, fileExtensions: ["iso", "cso", "pbp"]),
        .init(id: "pce", name: "PC Engine / TurboGrafx-16", shortName: "PCE", manufacturer: "NEC", family: .nec, fileExtensions: ["pce", "cue", "chd", "sgx"]),
        .init(id: "arcade", name: "Arcade", shortName: "Arcade", manufacturer: "Multiple", family: .arcade, fileExtensions: ["zip", "7z"]),
        .init(id: "neogeo", name: "Neo Geo", shortName: "Neo Geo", manufacturer: "SNK", family: .snk, fileExtensions: ["zip", "7z"]),
        .init(id: "atari2600", name: "Atari 2600", shortName: "A2600", manufacturer: "Atari", family: .atari, fileExtensions: ["a26", "bin"]),
        .init(id: "dos", name: "DOS / PC", shortName: "DOS", manufacturer: "IBM compatible", family: .computer, fileExtensions: ["dosz", "zip", "exe", "bat", "conf"]),
        .init(id: "scummvm", name: "ScummVM", shortName: "ScummVM", manufacturer: "ScummVM", family: .engine, fileExtensions: ["scummvm"])
    ]

    public static func system(forExtension ext: String) -> EmulationSystem? {
        let normalized = ext.lowercased().trimmingCharacters(in: CharacterSet(charactersIn: "."))
        return systems.first { $0.fileExtensions.contains(normalized) }
    }

    public static func system(id: String) -> EmulationSystem? { systems.first { $0.id == id } }
}
