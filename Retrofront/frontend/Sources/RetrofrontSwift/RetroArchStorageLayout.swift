import Foundation

/// RetroArch-compatible storage layout shared by the iOS and Linux Swift apps.
///
/// The paths mirror `reference/RetroArch/frontend/drivers/platform_darwin.m` on
/// iOS and `reference/RetroArch/frontend/drivers/platform_unix.c` on Linux so
/// the Swift shells and Rust frontend core agree on the same filesystem shape.
public struct RetroArchStorageLayout: Sendable {
  public let root: URL
  public let applicationDataRoot: URL
  public let bundlePath: URL?
  public let bundledCoreDirectory: URL?

  public init(root: URL, applicationDataRoot: URL? = nil, bundlePath: URL? = nil, bundledCoreDirectory: URL? = nil) {
    self.root = root
    self.applicationDataRoot = applicationDataRoot ?? root
    self.bundlePath = bundlePath
    self.bundledCoreDirectory = bundledCoreDirectory
  }

  public static var current: RetroArchStorageLayout {
    #if os(iOS) || os(tvOS)
    let documents = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    let root = documents.appendingPathComponent("RetroArch", isDirectory: true)
    let bundledCoreDirectory = Bundle.main.privateFrameworksURL
      ?? Bundle.main.bundleURL.appendingPathComponent("Frameworks", isDirectory: true)
    return RetroArchStorageLayout(
      root: root,
      applicationDataRoot: root,
      bundlePath: Bundle.main.bundleURL,
      bundledCoreDirectory: bundledCoreDirectory)
    #elseif os(Linux)
    let environment = ProcessInfo.processInfo.environment
    let root: URL
    if let xdgConfigHome = environment["XDG_CONFIG_HOME"], !xdgConfigHome.isEmpty {
      root = URL(fileURLWithPath: xdgConfigHome, isDirectory: true).appendingPathComponent("retroarch", isDirectory: true)
    } else if let home = environment["HOME"], !home.isEmpty {
      root = URL(fileURLWithPath: home, isDirectory: true)
        .appendingPathComponent(".config", isDirectory: true)
        .appendingPathComponent("retroarch", isDirectory: true)
    } else {
      root = URL(fileURLWithPath: "retroarch", isDirectory: true)
    }
    return RetroArchStorageLayout(root: root, applicationDataRoot: root)
    #else
    let root = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
      .appendingPathComponent("retroarch", isDirectory: true)
    return RetroArchStorageLayout(root: root, applicationDataRoot: root)
    #endif
  }

  public var configDirectory: URL { applicationDataRoot.appendingPathComponent("config", isDirectory: true) }
  public var configFile: URL { configDirectory.appendingPathComponent("retroarch.cfg") }
  public var remapsDirectory: URL { configDirectory.appendingPathComponent("remaps", isDirectory: true) }

  public var assetsDirectory: URL { applicationDataRoot.appendingPathComponent("assets", isDirectory: true) }
  public var autoconfigDirectory: URL { applicationDataRoot.appendingPathComponent("autoconfig", isDirectory: true) }
  public var cheatsDirectory: URL {
    #if os(iOS) || os(tvOS) || os(macOS)
    applicationDataRoot.appendingPathComponent("cht", isDirectory: true)
    #else
    applicationDataRoot.appendingPathComponent("cheats", isDirectory: true)
    #endif
  }
  public var databaseDirectory: URL { applicationDataRoot.appendingPathComponent("database/rdb", isDirectory: true) }
  public var coreAssetsDirectory: URL { applicationDataRoot.appendingPathComponent("downloads", isDirectory: true) }
  public var downloadsDirectory: URL { coreAssetsDirectory }
  public var infoDirectory: URL { applicationDataRoot.appendingPathComponent("info", isDirectory: true) }
  public var overlaysDirectory: URL { applicationDataRoot.appendingPathComponent("overlays", isDirectory: true) }
  public var oskOverlaysDirectory: URL { overlaysDirectory.appendingPathComponent("keyboards", isDirectory: true) }
  public var shadersDirectory: URL { applicationDataRoot.appendingPathComponent("shaders", isDirectory: true) }
  public var thumbnailsDirectory: URL { applicationDataRoot.appendingPathComponent("thumbnails", isDirectory: true) }
  public var videoFiltersDirectory: URL { applicationDataRoot.appendingPathComponent("filters/video", isDirectory: true) }
  public var audioFiltersDirectory: URL { applicationDataRoot.appendingPathComponent("filters/audio", isDirectory: true) }

  public var contentDirectory: URL { root.appendingPathComponent("Roms", isDirectory: true) }
  public var logsDirectory: URL { root.appendingPathComponent("logs", isDirectory: true) }
  public var playlistsDirectory: URL { root.appendingPathComponent("playlists", isDirectory: true) }
  public var recordsDirectory: URL { root.appendingPathComponent("records", isDirectory: true) }
  public var recordsConfigDirectory: URL { root.appendingPathComponent("records_config", isDirectory: true) }
  public var savesDirectory: URL { root.appendingPathComponent("saves", isDirectory: true) }
  public var screenshotsDirectory: URL { root.appendingPathComponent("screenshots", isDirectory: true) }
  public var statesDirectory: URL { root.appendingPathComponent("states", isDirectory: true) }
  public var systemDirectory: URL { root.appendingPathComponent("system", isDirectory: true) }
  public var cacheDirectory: URL { root.appendingPathComponent("cache", isDirectory: true) }
  public var dynamicWallpapersDirectory: URL { root.appendingPathComponent("wallpapers", isDirectory: true) }

  public var overlayConfig: URL { overlaysDirectory.appendingPathComponent("gamepads/Named_Overlays/retropad.cfg") }

  public var retroArchSettings: [(key: String, url: URL)] {
    [
      ("libretro_directory", bundledCoreDirectory ?? applicationDataRoot.appendingPathComponent("cores", isDirectory: true)),
      ("libretro_info_path", infoDirectory),
      ("core_options_path", applicationDataRoot.appendingPathComponent("retroarch-core-options.cfg")),
      ("content_directory", contentDirectory),
      ("menu_content_directory", root),
      ("savefile_directory", savesDirectory),
      ("savestate_directory", statesDirectory),
      ("system_directory", systemDirectory),
      ("playlist_directory", playlistsDirectory),
      ("core_assets_directory", coreAssetsDirectory),
      ("assets_directory", assetsDirectory),
      ("menu_assets_directory", assetsDirectory),
      ("thumbnails_directory", thumbnailsDirectory),
      ("cache_directory", cacheDirectory),
      ("screenshot_directory", screenshotsDirectory),
      ("input_remapping_directory", remapsDirectory),
      ("cheat_database_path", cheatsDirectory),
      ("content_database_path", databaseDirectory),
      ("overlay_directory", overlaysDirectory),
      ("osk_overlay_directory", oskOverlaysDirectory),
      ("input_overlay", overlayConfig),
      ("joypad_autoconfig_dir", autoconfigDirectory),
      ("video_shader_dir", shadersDirectory),
      ("video_filter_dir", videoFiltersDirectory),
      ("audio_filter_dir", audioFiltersDirectory),
      ("log_dir", logsDirectory),
      ("recording_output_directory", recordsDirectory),
      ("recording_config_directory", recordsConfigDirectory),
      ("dynamic_wallpapers_directory", dynamicWallpapersDirectory),
    ]
  }

  public func createWritableDirectories() throws {
    for directory in writableDirectories {
      try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }
  }

  public var writableDirectories: [URL] {
    [
      root, applicationDataRoot, configDirectory, remapsDirectory, assetsDirectory,
      autoconfigDirectory, cheatsDirectory, databaseDirectory, coreAssetsDirectory,
      infoDirectory, overlaysDirectory, oskOverlaysDirectory, shadersDirectory,
      thumbnailsDirectory, videoFiltersDirectory, audioFiltersDirectory,
      contentDirectory, logsDirectory, playlistsDirectory, recordsDirectory,
      recordsConfigDirectory, savesDirectory, screenshotsDirectory, statesDirectory,
      systemDirectory, cacheDirectory, dynamicWallpapersDirectory,
    ]
  }
}
