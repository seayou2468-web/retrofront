import XCTest
@testable import RetroFrontCore

final class RetroFrontCoreTests: XCTestCase {
    func testSystemCatalogFindsExtensions() {
        XCTAssertEqual(SystemCatalog.system(forExtension: "gba")?.id, "gba")
        XCTAssertEqual(SystemCatalog.system(forExtension: ".nes")?.id, "nes")
    }

    func testCRC32KnownValue() throws {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try Data("123456789".utf8).write(to: url)
        XCTAssertEqual(try CRC32.hexDigest(url: url), "CBF43926")
    }

    func testCoreInfoParser() {
        let info = """
        display_name = "Snes9x"
        supported_extensions = "smc|sfc"
        needs_fullpath = "false"
        """
        let core = CoreInfoParser().parseInfo(info, coreURL: URL(fileURLWithPath: "/cores/snes9x_libretro.dylib"))
        XCTAssertEqual(core.displayName, "Snes9x")
        XCTAssertTrue(core.systemIDs.contains("snes"))
        XCTAssertEqual(core.supportedExtensions, ["smc", "sfc"])
    }

    func testPlaylistParser() throws {
        let json = #"{"items":[{"path":"/roms/Mario.nes","label":"Mario","crc32":"DEADBEEF","db_name":"Nintendo - Nintendo Entertainment System.lpl"}]}"#
        let games = try PlaylistParser().parseLPL(data: Data(json.utf8))
        XCTAssertEqual(games.first?.title, "Mario")
        XCTAssertEqual(games.first?.systemID, "nes")
    }
}

extension RetroFrontCoreTests {
    func testDynamicCoreManagerPlansUnsignedImport() async throws {
        let temp = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: temp, withIntermediateDirectories: true)
        let core = temp.appendingPathComponent("example_libretro_ios.dylib")
        try Data("not a signed mach-o".utf8).write(to: core)
        let plan = DynamicCoreManager().installPlan(for: core, directories: FrontendDirectories(root: temp))
        XCTAssertEqual(plan.status, .importedUnsigned)
        XCTAssertEqual(plan.destination.lastPathComponent, "example_libretro_ios.dylib")
    }

    func testFeatureMatrixDocumentsDynamicCoresAndMenuEngines() {
        let ids = Set(FrontendFeatureMatrix.capabilities.map(\.id))
        XCTAssertTrue(ids.contains("dynamic-cores"))
        XCTAssertTrue(ids.contains("menu-engines"))
    }
}

extension RetroFrontCoreTests {
    func testIPSSoftPatchAppliesNormalAndRLERecords() throws {
        let temp = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: temp, withIntermediateDirectories: true)
        let patch = temp.appendingPathComponent("game.ips")
        var patchData = Data("PATCH".utf8)
        patchData.append(contentsOf: [0x00, 0x00, 0x01, 0x00, 0x02, 0xAA, 0xBB])
        patchData.append(contentsOf: [0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x03, 0xCC])
        patchData.append(contentsOf: [0x45, 0x4F, 0x46])
        try patchData.write(to: patch)
        var rom = Data([0, 0, 0, 0, 0, 0, 0])

        try SoftPatchManager().applyIPS(patchURL: patch, to: &rom)

        XCTAssertEqual(Array(rom), [0, 0xAA, 0xBB, 0, 0xCC, 0xCC, 0xCC])
    }
}

extension RetroFrontCoreTests {
    func testVideoFrameConvertsLibretroPixelFormats() {
        let rgb565Red = VideoFrame(width: 1, height: 1, pitch: 2, pixelFormat: LibretroPixelFormat.rgb565.rawValue, bytes: Data([0x00, 0xF8]))
        XCTAssertEqual(Array(rgb565Red.rgba8888), [255, 0, 0, 255])

        let xrgbBlue = VideoFrame(width: 1, height: 1, pitch: 4, pixelFormat: LibretroPixelFormat.xrgb8888.rawValue, bytes: Data([255, 0, 0, 0]))
        XCTAssertEqual(Array(xrgbBlue.rgba8888), [0, 0, 255, 255])

        let rgb1555Green = VideoFrame(width: 1, height: 1, pitch: 2, pixelFormat: LibretroPixelFormat.rgb1555.rawValue, bytes: Data([0xE0, 0x03]))
        XCTAssertEqual(Array(rgb1555Green.rgba8888), [0, 255, 0, 255])
    }
}
