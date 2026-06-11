import Foundation

public struct RetroArchLinuxLayout: Sendable {
  public let root: URL
  public let bundledCoreDirectory: URL?
  public let bundledAssetDirectory: URL?

  public static var current: RetroArchLinuxLayout {
    let environment = ProcessInfo.processInfo.environment
    let home = environment["HOME"].map { URL(fileURLWithPath: $0, isDirectory: true) }
    let configRoot: URL
    if let xdg = environment["XDG_CONFIG_HOME"], !xdg.isEmpty {
      configRoot = URL(fileURLWithPath: xdg, isDirectory: true)
    } else if let home {
      configRoot = home.appendingPathComponent(".config", isDirectory: true)
    } else {
      configRoot = URL(fileURLWithPath: "retroarch", isDirectory: true)
    }

    let bundledCore = environment["RETROFRONT_BUNDLED_CORE_DIR"].flatMap { $0.isEmpty ? nil : URL(fileURLWithPath: $0, isDirectory: true) }
    let bundledAssets = environment["RETROFRONT_BUNDLED_ASSET_DIR"].flatMap { $0.isEmpty ? nil : URL(fileURLWithPath: $0, isDirectory: true) }
    return RetroArchLinuxLayout(
      root: configRoot.appendingPathComponent("retroarch", isDirectory: true),
      bundledCoreDirectory: bundledCore,
      bundledAssetDirectory: bundledAssets
    )
  }

  public var configDirectory: URL { root.appendingPathComponent("config", isDirectory: true) }
  public var configFile: URL { configDirectory.appendingPathComponent("retroarch.cfg") }
  public var userCoreDirectory: URL { root.appendingPathComponent("cores", isDirectory: true) }
  public var infoDirectory: URL { root.appendingPathComponent("cores", isDirectory: true) }
  public var assetsDirectory: URL { root.appendingPathComponent("assets", isDirectory: true) }
  public var overlaysDirectory: URL { root.appendingPathComponent("overlays", isDirectory: true) }
  public var downloadsDirectory: URL { root.appendingPathComponent("downloads", isDirectory: true) }
  public var savesDirectory: URL { root.appendingPathComponent("saves", isDirectory: true) }
  public var statesDirectory: URL { root.appendingPathComponent("states", isDirectory: true) }
  public var systemDirectory: URL { root.appendingPathComponent("system", isDirectory: true) }
  public var screenshotsDirectory: URL { root.appendingPathComponent("screenshots", isDirectory: true) }
  public var playlistsDirectory: URL { root.appendingPathComponent("playlists", isDirectory: true) }
  public var cacheDirectory: URL { root.appendingPathComponent("temp", isDirectory: true) }
  public var overlayConfig: URL { overlaysDirectory.appendingPathComponent("gamepads/flat/retropad.cfg") }

  public func createWritableDirectories() throws {
    for directory in [root, configDirectory, userCoreDirectory, assetsDirectory, overlaysDirectory, downloadsDirectory, savesDirectory, statesDirectory, systemDirectory, screenshotsDirectory, playlistsDirectory, cacheDirectory] {
      try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }
  }

  public func apply(to frontend: Retrofront) throws {
    try createWritableDirectories()
    try frontend.setBaseDirectory(root.path)
    try frontend.setSetting(key: "libretro_directory", value: userCoreDirectory.path)
    try frontend.setSetting(key: "libretro_info_path", value: infoDirectory.path)
    try frontend.setSetting(key: "assets_directory", value: assetsDirectory.path)
    try frontend.setSetting(key: "menu_assets_directory", value: assetsDirectory.path)
    try frontend.setSetting(key: "overlay_directory", value: overlaysDirectory.path)
    try frontend.setSetting(key: "input_overlay", value: overlayConfig.path)
    try frontend.setSetting(key: "content_directory", value: root.path)
    try frontend.setSetting(key: "menu_content_directory", value: root.path)
    try frontend.setSetting(key: "core_assets_directory", value: downloadsDirectory.path)
    try frontend.setSetting(key: "savefile_directory", value: savesDirectory.path)
    try frontend.setSetting(key: "savestate_directory", value: statesDirectory.path)
    try frontend.setSetting(key: "system_directory", value: systemDirectory.path)
    try frontend.setSetting(key: "screenshot_directory", value: screenshotsDirectory.path)
    try frontend.setSetting(key: "playlist_directory", value: playlistsDirectory.path)
    try frontend.setSetting(key: "cache_directory", value: cacheDirectory.path)
    frontend.setInfoDir(infoDirectory.path)
  }
}
