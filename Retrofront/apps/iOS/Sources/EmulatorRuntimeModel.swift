import Foundation
import RetrofrontSwift
import UIKit
import Combine
import UniformTypeIdentifiers

public struct OverlayChoice: Identifiable, Equatable, Sendable {
  public let id: String
  public let path: String
  public let label: String
}

@MainActor
public final class EmulatorRuntimeModel: ObservableObject {
  @Published private(set) var frontendState: FrontendState = .empty
  @Published private(set) var systemInfo: LibretroSystemInfo?
  @Published private(set) var coreOptions: [CoreOption] = []
  @Published private(set) var displayImage: UIImage?
  @Published private(set) var aspectRatio: Double = 4.0/3.0
  @Published private(set) var isRunning = false
  @Published private(set) var availableCores: [CoreInfo] = []
  @Published private(set) var availableGames: [GameEntrySwift] = []
  @Published private(set) var corePath: String?
  @Published private(set) var loadedGameURL: URL?
  @Published private(set) var currentMenu: MenuList?
  @Published private(set) var overlayInfo: OverlayInfo?
  @Published private(set) var availableOverlays: [OverlayChoice] = []
  @Published private(set) var settings: [RetrofrontSetting] = []
  @Published private(set) var pendingCoreChoices: [CoreInfo] = []
  @Published private(set) var pendingContentURL: URL?
  @Published private(set) var launchToken: UInt = 0
  @Published private(set) var menuToken: UInt = 0
  @Published var statusMessage = "Ready"

  private var frontend: Retrofront?
  private var runTask: Task<Void, Never>?
  private var pixelBuffer: Data?

  public init() {
    setupFrontend()
    refreshAvailableCores()
    refreshGames()
    refreshMenu()
    overlayInfo = frontend?.overlayInfo()
  }

  private var storageLayout: RetroArchStorageLayout { .current }

  private var retroArchRoot: URL { storageLayout.root }

  private func setupFrontend() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend

      let layout = storageLayout
      let root = layout.root
      try layout.createWritableDirectories()
      let configPath = layout.configFile.path
      try? frontend.setBaseDirectory(root.path)
      applyBundleCoreDirectories(frontend)
      try? frontend.loadSettings(at: configPath)
      applyBundleCoreDirectories(frontend)
      applyRetroArchStorageSettings(frontend, layout: layout)
      installBundledAssetsIfNeeded(frontend)
      refreshOverlayChoices()
      applyRendererSetting(frontend)
      applyVideoSettings(frontend)
      loadConfiguredOverlay(frontend)
      frontend.saveSettings()
      refresh()
    } catch {
      statusMessage = "Initialization failed: \(error)"
    }
  }

  private func applyBundleCoreDirectories(_ frontend: Retrofront) {
    let layout = storageLayout
    frontend.setInfoDir(layout.infoDirectory.path)
    for setting in layout.retroArchSettings {
      try? frontend.setSetting(key: setting.key, value: setting.url.path)
    }
  }

  private func applyRetroArchStorageSettings(_ frontend: Retrofront, layout: RetroArchStorageLayout) {
    for setting in layout.retroArchSettings {
      try? frontend.setSetting(key: setting.key, value: setting.url.path)
    }
  }

  public func refreshAvailableCores() {
    guard let frontend else { return }
    if let frameworksURL = Bundle.main.privateFrameworksURL {
      frontend.scanCores(in: frameworksURL.path)
    }
    if let resourceURL = Bundle.main.resourceURL {
      frontend.scanCores(in: resourceURL.path)
      frontend.scanCores(in: resourceURL.appendingPathComponent("dylibs").path)
    }
    frontend.scanCores(in: storageLayout.retroArchSettings.first(where: { $0.key == "libretro_directory" })?.url.path ?? retroArchRoot.appendingPathComponent("cores", isDirectory: true).path)
    frontend.scanCores(in: retroArchRoot.appendingPathComponent("cores", isDirectory: true).path)
    frontend.scanCores(in: retroArchRoot.appendingPathComponent("Cores", isDirectory: true).path)
    frontend.scanConfiguredCores()
    availableCores = frontend.availableCores()
  }

  public func refreshGames() {
    guard let frontend else { return }
    refreshAvailableCores()
    let contentDir = URL(fileURLWithPath: frontend.setting("content_directory") ?? storageLayout.contentDirectory.path)
    try? FileManager.default.createDirectory(at: contentDir, withIntermediateDirectories: true)
    let exts = frontend.allSupportedExtensions().joined(separator: "|")
    frontend.scanGames(in: contentDir.path, extensions: exts)
    availableGames = sortedGames(frontend.availableGames())
  }

  public func rescanLibrary() {
    refreshAvailableCores()
    refreshGames()
    refresh()
    statusMessage = "Library refreshed"
  }

  public func importFile(at url: URL) {
    importGame(at: url)
  }

  public func importGame(at url: URL) {
    guard let frontend else { return }
    let destinationDir = URL(fileURLWithPath: frontend.setting("content_directory") ?? storageLayout.contentDirectory.path)
    try? FileManager.default.createDirectory(at: destinationDir, withIntermediateDirectories: true)
    let destination = destinationDir.appendingPathComponent(url.lastPathComponent)
    let success = url.startAccessingSecurityScopedResource()
    defer { if success { url.stopAccessingSecurityScopedResource() } }
    do {
      if FileManager.default.fileExists(atPath: destination.path) {
        try FileManager.default.removeItem(at: destination)
      }
      try FileManager.default.copyItem(at: url, to: destination)
      statusMessage = "Imported \(url.lastPathComponent)"
      refreshGames()
    } catch {
      statusMessage = "Import failed: \(error)"
    }
  }

  private func installBundledAssetsIfNeeded(_ frontend: Retrofront) {
    let infoProbe = storageLayout.infoDirectory.appendingPathComponent("mgba_libretro.info")
    guard !FileManager.default.fileExists(atPath: infoProbe.path) else {
      applyBundleCoreDirectories(frontend)
      return
    }
    installBundledAssets(frontend, updateStatus: false)
  }

  public func installBundledAssets() {
    guard let frontend else { return }
    installBundledAssets(frontend, updateStatus: true)
  }

  private func installBundledAssets(_ frontend: Retrofront, updateStatus: Bool) {
    guard let zipURL = Bundle.main.url(forResource: "assets", withExtension: "zip") else {
      if updateStatus { statusMessage = "assets.zip was not found in the app bundle" }
      return
    }
    let installRoot = storageLayout.root
    do {
      let report = try frontend.installAssetsZip(from: zipURL.path, to: installRoot.path)
      applyBundleCoreDirectories(frontend)
      refreshOverlayChoices()
      loadConfiguredOverlay(frontend)
      refresh()
      if updateStatus { statusMessage = "Installed assets: \(report.filesWritten) files" }
    } catch {
      if updateStatus { statusMessage = "Assets install failed: \(error)" }
    }
  }

  public func loadCore(_ core: CoreInfo) {
    guard let frontend else { return }
    do {
      stop()
      systemInfo = try frontend.loadCore(at: core.path)
      corePath = core.path
      refresh()
      statusMessage = "Loaded core: \(systemInfo?.libraryName ?? core.displayName)"
    } catch {
      statusMessage = "Core load failed: \(error)"
    }
  }

  public func loadGame(at url: URL) {
    guard let frontend else { return }
    refreshAvailableCores()
    guard FileManager.default.fileExists(atPath: url.path) else {
      statusMessage = "Game file missing: \(url.lastPathComponent)"
      refreshGames()
      return
    }
    guard let plan = frontend.planContentLaunch(path: url.path) else {
      statusMessage = "Could not create launch plan"
      return
    }
    switch plan.decision {
    case .selected:
      doLaunch(url, preferredCore: plan.selectedCorePath)
    case .needsCoreChoice:
      pendingContentURL = url
      pendingCoreChoices = frontend.launchCandidates()
      statusMessage = "Select a core for .\(plan.contentExtension)"
    case .noCore:
      statusMessage = "No compatible core found for .\(plan.contentExtension). Load a matching bundled core first."
    }
  }

  public func launchPendingContent(with core: CoreInfo) {
    guard let url = pendingContentURL else { return }
    pendingContentURL = nil
    pendingCoreChoices = []
    doLaunch(url, preferredCore: core.path)
  }

  public func cancelCoreChoice() {
    pendingContentURL = nil
    pendingCoreChoices = []
  }

  private func doLaunch(_ url: URL, preferredCore: String?) {
    guard let frontend else { return }
    do {
      stop()
      try frontend.launchContent(at: url.path, preferredCore: preferredCore)
      loadedGameURL = url
      systemInfo = try? frontend.systemInfo()
      corePath = preferredCore ?? frontend.planContentLaunch(path: url.path)?.selectedCorePath
      refresh()
      launchToken &+= 1
      statusMessage = "Loaded game: \(url.lastPathComponent)"
    } catch {
      statusMessage = "Game load failed: \(error)"
    }
  }

  public func setJoypadButton(_ button: JoypadButton, pressed: Bool) {
    try? frontend?.setJoypadButton(button, pressed: pressed)
  }

  public func setOverlayTouch(slot: Int, location: CGPoint, in size: CGSize, active: Bool) {
    guard size.width > 0, size.height > 0 else { return }
    let x = Float(max(0, min(1, location.x / size.width)))
    let y = Float(max(0, min(1, location.y / size.height)))
    try? frontend?.setOverlayTouch(slot: slot, x: x, y: y, active: active)
    if frontend?.consumeOverlayMenuToggle() == true {
      menuAction(8)
      menuToken &+= 1
    }
  }

  public func clearOverlayTouches() {
    frontend?.clearOverlayTouches()
  }

  private func loadConfiguredOverlay(_ frontend: Retrofront) {
    let enabled = frontend.setting("input_overlay_enable") != "false"
    if let path = frontend.setting("input_overlay"), FileManager.default.fileExists(atPath: path) {
      try? frontend.loadOverlay(at: path)
    }
    frontend.setOverlayEnabled(enabled)
    overlayInfo = frontend.overlayInfo()
  }

  public func overlayRenderDescs() -> [OverlayRenderDesc] {
    frontend?.overlayRenderDescs() ?? []
  }

  public func setOverlayOrientation(for size: CGSize) {
    guard size.width > 0, size.height > 0 else { return }
    do {
      try frontend?.setOverlayOrientation(portrait: size.height > size.width)
      overlayInfo = frontend?.overlayInfo()
    } catch {
      // Some overlays do not define portrait/landscape variants; keep the current overlay.
    }
  }

  public func refreshOverlayChoices() {
    guard let frontend else { return }
    let overlayDir = URL(fileURLWithPath: frontend.setting("overlay_directory") ?? storageLayout.overlaysDirectory.path)
    let fm = FileManager.default
    guard let enumerator = fm.enumerator(at: overlayDir, includingPropertiesForKeys: nil) else {
      availableOverlays = []
      return
    }
    var choices: [OverlayChoice] = []
    for case let url as URL in enumerator where url.pathExtension.lowercased() == "cfg" {
      let relative = url.path.replacingOccurrences(of: overlayDir.path + "/", with: "")
      let isGamepad = relative.contains("gamepads/")
      let label = relative.replacingOccurrences(of: ".cfg", with: "")
      choices.append(OverlayChoice(id: url.path, path: url.path, label: isGamepad ? label : "Other / \(label)"))
    }
    availableOverlays = choices.sorted { left, right in
      let leftGamepad = left.label.hasPrefix("gamepads/")
      let rightGamepad = right.label.hasPrefix("gamepads/")
      if leftGamepad != rightGamepad { return leftGamepad }
      return left.label.localizedStandardCompare(right.label) == .orderedAscending
    }
  }

  private func applyVideoSettings(_ frontend: Retrofront) {
    var config = frontend.gfxVideoConfig() ?? GfxVideoConfig(baseWidth: 0, baseHeight: 0)
    config = GfxVideoConfig(
      baseWidth: config.baseWidth,
      baseHeight: config.baseHeight,
      maxWidth: config.maxWidth,
      maxHeight: config.maxHeight,
      aspectRatio: config.aspectRatio,
      outputWidth: config.outputWidth,
      outputHeight: config.outputHeight,
      scaleMode: scaleModeFromSetting(frontend.setting("video_scale_mode")),
      filterMode: filterModeFromSetting(frontend.setting("video_filter_mode")),
      rotationQuarters: config.rotationQuarters,
      vsync: frontend.setting("video_vsync") != "false")
    try? frontend.setGfxVideoConfig(config)
  }

  public func toggleRunning() { isRunning ? stop() : play() }

  public func play() {
    guard frontendState == .gameLoaded, !isRunning else { return }
    isRunning = true
    runTask = Task.detached(priority: .userInitiated) { [weak self] in
      while !Task.isCancelled {
        guard let self = self else { break }
        let shouldStop = await MainActor.run { self.runOneFrame() }
        if shouldStop { break }
        try? await Task.sleep(nanoseconds: 16_666_667)
      }
    }
  }

  public func stop() {
    isRunning = false
    runTask?.cancel()
    runTask = nil
    try? frontend?.saveSRAM()
  }

  public func resetContent() {
    do {
      try frontend?.reset()
      statusMessage = "Game reset"
    } catch {
      statusMessage = "Reset failed: \(error)"
    }
    refresh()
  }

  public func saveSRAMNow() {
    do {
      try frontend?.saveSRAM()
      statusMessage = "SRAM saved"
    } catch {
      statusMessage = "SRAM save failed: \(error)"
    }
    refresh()
  }

  public func saveState(slot: UInt32 = 0) {
    do {
      try frontend?.saveState(slot: slot)
      statusMessage = "State saved to slot \(slot)"
    } catch {
      statusMessage = "Save state failed: \(error)"
    }
    refresh()
  }

  public func loadState(slot: UInt32 = 0) {
    do {
      try frontend?.loadState(slot: slot)
      statusMessage = "State loaded from slot \(slot)"
    } catch {
      statusMessage = "Load state failed: \(error)"
    }
    refresh()
  }

  public func closeContent() {
    stop()
    frontend?.unloadGame()
    loadedGameURL = nil
    displayImage = nil
    frontendState = frontend?.state ?? .empty
    refresh()
    statusMessage = "Game exited"
  }

  @discardableResult
  private func runOneFrame() -> Bool {
    guard let frontend else { return true }
    do {
      _ = try frontend.runFrame()
      if let info = frontend.latestVideoFrameInfo() {
        if pixelBuffer == nil || pixelBuffer?.count != Int(info.rgbaLen) {
          pixelBuffer = Data(count: Int(info.rgbaLen))
        }
        let copied = pixelBuffer?.withUnsafeMutableBytes { buffer -> Int in
          guard let base = buffer.baseAddress else { return 0 }
          return frontend.copyLatestVideoFrame(to: base, length: buffer.count)
        } ?? 0
        if copied == Int(info.rgbaLen), let data = pixelBuffer {
          displayImage = Self.imageFromData(data, width: Int(info.width), height: Int(info.height))
        }
      }
      return false
    } catch {
      Task { @MainActor in
        self.stop()
        self.statusMessage = "Run error: \(error)"
      }
      return true
    }
  }

  public func refresh() {
    guard let frontend else { return }
    frontendState = frontend.state
    coreOptions = frontend.coreOptions()
    settings = frontend.settings()
    if let config = frontend.gfxVideoConfig() {
      if config.aspectRatio > 0 { aspectRatio = Double(config.aspectRatio) }
      else if config.baseHeight > 0 { aspectRatio = Double(config.baseWidth) / Double(config.baseHeight) }
    }
    refreshMenu()
    overlayInfo = frontend.overlayInfo()
  }

  public func refreshMenu() { currentMenu = frontend?.currentMenuList() }

  public func menuAction(_ actionId: UInt32) {
    guard let frontend else { return }
    if handleMenuSettingAction(actionId) {
      refreshMenu()
      return
    }
    switch actionId {
    case 9:
      resetContent()
      return
    case 12, 26:
      closeContent()
      return
    case 27:
      saveState(slot: 0)
      return
    case 28:
      loadState(slot: 0)
      return
    case 29:
      saveSRAMNow()
      return
    default:
      break
    }
    if frontend.activateMenuAction(actionId) { refreshMenu() }
  }

  private func handleMenuSettingAction(_ actionId: UInt32) -> Bool {
    switch actionId {
    case 621, 694:
      cycleVideoScaleMode()
    case 622:
      toggleVideoFilter()
    case 623:
      setVsyncEnabled(!vsyncEnabled)
    case 641:
      setAudioEnabled(!audioEnabledSetting)
    case 642:
      setAudioSync(!audioSyncSetting)
    case 650:
      cycleAudioLatency()
    case 664:
      setOverlayEnabledSetting(!overlayEnabledSetting)
    case 665:
      setHapticsEnabled(!hapticsEnabledSetting)
    case 690:
      cycleSetting(key: "play_screen_orientation", values: ["auto", "portrait", "landscape"])
      statusMessage = "Play orientation: \(settingValue("play_screen_orientation"))"
    case 715:
      cycleLibrarySort()
    case 716:
      setLibraryCoreBadgesEnabled(!libraryCoreBadgesEnabled)
    case 717:
      setLibraryFileDetailsEnabled(!libraryFileDetailsEnabled)
    case 718:
      setLibraryAutoScanEnabled(!libraryAutoScanEnabled)
    case 725:
      let enabled = settingValue("savestate_auto_save") != "true"
      setSetting(key: "savestate_auto_save", value: enabled ? "true" : "false")
      statusMessage = enabled ? "Auto save state enabled" : "Auto save state disabled"
    case 726:
      let enabled = settingValue("savestate_auto_load") != "true"
      setSetting(key: "savestate_auto_load", value: enabled ? "true" : "false")
      statusMessage = enabled ? "Auto load state enabled" : "Auto load state disabled"
    default:
      return false
    }
    return true
  }

  public func openQuickMenu() {
    menuAction(8)
    menuToken &+= 1
  }

  public func menuPop() {
    if frontend?.menuPop() == true { refreshMenu() }
  }

  public func settingValue(_ key: String) -> String {
    frontend?.setting(key) ?? settings.first(where: { $0.key == key })?.value ?? ""
  }

  public var overlayEnabledSetting: Bool {
    settingValue("input_overlay_enable") != "false"
  }

  public var hapticsEnabledSetting: Bool {
    settingValue("input_haptic_feedback") != "false"
  }

  public var audioEnabledSetting: Bool {
    settingValue("audio_enable") != "false"
  }

  public var audioSyncSetting: Bool {
    settingValue("audio_sync") != "false"
  }

  public var audioLatencyLabel: String {
    let value = settingValue("audio_latency_ms")
    return value.isEmpty ? "64 ms" : "\(value) ms"
  }

  public var libraryCoreBadgesEnabled: Bool {
    settingValue("library_show_core_badges") != "false"
  }

  public var libraryFileDetailsEnabled: Bool {
    settingValue("library_show_file_details") != "false"
  }

  public var libraryAutoScanEnabled: Bool {
    settingValue("library_auto_scan_on_launch") != "false"
  }

  public var librarySortLabel: String {
    switch settingValue("library_sort_mode") {
    case "extension": return "Extension"
    case "name_descending": return "Name ↓"
    default: return "Name ↑"
    }
  }

  public var libraryRomTypeCount: Int {
    Set(availableGames.map { URL(fileURLWithPath: $0.path).pathExtension.lowercased() }.filter { !$0.isEmpty }).count
  }

  public var overlayOpacityLabel: String {
    let value = settingValue("input_overlay_opacity")
    return value.isEmpty ? "70%" : "\(Int((Double(value) ?? 0.70) * 100))%"
  }

  public var overlaySelectionLabel: String {
    let current = settingValue("input_overlay")
    if let match = availableOverlays.first(where: { $0.path == current }) { return match.label }
    return current.isEmpty ? "Not selected" : URL(fileURLWithPath: current).deletingPathExtension().lastPathComponent
  }

  public var videoScaleModeLabel: String {
    switch settingValue("video_scale_mode") {
    case "integer": return "Integer"
    case "stretch": return "Stretch"
    default: return "Aspect"
    }
  }

  public var videoFilterLabel: String {
    settingValue("video_filter_mode") == "linear" ? "Linear" : "Nearest"
  }

  public var rendererLabel: String {
    switch settingValue("video_driver") {
    case "software": return "Software"
    case "moltenvk": return "MoltenVK"
    case "opengl": return "OpenGL ES"
    default: return "Metal"
    }
  }

  public var vsyncEnabled: Bool {
    settingValue("video_vsync") != "false"
  }

  public var videoScaleModeChoices: [(label: String, value: String)] {
    [("Aspect", "keep_aspect"), ("Integer", "integer"), ("Stretch", "stretch")]
  }

  public var videoFilterChoices: [(label: String, value: String)] {
    [("Nearest", "nearest"), ("Linear", "linear")]
  }

  public var rendererChoices: [(label: String, value: String)] {
    [("Metal", "metal"), ("Software", "software"), ("MoltenVK", "moltenvk"), ("OpenGL ES", "opengl")]
  }

  public var audioLatencyChoices: [(label: String, value: String)] {
    [("32 ms", "32"), ("64 ms", "64"), ("96 ms", "96"), ("128 ms", "128")]
  }

  public var overlayOpacityChoices: [(label: String, value: String)] {
    [("45%", "0.45"), ("70%", "0.70"), ("90%", "0.90")]
  }

  public var librarySortChoices: [(label: String, value: String)] {
    [("Name ↑", "name_ascending"), ("Name ↓", "name_descending"), ("Extension", "extension")]
  }

  public func setOverlayEnabledSetting(_ enabled: Bool) {
    setSetting(key: "input_overlay_enable", value: enabled ? "true" : "false")
    frontend?.setOverlayEnabled(enabled)
    refresh()
    statusMessage = enabled ? "Touch overlay enabled" : "Touch overlay disabled"
  }

  public func setHapticsEnabled(_ enabled: Bool) {
    setSetting(key: "input_haptic_feedback", value: enabled ? "true" : "false")
    statusMessage = enabled ? "Haptics enabled" : "Haptics disabled"
  }

  public func setAudioEnabled(_ enabled: Bool) {
    setSetting(key: "audio_enable", value: enabled ? "true" : "false")
    statusMessage = enabled ? "Audio enabled" : "Audio disabled"
  }

  public func setAudioSync(_ enabled: Bool) {
    setSetting(key: "audio_sync", value: enabled ? "true" : "false")
    statusMessage = enabled ? "Audio sync enabled" : "Audio sync disabled"
  }


  public func setAudioLatency(_ value: String) {
    setSetting(key: "audio_latency_ms", value: value)
    statusMessage = "Audio latency: \(audioLatencyLabel)"
  }

  public func setLibrarySort(_ value: String) {
    setSetting(key: "library_sort_mode", value: value)
    refreshGames()
    statusMessage = "Library sort: \(librarySortLabel)"
  }

  public func selectOverlay(_ choice: OverlayChoice) {
    setSetting(key: "input_overlay", value: choice.path)
    if let frontend { loadConfiguredOverlay(frontend) }
    statusMessage = "Overlay: \(choice.label)"
  }

  public func setOverlayOpacity(_ value: String) {
    setSetting(key: "input_overlay_opacity", value: value)
    if let frontend { loadConfiguredOverlay(frontend) }
    statusMessage = "Overlay opacity: \(overlayOpacityLabel)"
  }

  public func setVideoScaleMode(_ value: String) {
    setSetting(key: "video_scale_mode", value: value)
    if let frontend { applyVideoSettings(frontend) }
    refresh()
    statusMessage = "Video scale: \(videoScaleModeLabel)"
  }

  public func setVideoFilter(_ value: String) {
    setSetting(key: "video_filter_mode", value: value)
    if let frontend { applyVideoSettings(frontend) }
    refresh()
    statusMessage = "Video filter: \(videoFilterLabel)"
  }

  public func setRenderer(_ value: String) {
    setSetting(key: "video_driver", value: value)
    if let frontend { applyRendererSetting(frontend) }
    refresh()
    statusMessage = "Renderer: \(rendererLabel)"
  }

  public func loadBundledCore(_ core: CoreInfo) {
    loadCore(core)
  }


  public func cycleAudioLatency() {
    cycleSetting(key: "audio_latency_ms", values: ["32", "64", "96", "128"])
    statusMessage = "Audio latency: \(audioLatencyLabel)"
  }

  public func cycleLibrarySort() {
    cycleSetting(key: "library_sort_mode", values: ["name_ascending", "name_descending", "extension"])
    refreshGames()
    statusMessage = "Library sort: \(librarySortLabel)"
  }

  public func setLibraryCoreBadgesEnabled(_ enabled: Bool) {
    setSetting(key: "library_show_core_badges", value: enabled ? "true" : "false")
    statusMessage = enabled ? "Core badges enabled" : "Core badges hidden"
  }

  public func setLibraryFileDetailsEnabled(_ enabled: Bool) {
    setSetting(key: "library_show_file_details", value: enabled ? "true" : "false")
    statusMessage = enabled ? "ROM details enabled" : "ROM details hidden"
  }

  public func setLibraryAutoScanEnabled(_ enabled: Bool) {
    setSetting(key: "library_auto_scan_on_launch", value: enabled ? "true" : "false")
    statusMessage = enabled ? "Library auto scan enabled" : "Library auto scan disabled"
  }

  public func cycleOverlaySelection() {
    refreshOverlayChoices()
    guard !availableOverlays.isEmpty else {
      statusMessage = "No overlays found"
      return
    }
    let current = settingValue("input_overlay")
    let currentIndex = availableOverlays.firstIndex { $0.path == current } ?? -1
    let next = availableOverlays[(currentIndex + 1) % availableOverlays.count]
    setSetting(key: "input_overlay", value: next.path)
    if let frontend { loadConfiguredOverlay(frontend) }
    statusMessage = "Overlay: \(next.label)"
  }

  public func cycleOverlayOpacity() {
    let values = ["0.45", "0.70", "0.90"]
    cycleSetting(key: "input_overlay_opacity", values: values)
    if let frontend { loadConfiguredOverlay(frontend) }
    statusMessage = "Overlay opacity: \(overlayOpacityLabel)"
  }

  public func cycleVideoScaleMode() {
    let values = ["keep_aspect", "integer", "stretch"]
    cycleSetting(key: "video_scale_mode", values: values)
    if let frontend { applyVideoSettings(frontend) }
    refresh()
    statusMessage = "Video scale: \(videoScaleModeLabel)"
  }

  public func toggleVideoFilter() {
    setSetting(key: "video_filter_mode", value: settingValue("video_filter_mode") == "linear" ? "nearest" : "linear")
    if let frontend { applyVideoSettings(frontend) }
    refresh()
    statusMessage = "Video filter: \(videoFilterLabel)"
  }

  public func setVsyncEnabled(_ enabled: Bool) {
    setSetting(key: "video_vsync", value: enabled ? "true" : "false")
    if let frontend { applyVideoSettings(frontend) }
    refresh()
    statusMessage = enabled ? "VSync enabled" : "VSync disabled"
  }

  public func romDetails(for game: GameEntrySwift) -> String {
    guard libraryFileDetailsEnabled else { return URL(fileURLWithPath: game.path).deletingLastPathComponent().lastPathComponent }
    let url = URL(fileURLWithPath: game.path)
    let ext = url.pathExtension.isEmpty ? "ROM" : url.pathExtension.uppercased()
    let size = (try? FileManager.default.attributesOfItem(atPath: game.path)[.size] as? NSNumber)?.int64Value ?? 0
    let formatted = ByteCountFormatter.string(fromByteCount: size, countStyle: .file)
    return "\(ext) • \(formatted)"
  }

  public func compatibleCoreSummary(for game: GameEntrySwift) -> String {
    guard libraryCoreBadgesEnabled, let frontend else { return URL(fileURLWithPath: game.path).lastPathComponent }
    guard let plan = frontend.planContentLaunch(path: game.path) else { return "Compatibility unknown" }
    switch plan.decision {
    case .selected:
      if let selected = plan.selectedCorePath,
         let core = availableCores.first(where: { $0.path == selected }) {
        return "Ready with \(core.displayName)"
      }
      return "Ready • \(plan.candidateCount) core"
    case .needsCoreChoice:
      return "\(plan.candidateCount) compatible cores"
    case .noCore:
      return "No compatible core for .\(plan.contentExtension)"
    }
  }

  private func sortedGames(_ games: [GameEntrySwift]) -> [GameEntrySwift] {
    switch settingValue("library_sort_mode") {
    case "extension":
      return games.sorted {
        let left = URL(fileURLWithPath: $0.path).pathExtension.lowercased()
        let right = URL(fileURLWithPath: $1.path).pathExtension.lowercased()
        return left == right ? $0.label.localizedCaseInsensitiveCompare($1.label) == .orderedAscending : left < right
      }
    case "name_descending":
      return games.sorted { $0.label.localizedCaseInsensitiveCompare($1.label) == .orderedDescending }
    default:
      return games.sorted { $0.label.localizedCaseInsensitiveCompare($1.label) == .orderedAscending }
    }
  }

  public func setSetting(key: String, value: String) {
    guard let frontend else { return }
    do {
      try frontend.setSetting(key: key, value: value)
      frontend.saveSettings()
      refresh()
    } catch {
      statusMessage = "Setting failed: \(error)"
    }
  }

  private func cycleSetting(key: String, values: [String]) {
    let current = settingValue(key)
    let next: String
    if let index = values.firstIndex(of: current) {
      next = values[(index + 1) % values.count]
    } else {
      next = values.first ?? current
    }
    setSetting(key: key, value: next)
  }

  private func scaleModeFromSetting(_ value: String?) -> GfxScaleMode {
    switch value {
    case "integer": return .integer
    case "stretch": return .stretch
    default: return .keepAspect
    }
  }

  private func filterModeFromSetting(_ value: String?) -> GfxFilterMode {
    value == "linear" ? .linear : .nearest
  }

  private func applyRendererSetting(_ frontend: Retrofront) {
    let value = frontend.setting("video_driver") ?? "metal"
    let backend: GfxBackend = switch value.lowercased() {
    case "software": .software
    case "opengl", "gl", "gles", "opengles": .openGL
    case "vulkan", "vulkn": .vulkan
    case "moltenvk": .moltenVK
    case "metal": .metal
    default: .wgpu
    }
    try? frontend.setGfxBackend(backend)
    try? frontend.setSetting(key: "video_wgpu_renderer", value: value == "software" ? "metal" : value)
  }

  public func setOption(key: String, value: String) {
    guard let frontend else { return }
    try? frontend.setCoreOption(key: key, value: value)
    refresh()
  }

  private static func imageFromData(_ data: Data, width: Int, height: Int) -> UIImage? {
    guard width > 0, height > 0 else { return nil }
    guard let provider = CGDataProvider(data: data as CFData) else { return nil }
    guard let cgImage = CGImage(
      width: width,
      height: height,
      bitsPerComponent: 8,
      bitsPerPixel: 32,
      bytesPerRow: width * 4,
      space: CGColorSpaceCreateDeviceRGB(),
      bitmapInfo: CGBitmapInfo.byteOrder32Big.union(CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue)),
      provider: provider,
      decode: nil,
      shouldInterpolate: false,
      intent: .defaultIntent
    ) else { return nil }
    return UIImage(cgImage: cgImage)
  }
}
