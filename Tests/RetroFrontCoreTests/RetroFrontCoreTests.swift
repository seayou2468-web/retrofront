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
