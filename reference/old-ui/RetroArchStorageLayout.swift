import Foundation

/// Mirrors RetroArch's iOS/tvOS directory layout from `reference/RetroArch/frontend/drivers/platform_darwin.m`.
struct RetroArchStorageLayout {
  let root: URL
  let bundlePath: URL?

  static var current: RetroArchStorageLayout {
    let documents = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    return RetroArchStorageLayout(root: documents.appendingPathComponent("RetroArch", isDirectory: true), bundlePath: Bundle.main.bundleURL)
  }

  var configDirectory: URL { root.appendingPathComponent("config", isDirectory: true) }
  var configFile: URL { configDirectory.appendingPathComponent("retroarch.cfg") }
  var assetsDirectory: URL { root.appendingPathComponent("assets", isDirectory: true) }
  var infoDirectory: URL { root.appendingPathComponent("info", isDirectory: true) }
  var overlaysDirectory: URL { root.appendingPathComponent("overlays", isDirectory: true) }
  var downloadsDirectory: URL { root.appendingPathComponent("downloads", isDirectory: true) }
  var contentDirectory: URL { root.appendingPathComponent("Roms", isDirectory: true) }
  var savesDirectory: URL { root.appendingPathComponent("saves", isDirectory: true) }
  var statesDirectory: URL { root.appendingPathComponent("states", isDirectory: true) }
  var systemDirectory: URL { root.appendingPathComponent("system", isDirectory: true) }
  var playlistsDirectory: URL { root.appendingPathComponent("playlists", isDirectory: true) }
  var cacheDirectory: URL { URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true) }
  var bundledCoreDirectory: URL? {
    if let frameworks = Bundle.main.privateFrameworksURL { return frameworks }
    return bundlePath?.appendingPathComponent("Frameworks", isDirectory: true)
  }

  var overlayConfig: URL { overlaysDirectory.appendingPathComponent("gamepads/flat/retropad.cfg") }

  func createWritableDirectories() throws {
    for directory in [root, configDirectory, assetsDirectory, infoDirectory, overlaysDirectory, downloadsDirectory, contentDirectory, savesDirectory, statesDirectory, systemDirectory, playlistsDirectory] {
      try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }
  }
}
