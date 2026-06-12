import Foundation
import RetrofrontSwift

extension EmulatorRuntimeModel {
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
      saveState()
      return
    case 28:
      loadState()
      return
    case 29:
      saveSRAMNow()
      return
    default:
      break
    }
    if frontend.activateMenuAction(actionId) { refreshMenu() }
  }

  func handleMenuSettingAction(_ actionId: UInt32) -> Bool {
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
    case 38:
      cycleStateSlot(delta: -1)
    case 39:
      cycleStateSlot(delta: 1)
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


  public var menuDriverChoices: [(label: String, value: String)] {
    [("Material UI", "materialui"), ("Ozone", "ozone"), ("XMB", "xmb"), ("RGUI", "rgui"), ("One UI (fallback)", "oneui")]
  }

  public var menuDriverLabel: String {
    let current = settingValue("menu_driver").isEmpty ? "materialui" : settingValue("menu_driver")
    return menuDriverChoices.first { $0.value == current }?.label ?? current
  }

  public func setMenuDriver(_ value: String) {
    setSetting(key: "menu_driver", value: value)
    frontend?.pushSkinSettingsMenu()
    refreshMenu()
    statusMessage = "Menu driver: \(menuDriverLabel)"
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
    case "vulkan": return "Vulkan"
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
    [("Metal", "metal"), ("Software", "software"), ("MoltenVK", "moltenvk"), ("OpenGL ES", "opengl"), ("Vulkan", "vulkan")]
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


  public func cycleStateSlot(delta: Int) {
    let current = Int(settingValue("state_slot")) ?? 0
    let next: Int
    if delta > 0 {
      next = current >= 999 ? 0 : current + 1
    } else {
      next = current <= 0 ? -1 : current - 1
    }
    setSetting(key: "state_slot", value: String(next))
    statusMessage = "State slot: \(stateSlotLabel)"
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
    @unknown default:
      return "Compatibility unknown for .\(plan.contentExtension)"
    }
  }

  func sortedGames(_ games: [GameEntrySwift]) -> [GameEntrySwift] {
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

  func cycleSetting(key: String, values: [String]) {
    let current = settingValue(key)
    let next: String
    if let index = values.firstIndex(of: current) {
      next = values[(index + 1) % values.count]
    } else {
      next = values.first ?? current
    }
    setSetting(key: key, value: next)
  }

  func scaleModeFromSetting(_ value: String?) -> GfxScaleMode {
    switch value {
    case "integer": return .integer
    case "stretch": return .stretch
    default: return .keepAspect
    }
  }

  func filterModeFromSetting(_ value: String?) -> GfxFilterMode {
    value == "linear" ? .linear : .nearest
  }

  func applyRendererSetting(_ frontend: Retrofront) {
    let value = frontend.setting("video_driver") ?? "metal"
    let backend: GfxBackend = switch value.lowercased() {
    case "software": .software
    case "opengl", "gl", "gles", "opengles": .openGL
    case "vulkan", "vk": .vulkan
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
}
