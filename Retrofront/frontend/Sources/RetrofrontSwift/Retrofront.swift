import CRetrofrontCore
import Foundation

public enum RetrofrontError: Error, Equatable, CustomStringConvertible {
  case coreUnavailable
  case operationFailed(String)

  public var description: String {
    switch self {
    case .coreUnavailable:
      return "Retrofront Rust core is unavailable"
    case .operationFailed(let message):
      return message
    }
  }
}

public enum FrontendState: UInt32, Equatable, Sendable {
  case empty = 0
  case coreLoaded = 1
  case gameLoaded = 2
}

public struct LibretroSystemInfo: Equatable, Sendable {
  public let libraryName: String
  public let libraryVersion: String
  public let validExtensions: [String]
  public let needsFullPath: Bool
  public let blocksExtraction: Bool
}


public enum JoypadButton: UInt32, CaseIterable, Sendable {
  case b = 0
  case y = 1
  case select = 2
  case start = 3
  case up = 4
  case down = 5
  case left = 6
  case right = 7
  case a = 8
  case x = 9
  case l = 10
  case r = 11
  case l2 = 12
  case r2 = 13
  case l3 = 14
  case r3 = 15
}

public enum GfxBackend: UInt32, Equatable, Sendable {
  case software = 0
  case bgfx = 1
}

public enum GfxScaleMode: UInt32, Equatable, Sendable {
  case stretch = 0
  case keepAspect = 1
  case integer = 2
}

public enum GfxFilterMode: UInt32, Equatable, Sendable {
  case nearest = 0
  case linear = 1
}

public struct GfxVideoConfig: Equatable, Sendable {
  public let baseWidth: UInt32
  public let baseHeight: UInt32
  public let maxWidth: UInt32
  public let maxHeight: UInt32
  public let aspectRatio: Float
  public let outputWidth: UInt32
  public let outputHeight: UInt32
  public let scaleMode: GfxScaleMode
  public let filterMode: GfxFilterMode
  public let rotationQuarters: UInt32
  public let vsync: Bool

  public init(baseWidth: UInt32, baseHeight: UInt32, maxWidth: UInt32 = 0, maxHeight: UInt32 = 0, aspectRatio: Float = 0, outputWidth: UInt32 = 0, outputHeight: UInt32 = 0, scaleMode: GfxScaleMode = .keepAspect, filterMode: GfxFilterMode = .nearest, rotationQuarters: UInt32 = 0, vsync: Bool = true) {
    self.baseWidth = baseWidth
    self.baseHeight = baseHeight
    self.maxWidth = maxWidth
    self.maxHeight = maxHeight
    self.aspectRatio = aspectRatio
    self.outputWidth = outputWidth
    self.outputHeight = outputHeight
    self.scaleMode = scaleMode
    self.filterMode = filterMode
    self.rotationQuarters = rotationQuarters
    self.vsync = vsync
  }
}

public struct GfxDriverStatus: Equatable, Sendable {
  public let backend: GfxBackend
  public let frameNumber: UInt64
  public let hardwareReady: Bool
  public let rendered: Bool
}

public struct VideoFrame: Equatable, Sendable {
  public let width: UInt32
  public let height: UInt32
  public let sourcePitch: UInt64
  public let pixelFormat: UInt32
  public let frameNumber: UInt64
  public let rgba: [UInt8]
}

public enum FrontendEvent: Equatable, Sendable {
  case videoFrame(width: UInt32, height: UInt32, pitch: UInt64)
  case audioBatch(frames: UInt64)
  case audioSample(left: Int16, right: Int16)
  case environmentCommand(command: UInt32, handled: Bool)
  case inputPoll
}

public struct CoreOptionValue: Equatable, Sendable {
  public let value: String
  public let label: String
}

public struct CoreOption: Equatable, Sendable {
  public let key: String
  public let desc: String
  public let info: String
  public let value: String
  public let values: [CoreOptionValue]
}

public struct GameEntrySwift: Equatable, Sendable {
  public let path: String
  public let label: String
}

public enum LaunchDecision: UInt32, Equatable, Sendable {
  case noCore = 0
  case selected = 1
  case needsCoreChoice = 2
}

public struct LaunchPlan: Equatable, Sendable {
  public let contentPath: String
  public let contentExtension: String
  public let decision: LaunchDecision
  public let selectedCorePath: String?
  public let candidateCount: Int
  public let reason: String
}

public struct CoreInfo: Equatable, Sendable {
  public let path: String
  public let displayName: String
  public let systemName: String
  public let supportedExtensions: [String]
}

public enum MenuEntryKind: UInt32, Sendable {
  case action = 0
  case submenu = 1
  case toggle = 2
  case setting = 3
}

public struct MenuEntry: Sendable {
  public let label: String
  public let sublabel: String
  public let kind: MenuEntryKind
  public let value: String
  public let actionId: UInt32
}

public struct MenuList: Sendable {
  public let title: String
  public let entries: [MenuEntry]
}

public struct RetrofrontSetting: Equatable, Sendable {
  public let key: String
  public let value: String
}

public struct AssetInstallReport: Equatable, Sendable {
  public let filesWritten: Int
  public let directoriesCreated: Int
}

public struct OverlayInfo: Equatable, Sendable {
  public let enabled: Bool
  public let activeIndex: Int
  public let overlayCount: Int
  public let activeName: String
}

public struct OverlayRenderDesc: Equatable, Sendable {
  public let imagePath: String
  public let imageIndex: Int
  public let x: Float
  public let y: Float
  public let w: Float
  public let h: Float
  public let alpha: Float
}

public final class Retrofront: @unchecked Sendable {
  private let handle: OpaquePointer

  public init() throws {
    guard let handle = rf_frontend_create() else {
      throw RetrofrontError.coreUnavailable
    }
    self.handle = handle
  }

  deinit {
    rf_frontend_destroy(handle)
  }

  public var state: FrontendState {
    FrontendState(rawValue: rf_frontend_state(handle)) ?? .empty
  }

  @discardableResult
  public func loadCore(at path: String) throws -> LibretroSystemInfo {
    let ok = path.withCString { rf_frontend_load_core(handle, $0) }
    guard ok else { throw lastError() }
    return try systemInfo()
  }

  public func loadGame(at path: String, metadata: String? = nil) throws {
    let ok = path.withCString { cPath in
      if let metadata {
        return metadata.withCString { cMeta in
          rf_frontend_load_game(handle, cPath, cMeta)
        }
      }
      return rf_frontend_load_game(handle, cPath, nil)
    }
    guard ok else { throw lastError() }
  }

  public func launchContent(at path: String, preferredCore: String? = nil, metadata: String? = nil) throws {
    let ok = path.withCString { cPath in
      if let preferredCore {
        return preferredCore.withCString { cCore in
          if let metadata {
            return metadata.withCString { cMeta in rf_frontend_launch_content(handle, cPath, cCore, cMeta) }
          }
          return rf_frontend_launch_content(handle, cPath, cCore, nil)
        }
      }
      if let metadata {
        return metadata.withCString { cMeta in rf_frontend_launch_content(handle, cPath, nil, cMeta) }
      }
      return rf_frontend_launch_content(handle, cPath, nil, nil)
    }
    guard ok else { throw lastError() }
  }

  public func runFrame() throws -> [FrontendEvent] {
    guard rf_frontend_run_frame(handle) else {
      throw lastError()
    }
    return drainEvents()
  }

  public func unloadGame() {
    rf_frontend_unload_game(handle)
  }


  public func setJoypadButton(_ button: JoypadButton, pressed: Bool) throws {
    guard rf_frontend_set_joypad_button(handle, button.rawValue, pressed) else {
      throw lastError()
    }
  }

  public func loadOverlay(at path: String) throws {
    let ok = path.withCString { rf_frontend_load_overlay(handle, $0) }
    guard ok else { throw lastError() }
  }

  public func setOverlayEnabled(_ enabled: Bool) {
    rf_frontend_set_overlay_enabled(handle, enabled)
  }

  public func setOverlayTouch(slot: Int, x: Float, y: Float, active: Bool) throws {
    guard rf_frontend_set_overlay_touch(handle, UInt(slot), x, y, active) else {
      throw lastError()
    }
  }

  public func clearOverlayTouches() {
    rf_frontend_clear_overlay_touches(handle)
  }

  public func consumeOverlayMenuToggle() -> Bool {
    rf_frontend_consume_overlay_menu_toggle(handle)
  }

  public func overlayInfo() -> OverlayInfo? {
    var raw = RfOverlayInfo()
    guard rf_frontend_overlay_info(handle, &raw) else { return nil }
    return OverlayInfo(
      enabled: raw.enabled,
      activeIndex: Int(raw.active_index),
      overlayCount: Int(raw.overlay_count),
      activeName: raw.active_name.map { String(cString: $0) } ?? ""
    )
  }

  public func overlayRenderDescs() -> [OverlayRenderDesc] {
    let count = Int(rf_frontend_overlay_render_desc_count(handle))
    return (0..<count).compactMap { index in
      var raw = RfOverlayRenderDesc()
      guard rf_frontend_get_overlay_render_desc(handle, UInt(index), &raw) else { return nil }
      return OverlayRenderDesc(
        imagePath: raw.image_path.map { String(cString: $0) } ?? "",
        imageIndex: Int(raw.image_index),
        x: raw.x, y: raw.y, w: raw.w, h: raw.h, alpha: raw.alpha
      )
    }
  }

  public func setGfxBackend(_ backend: GfxBackend) throws {
    guard rf_frontend_set_gfx_backend(handle, backend.rawValue) else {
      throw lastError()
    }
  }


  public func gfxVideoConfig() -> GfxVideoConfig? {
    var raw = RfGfxVideoConfig()
    guard rf_frontend_get_gfx_video_config(handle, &raw) else { return nil }
    return GfxVideoConfig(
      baseWidth: raw.base_width,
      baseHeight: raw.base_height,
      maxWidth: raw.max_width,
      maxHeight: raw.max_height,
      aspectRatio: raw.aspect_ratio,
      outputWidth: raw.output_width,
      outputHeight: raw.output_height,
      scaleMode: GfxScaleMode(rawValue: raw.scale_mode) ?? .keepAspect,
      filterMode: GfxFilterMode(rawValue: raw.filter_mode) ?? .nearest,
      rotationQuarters: raw.rotation_quarters,
      vsync: raw.vsync
    )
  }

  public func setGfxVideoConfig(_ config: GfxVideoConfig) throws {
    var raw = RfGfxVideoConfig(
      base_width: config.baseWidth,
      base_height: config.baseHeight,
      max_width: config.maxWidth,
      max_height: config.maxHeight,
      aspect_ratio: config.aspectRatio,
      output_width: config.outputWidth,
      output_height: config.outputHeight,
      scale_mode: config.scaleMode.rawValue,
      filter_mode: config.filterMode.rawValue,
      rotation_quarters: config.rotationQuarters,
      vsync: config.vsync)
    guard rf_frontend_set_gfx_video_config(handle, &raw) else {
      throw lastError()
    }
  }

  public func setGfxHostHandles(_ handles: RfGfxHostHandles) throws {
    var raw = handles
    guard rf_frontend_set_gfx_host_handles(handle, &raw) else {
      throw lastError()
    }
  }

  public func gfxDriverStatus() -> GfxDriverStatus? {
    var info = RfGfxDriverInfo()
    guard rf_frontend_gfx_driver_info(handle, &info) else { return nil }
    return GfxDriverStatus(
      backend: GfxBackend(rawValue: info.backend) ?? .software,
      frameNumber: info.frame_number,
      hardwareReady: info.hardware_ready,
      rendered: info.rendered)
  }


  public struct VideoFrameInfo {
    public let width: UInt32
    public let height: UInt32
    public let pitch: UInt64
    public let pixelFormat: UInt32
    public let frameNumber: UInt64
    public let rgbaLen: UInt64
  }

  public func latestVideoFrameInfo() -> VideoFrameInfo? {
    var info = RfVideoFrameInfo()
    guard rf_frontend_video_frame_info(handle, &info) else { return nil }
    return VideoFrameInfo(
      width: info.width,
      height: info.height,
      pitch: info.pitch,
      pixelFormat: info.pixel_format,
      frameNumber: info.frame_number,
      rgbaLen: info.rgba_len
    )
  }

  public func copyLatestVideoFrame(to buffer: UnsafeMutableRawPointer, length: Int) -> Int {
    return Int(rf_frontend_copy_video_frame_rgba(handle, buffer, UInt(length)))
  }

  public func latestVideoFrame() -> VideoFrame? {
    var info = RfVideoFrameInfo()
    guard rf_frontend_video_frame_info(handle, &info) else { return nil }
    var rgba = [UInt8](repeating: 0, count: Int(info.rgba_len))
    let copied = rgba.withUnsafeMutableBufferPointer { buffer in
      rf_frontend_copy_video_frame_rgba(handle, buffer.baseAddress, UInt(buffer.count))
    }
    guard UInt64(copied) == info.rgba_len else { return nil }
    return VideoFrame(
      width: info.width,
      height: info.height,
      sourcePitch: info.pitch,
      pixelFormat: info.pixel_format,
      frameNumber: info.frame_number,
      rgba: rgba)
  }

  public func systemInfo() throws -> LibretroSystemInfo {
    var raw = RfSystemInfo()
    guard rf_frontend_system_info(handle, &raw) else {
      throw lastError()
    }
    let extensions = String(cString: raw.valid_extensions).split(separator: "|").map(String.init)
    return LibretroSystemInfo(
      libraryName: String(cString: raw.library_name),
      libraryVersion: String(cString: raw.library_version),
      validExtensions: extensions,
      needsFullPath: raw.need_fullpath,
      blocksExtraction: raw.block_extract
    )
  }

  public func drainEvents() -> [FrontendEvent] {
    var events: [FrontendEvent] = []
    var raw = RfEvent()
    while rf_frontend_next_event(handle, &raw) {
      if let event = FrontendEvent(raw) {
        events.append(event)
      }
    }
    return events
  }

  public func setOptionsConfigPath(_ path: String) throws {
    guard path.withCString({ rf_frontend_set_options_config_path(handle, $0) }) else {
      throw lastError()
    }
  }

  public func coreOptions() -> [CoreOption] {
    rf_frontend_clear_options_cache(handle)
    let count = rf_frontend_options_count(handle)
    var options: [CoreOption] = []
    for i in 0..<count {
      var raw = RfCoreOption()
      if rf_frontend_get_option(handle, UInt(i), &raw) {
        var values: [CoreOptionValue] = []
        for j in 0..<raw.values_count {
            let val = raw.values[Int(j)]
            values.append(CoreOptionValue(
                value: String(cString: val.value),
                label: String(cString: val.label)
            ))
        }
        options.append(CoreOption(
          key: String(cString: raw.key),
          desc: String(cString: raw.desc),
          info: String(cString: raw.info),
          value: String(cString: raw.value),
          values: values
        ))
      }
    }
    return options
  }

  public func setCoreOption(key: String, value: String) throws {
    guard key.withCString({ k in value.withCString({ v in rf_frontend_set_option(handle, k, v) }) }) else {
      throw lastError()
    }
  }

  public func clearOptionsCache() {
    rf_frontend_clear_options_cache(handle)
  }

  // Core Discovery
  public func setInfoDir(_ path: String) {
    path.withCString { rf_frontend_set_info_dir(handle, $0) }
  }

  public func scanCores(in directory: String) {
    directory.withCString { rf_frontend_scan_cores(handle, $0) }
  }

  public func scanConfiguredCores() {
    rf_frontend_scan_configured_cores(handle)
  }

  public func allSupportedExtensions() -> [String] {
    guard let pointer = rf_frontend_all_extensions(handle) else { return [] }
    return String(cString: pointer).split(separator: "|").map(String.init)
  }

  public func scanGames(in directory: String, extensions: String) {
    directory.withCString { d in
      extensions.withCString { e in
        rf_frontend_scan_games(handle, d, e)
      }
    }
  }


  public func planContentLaunch(path: String, preferredCore: String? = nil) -> LaunchPlan? {
    var raw = RfLaunchPlan()
    let ok = path.withCString { cPath in
      if let preferredCore {
        return preferredCore.withCString { cCore in rf_frontend_plan_content_launch(handle, cPath, cCore, &raw) }
      }
      return rf_frontend_plan_content_launch(handle, cPath, nil, &raw)
    }
    guard ok else { return nil }
    let selected = String(cString: raw.selected_core_path)
    return LaunchPlan(
      contentPath: String(cString: raw.content_path),
      contentExtension: String(cString: raw.content_extension),
      decision: LaunchDecision(rawValue: raw.decision) ?? .noCore,
      selectedCorePath: selected.isEmpty ? nil : selected,
      candidateCount: Int(raw.candidate_count),
      reason: String(cString: raw.reason)
    )
  }

  public func launchCandidates() -> [CoreInfo] {
    let count = rf_frontend_launch_candidate_count(handle)
    var cores: [CoreInfo] = []
    for i in 0..<count {
      var raw = RfCoreInfo()
      if rf_frontend_get_launch_candidate(handle, UInt(i), &raw) {
        cores.append(CoreInfo(
          path: String(cString: raw.path),
          displayName: String(cString: raw.display_name),
          systemName: String(cString: raw.system_name),
          supportedExtensions: String(cString: raw.supported_extensions).split(separator: "|").map(String.init)
        ))
      }
    }
    return cores
  }

  public func availableGames() -> [GameEntrySwift] {
    let count = rf_frontend_games_count(handle)
    var games: [GameEntrySwift] = []
    for i in 0..<count {
      var raw = RfGameEntry()
      if rf_frontend_get_game_info(handle, UInt(i), &raw) {
        games.append(GameEntrySwift(
          path: String(cString: raw.path),
          label: String(cString: raw.label)
        ))
      }
    }
    return games
  }

  public func availableCores() -> [CoreInfo] {
    let count = rf_frontend_cores_count(handle)
    var cores: [CoreInfo] = []
    for i in 0..<count {
      var raw = RfCoreInfo()
      if rf_frontend_get_core_info(handle, UInt(i), &raw) {
        cores.append(CoreInfo(
          path: String(cString: raw.path),
          displayName: String(cString: raw.display_name),
          systemName: String(cString: raw.system_name),
          supportedExtensions: String(cString: raw.supported_extensions).split(separator: "|").map(String.init)
        ))
      }
    }
    return cores
  }

  // Menu Engine
  public func currentMenuList() -> MenuList? {
    var raw = RfMenuList()
    guard rf_frontend_menu_current_list(handle, &raw) else { return nil }
    var entries: [MenuEntry] = []
    for i in 0..<raw.entry_count {
      var entryRaw = RfMenuEntry()
      if rf_frontend_menu_get_entry(handle, UInt(i), &entryRaw) {
        entries.append(MenuEntry(
          label: String(cString: entryRaw.label),
          sublabel: String(cString: entryRaw.sublabel),
          kind: MenuEntryKind(rawValue: entryRaw.kind) ?? .action,
          value: String(cString: entryRaw.value),
          actionId: entryRaw.action_id
        ))
      }
    }
    return MenuList(title: String(cString: raw.title), entries: entries)
  }

  public func pushCoreList() {
    rf_frontend_menu_push_core_list(handle)
  }

  public func pushContentList() {
    rf_frontend_menu_push_content_list(handle)
  }

  public func pushSettingsMenu() {
    rf_frontend_menu_push_settings(handle)
  }

  public func pushInformationMenu() {
    rf_frontend_menu_push_information(handle)
  }

  public func pushSkinSettingsMenu() {
    rf_frontend_menu_push_skin_settings(handle)
  }

  public func activateMenuAction(_ actionId: UInt32) -> Bool {
    return rf_frontend_menu_activate(handle, actionId)
  }

  public func menuPop() -> Bool {
    return rf_frontend_menu_pop(handle)
  }

  public func loadSettings(at path: String) throws {
    guard path.withCString({ rf_frontend_load_settings(handle, $0) }) else {
      throw lastError()
    }
  }

  public func setBaseDirectory(_ path: String) throws {
    guard path.withCString({ rf_frontend_set_base_dir(handle, $0) }) else {
      throw lastError()
    }
  }

  public func saveSettings() {
    rf_frontend_save_settings(handle)
  }

  public func setting(_ key: String) -> String? {
    key.withCString { cKey in
      guard let pointer = rf_frontend_get_setting(handle, cKey) else { return nil }
      return String(cString: pointer)
    }
  }

  public func setSetting(key: String, value: String) throws {
    guard key.withCString({ cKey in value.withCString({ cValue in rf_frontend_set_setting(handle, cKey, cValue) }) }) else {
      throw lastError()
    }
  }

  public func installAssetsZip(from zipPath: String, to destinationDir: String) throws -> AssetInstallReport {
    var raw = RfAssetInstallReport()
    let ok = zipPath.withCString { zip in
      destinationDir.withCString { dest in
        rf_frontend_install_assets_zip(handle, zip, dest, &raw)
      }
    }
    guard ok else { throw lastError() }
    return AssetInstallReport(filesWritten: Int(raw.files_written), directoriesCreated: Int(raw.directories_created))
  }

  public func settings() -> [RetrofrontSetting] {
    let count = rf_frontend_settings_count(handle)
    var settings: [RetrofrontSetting] = []
    for i in 0..<count {
      var raw = RfSettingEntry()
      if rf_frontend_get_setting_at(handle, UInt(i), &raw) {
        settings.append(RetrofrontSetting(key: String(cString: raw.key), value: String(cString: raw.value)))
      }
    }
    return settings
  }

  private func lastError() -> RetrofrontError {
    guard let pointer = rf_frontend_last_error(handle) else {
      return .operationFailed("unknown Retrofront core error")
    }
    let message = String(cString: pointer)
    return .operationFailed(message.isEmpty ? "unknown Retrofront core error" : message)
  }
}

extension FrontendEvent {
  fileprivate init?(_ raw: RfEvent) {
    switch raw.kind {
    case 1:
      self = .videoFrame(width: UInt32(raw.a), height: UInt32(raw.b), pitch: raw.c)
    case 2:
      self = .audioBatch(frames: raw.a)
    case 3:
      self = .audioSample(
        left: Int16(bitPattern: UInt16(truncatingIfNeeded: raw.a)),
        right: Int16(bitPattern: UInt16(truncatingIfNeeded: raw.b)))
    case 4:
      self = .environmentCommand(command: UInt32(raw.a), handled: raw.b != 0)
    case 5:
      self = .inputPoll
    default:
      return nil
    }
  }
}
