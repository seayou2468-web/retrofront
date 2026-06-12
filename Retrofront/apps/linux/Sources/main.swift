import Adwaita
import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif
#if os(Linux)
import Glibc
#endif
import RetrofrontSwift

enum FrontendAssetArchive: String, CaseIterable {
  case assets
  case info
  case overlays
  case gluiMinimalAssets = "glui_minimal_assets"

  var fileName: String { "\(rawValue).zip" }
  var downloadURL: URL { URL(string: "https://buildbot.libretro.com/assets/frontend/\(fileName)")! }
}

struct LinuxOverlayChoice: Equatable {
  let path: String
  let label: String
}

struct CommandLineOptions {
  var command: String?
  var positional: [String] = []
  var corePath: String?
  var romPath: String?
  var key: String?
  var value: String?
  var frames = 180
  var gui = false
  var help = false

  init(arguments: [String] = Array(CommandLine.arguments.dropFirst())) {
    var index = 0
    while index < arguments.count {
      let argument = arguments[index]
      switch argument {
      case "--help", "-h":
        help = true
      case "--gui":
        gui = true
      case "--core":
        index += 1
        if index < arguments.count { corePath = arguments[index] }
      case "--rom", "--content":
        index += 1
        if index < arguments.count { romPath = arguments[index] }
      case "--key":
        index += 1
        if index < arguments.count { key = arguments[index] }
      case "--value":
        index += 1
        if index < arguments.count { value = arguments[index] }
      case "--frames":
        index += 1
        if index < arguments.count { frames = Int(arguments[index]) ?? frames }
      default:
        if command == nil, !argument.hasPrefix("-") {
          command = argument
        } else {
          positional.append(argument)
        }
      }
      index += 1
    }
    if command == nil, romPath != nil { command = "launch" }
  }
}

@MainActor
final class LinuxRetrofrontRuntime {
  let frontend: Retrofront
  let layout: RetroArchStorageLayout

  var statusMessage = "Ready"
  private(set) var loadedCore: CoreInfo?
  private(set) var loadedGamePath: String?
  private(set) var loadedGameLabel: String?
  private var runTask: Task<Void, Never>?

  init() throws {
    frontend = try Retrofront()
    layout = .current
    try layout.createWritableDirectories()
    try frontend.setBaseDirectory(layout.root.path)
    try? frontend.loadSettings(at: layout.configFile.path)
    applyRetroArchLayout()
    frontend.setInfoDir(layout.infoDirectory.path)
    refreshLibrary()
    frontend.saveSettings()
  }

  deinit {
    runTask?.cancel()
    try? frontend.saveSRAM()
  }

  private func applyRetroArchLayout() {
    for setting in layout.retroArchSettings {
      try? frontend.setSetting(key: setting.key, value: setting.url.path)
    }
  }

  func refreshLibrary() {
    scanCores()
    scanGames()
    refreshOverlayChoices()
  }

  func scanCores() {
    frontend.scanCores(in: layout.root.appendingPathComponent("cores", isDirectory: true).path)
    frontend.scanCores(in: layout.root.appendingPathComponent("Cores", isDirectory: true).path)
    frontend.scanCores(in: URL(fileURLWithPath: FileManager.default.currentDirectoryPath).appendingPathComponent("cores", isDirectory: true).path)
    frontend.scanConfiguredCores()
  }

  func scanGames() {
    let contentDirectory = URL(fileURLWithPath: settingValue("content_directory", fallback: layout.contentDirectory.path))
    try? FileManager.default.createDirectory(at: contentDirectory, withIntermediateDirectories: true)
    frontend.scanGames(in: contentDirectory.path, extensions: frontend.allSupportedExtensions().joined(separator: "|"))
  }

  var cores: [CoreInfo] {
    frontend.availableCores().sorted { $0.displayName.localizedCaseInsensitiveCompare($1.displayName) == .orderedAscending }
  }

  var games: [GameEntrySwift] {
    switch settingValue("library_sort_mode", fallback: "name_ascending") {
    case "extension":
      return frontend.availableGames().sorted {
        let leftExtension = URL(fileURLWithPath: $0.path).pathExtension.lowercased()
        let rightExtension = URL(fileURLWithPath: $1.path).pathExtension.lowercased()
        return leftExtension == rightExtension
          ? $0.label.localizedCaseInsensitiveCompare($1.label) == .orderedAscending
          : leftExtension < rightExtension
      }
    case "name_descending":
      return frontend.availableGames().sorted { $0.label.localizedCaseInsensitiveCompare($1.label) == .orderedDescending }
    default:
      return frontend.availableGames().sorted { $0.label.localizedCaseInsensitiveCompare($1.label) == .orderedAscending }
    }
  }

  func settingValue(_ key: String, fallback: String = "") -> String {
    frontend.setting(key) ?? fallback
  }

  func setSetting(key: String, value: String) throws {
    try frontend.setSetting(key: key, value: value)
    frontend.saveSettings()
    statusMessage = "Saved setting: \(key)=\(value)"
  }

  func importGame(at path: String) throws -> URL {
    let source = URL(fileURLWithPath: path).standardizedFileURL
    let contentDirectory = URL(fileURLWithPath: settingValue("content_directory", fallback: layout.contentDirectory.path))
    try FileManager.default.createDirectory(at: contentDirectory, withIntermediateDirectories: true)
    let destination = contentDirectory.appendingPathComponent(source.lastPathComponent)
    if FileManager.default.fileExists(atPath: destination.path) {
      try FileManager.default.removeItem(at: destination)
    }
    try FileManager.default.copyItem(at: source, to: destination)
    scanGames()
    statusMessage = "Imported \(source.lastPathComponent)"
    return destination
  }

  @discardableResult
  func loadCore(_ core: CoreInfo) throws -> LibretroSystemInfo {
    stop()
    let info = try frontend.loadCore(at: core.path)
    loadedCore = core
    statusMessage = "Loaded core: \(info.libraryName)"
    return info
  }

  func launchGame(at path: String, preferredCore: String? = nil) throws {
    stop()
    let absolutePath = URL(fileURLWithPath: path).standardizedFileURL.path
    guard FileManager.default.fileExists(atPath: absolutePath) else {
      throw RuntimeError.message("ROM not found: \(absolutePath)")
    }

    scanCores()
    guard let plan = frontend.planContentLaunch(path: absolutePath, preferredCore: preferredCore) else {
      throw RuntimeError.message("Could not create a launch plan for \(absolutePath)")
    }

    switch plan.decision {
    case .selected:
      try frontend.launchContent(at: absolutePath, preferredCore: plan.selectedCorePath)
    case .needsCoreChoice:
      let selectedCore = preferredCore ?? frontend.launchCandidates().first?.path
      guard let selectedCore else {
        throw RuntimeError.message("Multiple compatible cores are available; pass --core <path> to select one.")
      }
      try frontend.launchContent(at: absolutePath, preferredCore: selectedCore)
    case .noCore:
      throw RuntimeError.message(plan.reason.isEmpty ? "No compatible core for \(absolutePath)" : plan.reason)
    @unknown default:
      throw RuntimeError.message("Unsupported launch decision for \(absolutePath)")
    }

    let label = URL(fileURLWithPath: absolutePath).deletingPathExtension().lastPathComponent
    loadedGamePath = absolutePath
    loadedGameLabel = label
    loadedCore = cores.first { $0.path == preferredCore || $0.path == plan.selectedCorePath } ?? loadedCore
    statusMessage = "Launched \(label)"
  }

  func runFrames(_ count: Int) throws {
    guard frontend.state == .gameLoaded else { throw RuntimeError.message("No game is loaded") }
    for _ in 0..<max(1, count) { _ = try frontend.runFrame() }
    statusMessage = "Ran \(max(1, count)) frame(s)"
  }

  @MainActor
  func play() {
    guard frontend.state == .gameLoaded, runTask == nil else { return }
    statusMessage = "Running"
    runTask = Task { [weak self] in
      while !Task.isCancelled {
        do {
          _ = try self?.frontend.runFrame()
          try? await Task.sleep(nanoseconds: 16_666_667)
        } catch {
          self?.statusMessage = "Run error: \(error)"
          break
        }
      }
    }
  }

  @MainActor
  func stop() {
    runTask?.cancel()
    runTask = nil
    try? frontend.saveSRAM()
    statusMessage = "Stopped"
  }

  func reset() throws {
    try frontend.reset()
    statusMessage = "Reset content"
  }

  func saveState(slot: UInt32) throws {
    try frontend.saveState(slot: slot)
    statusMessage = "Saved state slot \(slot)"
  }

  func loadState(slot: UInt32) throws {
    try frontend.loadState(slot: slot)
    statusMessage = "Loaded state slot \(slot)"
  }

  func refreshOverlayChoices() {
    let overlayDirectory = URL(fileURLWithPath: settingValue("overlay_directory", fallback: layout.overlaysDirectory.path))
    guard let enumerator = FileManager.default.enumerator(at: overlayDirectory, includingPropertiesForKeys: nil) else { return }
    for case let url as URL in enumerator where url.pathExtension.lowercased() == "cfg" {
      if settingValue("input_overlay").isEmpty {
        try? frontend.setSetting(key: "input_overlay", value: url.path)
      }
    }
  }

  var overlays: [LinuxOverlayChoice] {
    let overlayDirectory = URL(fileURLWithPath: settingValue("overlay_directory", fallback: layout.overlaysDirectory.path))
    guard let enumerator = FileManager.default.enumerator(at: overlayDirectory, includingPropertiesForKeys: nil) else { return [] }
    var choices: [LinuxOverlayChoice] = []
    for case let url as URL in enumerator where url.pathExtension.lowercased() == "cfg" {
      let relative = url.path.replacingOccurrences(of: overlayDirectory.path + "/", with: "")
      choices.append(LinuxOverlayChoice(path: url.path, label: relative.replacingOccurrences(of: ".cfg", with: "")))
    }
    return choices.sorted { $0.label.localizedStandardCompare($1.label) == .orderedAscending }
  }

  @discardableResult
  func installBundledAsset(_ archive: FrontendAssetArchive) throws -> AssetInstallReport {
    guard let zipURL = Bundle.main.url(forResource: archive.rawValue, withExtension: "zip") else {
      throw RuntimeError.message("\(archive.fileName) was not found in the bundled resources")
    }
    return try installArchive(archive, zipURL: zipURL, sourceLabel: "bundled")
  }

  @discardableResult
  func downloadAndInstallAsset(_ archive: FrontendAssetArchive) throws -> AssetInstallReport {
    let data = try Data(contentsOf: archive.downloadURL)
    let cacheURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + "-" + archive.fileName)
    try data.write(to: cacheURL, options: .atomic)
    defer { try? FileManager.default.removeItem(at: cacheURL) }
    return try installArchive(archive, zipURL: cacheURL, sourceLabel: "downloaded")
  }

  @discardableResult
  func installArchive(_ archive: FrontendAssetArchive, zipURL: URL, sourceLabel: String) throws -> AssetInstallReport {
    let installRoot = installDestination(for: archive)
    try FileManager.default.createDirectory(at: installRoot, withIntermediateDirectories: true)
    let report = try frontend.installAssetsZip(from: zipURL.path, to: installRoot.path)
    applyRetroArchLayout()
    refreshLibrary()
    if let selected = overlays.first { try? selectOverlay(selected) }
    statusMessage = "Installed \(sourceLabel) \(archive.fileName): \(report.filesWritten) files"
    return report
  }

  func installDestination(for archive: FrontendAssetArchive) -> URL {
    // Libretro buildbot frontend zips contain the contents of each RetroArch
    // directory. Extract each archive directly into the matching configured
    // directory so ozone/materialui/xmb/rgui assets and overlays resolve from
    // the same paths RetroArch writes to retroarch.cfg.
    switch archive {
    case .assets, .gluiMinimalAssets: return layout.assetsDirectory
    case .info: return layout.infoDirectory
    case .overlays: return layout.overlaysDirectory
    }
  }

  func selectOverlay(_ overlay: LinuxOverlayChoice) throws {
    try frontend.setSetting(key: "input_overlay", value: overlay.path)
    try frontend.loadOverlay(at: overlay.path)
    frontend.setOverlayEnabled(true)
    frontend.saveSettings()
    statusMessage = "Overlay: \(overlay.label)"
  }

  func setOverlayEnabled(_ enabled: Bool) {
    try? frontend.setSetting(key: "input_overlay_enable", value: enabled ? "true" : "false")
    frontend.setOverlayEnabled(enabled)
    frontend.saveSettings()
    statusMessage = enabled ? "Overlay enabled" : "Overlay disabled"
  }

  var summaryText: String {
    """
    State: \(frontend.state)
    Status: \(statusMessage)
    RetroArch root: \(layout.root.path)
    Config: \(layout.configFile.path)
    Cores: \(cores.count)
    ROMs: \(games.count)
    Current core: \(loadedCore?.displayName ?? "None")
    Current game: \(loadedGameLabel ?? "None")
    Overlay: \(URL(fileURLWithPath: settingValue("input_overlay")).lastPathComponent)
    """
  }
}

enum RuntimeError: Error, CustomStringConvertible {
  case message(String)

  var description: String {
    switch self {
    case let .message(message): return message
    }
  }
}

@MainActor
final class AdwaitaRetrofrontApp {
  private let runtime: LinuxRetrofrontRuntime
  private var window: ApplicationWindow?
  private var content = Box(orientation: .vertical, spacing: 16)
  private var statusLabel = Label("Ready")

  init(runtime: LinuxRetrofrontRuntime) {
    self.runtime = runtime
  }

  func run() {
    let app = Application(id: "com.retrofront.linux")
    app.onActivate { [weak self, runtime] in
      guard let self else { return }
      let window = ApplicationWindow(application: app)
      self.window = window
      window.title = "Retrofront"
      window.defaultWidth = 1180
      window.defaultHeight = 820

      let toolbar = ToolbarView()
      let header = HeaderBar(title: "Retrofront", subtitle: "Linux emulator frontend")
      toolbar.addTopBar(header)

      let scroller = ScrolledWindow()
      content = Box(orientation: .vertical, spacing: 16)
      content.setMargins(18)
      scroller.child = content
      scroller.hexpand()
      scroller.vexpand()
      toolbar.setContent(scroller)
      window.setContent(toolbar)
      render()
      runtime.refreshLibrary()
      window.present()
    }
    app.run()
  }

  private func render() {
    for child in content.children() { content.remove(child) }
    statusLabel = Label(runtime.summaryText)
    statusLabel.xalign = 0
    statusLabel.selectable = true
    statusLabel.addCSSClass(.caption)
    content.append(statusLabel)
    content.append(actionBar())
    content.append(libraryGroup())
    content.append(coreGroup())
    content.append(playbackGroup())
    content.append(settingsGroup())
    content.append(overlayGroup())
    content.append(menuGroup())
  }

  private func actionBar() -> Widget {
    let box = Box(orientation: .horizontal, spacing: 8)
    box.append(Button(label: "Refresh") { [weak self] in self?.perform { $0.refreshLibrary() } })
    box.append(Button(label: "Import ROM…") { [weak self] in self?.openROMImporter() })
    box.append(Button(label: "Launch First ROM") { [weak self] in self?.perform { runtime in
      guard let game = runtime.games.first else { throw RuntimeError.message("No ROMs available") }
      try runtime.launchGame(at: game.path)
    } })
    box.append(Button(label: "Load First Core") { [weak self] in self?.perform { runtime in
      guard let core = runtime.cores.first else { throw RuntimeError.message("No cores available") }
      _ = try runtime.loadCore(core)
    } })
    return box
  }

  private func libraryGroup() -> Widget {
    let group = PreferencesGroup(title: "Library", description: "Imported ROMs and direct launch actions")
    for game in runtime.games.prefix(40) {
      let row = ActionRow(title: game.label, subtitle: game.path)
      let launch = Button(label: "Launch") { [weak self] in self?.perform { try $0.launchGame(at: game.path) } }
      row.addSuffix(launch)
      row.activatableWidget = launch
      group.add(row)
    }
    if runtime.games.isEmpty { group.add(ActionRow(title: "No ROMs", subtitle: "Use Import ROM or CLI import to add content.")) }
    return group
  }

  private func coreGroup() -> Widget {
    let group = PreferencesGroup(title: "Cores", description: "Discovered libretro cores")
    for core in runtime.cores.prefix(40) {
      let row = ActionRow(title: core.displayName, subtitle: core.path)
      let load = Button(label: "Load") { [weak self] in self?.perform { _ = try $0.loadCore(core) } }
      row.addSuffix(load)
      row.activatableWidget = load
      group.add(row)
    }
    if runtime.cores.isEmpty { group.add(ActionRow(title: "No cores", subtitle: "Install cores in \(runtime.layout.root.appendingPathComponent("cores").path).")) }
    return group
  }

  private func playbackGroup() -> Widget {
    let group = PreferencesGroup(title: "Playback", description: "Run loop, reset, SRAM and save states")
    let row = ActionRow(title: "Runtime Controls", subtitle: "Play, stop, reset, save/load slot 0")
    row.addSuffix(Button(label: "Play") { [weak self] in self?.perform { $0.play() } })
    row.addSuffix(Button(label: "Stop") { [weak self] in self?.perform { $0.stop() } })
    row.addSuffix(Button(label: "Reset") { [weak self] in self?.perform { try $0.reset() } })
    row.addSuffix(Button(label: "Save 0") { [weak self] in self?.perform { try $0.saveState(slot: 0) } })
    row.addSuffix(Button(label: "Load 0") { [weak self] in self?.perform { try $0.loadState(slot: 0) } })
    group.add(row)
    return group
  }

  private func settingsGroup() -> Widget {
    let group = PreferencesGroup(title: "Settings", description: "Common video, audio and library settings")
    addToggle(group, title: "Audio", key: "audio_enable", defaultEnabled: true)
    addToggle(group, title: "Audio Sync", key: "audio_sync", defaultEnabled: true)
    addToggle(group, title: "VSync", key: "video_vsync", defaultEnabled: true)
    addToggle(group, title: "Library Core Badges", key: "library_show_core_badges", defaultEnabled: true)
    addChoice(group, title: "Video Scale", key: "video_scale_mode", values: [("Aspect", "keep_aspect"), ("Integer", "integer"), ("Stretch", "stretch")])
    addChoice(group, title: "Video Filter", key: "video_filter_mode", values: [("Nearest", "nearest"), ("Linear", "linear")])
    addChoice(group, title: "Audio Latency", key: "audio_latency_ms", values: [("32 ms", "32"), ("64 ms", "64"), ("96 ms", "96"), ("128 ms", "128")])
    addChoice(group, title: "Library Sort", key: "library_sort_mode", values: [("Name ↑", "name_ascending"), ("Name ↓", "name_descending"), ("Extension", "extension")])
    addChoice(group, title: "Menu Driver", key: "menu_driver", values: [("Material UI", "materialui"), ("Ozone", "ozone"), ("XMB", "xmb"), ("RGUI", "rgui")])

    let assetsRow = ActionRow(title: "Frontend Assets", subtitle: "Install/download assets.zip, info.zip, and overlays.zip individually")
    for archive in FrontendAssetArchive.allCases {
      assetsRow.addSuffix(Button(label: "Fetch \(archive.rawValue)") { [weak self] in
        self?.perform { runtime in _ = try runtime.downloadAndInstallAsset(archive) }
      })
      assetsRow.addSuffix(Button(label: "Bundled \(archive.rawValue)") { [weak self] in
        self?.perform { runtime in _ = try runtime.installBundledAsset(archive) }
      })
    }
    group.add(assetsRow)
    return group
  }

  private func overlayGroup() -> Widget {
    let group = PreferencesGroup(title: "Overlays", description: "Touch overlay selection and current render metadata")
    let enableRow = ActionRow(title: "Touch Overlay", subtitle: runtime.settingValue("input_overlay_enable", fallback: "true"))
    let toggle = Switch(active: runtime.settingValue("input_overlay_enable", fallback: "true") != "false")
    toggle.onActiveChanged { [weak self, weak toggle] in
      guard let self, let toggle else { return }
      self.runtime.setOverlayEnabled(toggle.active)
      self.render()
    }
    enableRow.addSuffix(toggle)
    enableRow.activatableWidget = toggle
    group.add(enableRow)

    for overlay in runtime.overlays.prefix(30) {
      let row = ActionRow(title: overlay.label, subtitle: overlay.path)
      let button = Button(label: "Use") { [weak self] in self?.perform { try $0.selectOverlay(overlay) } }
      row.addSuffix(button)
      row.activatableWidget = button
      group.add(row)
    }

    let info = runtime.frontend.overlayInfo()
    group.add(ActionRow(title: "Active Overlay", subtitle: info?.activeName ?? "None"))
    return group
  }

  private func menuGroup() -> Widget {
    let group = PreferencesGroup(title: "Runtime Menu", description: "Core/content/settings menu actions exposed by the shared runtime")
    if let menu = runtime.frontend.currentMenuList() {
      group.add(ActionRow(title: menu.title, subtitle: "\(menu.entries.count) entries"))
      for entry in menu.entries.prefix(30) {
        let row = ActionRow(title: entry.label, subtitle: entry.sublabel.isEmpty ? entry.value : entry.sublabel)
        if entry.actionId != 0 {
          let button = Button(label: "Run") { [weak self] in self?.perform { runtime in
            _ = runtime.frontend.activateMenuAction(entry.actionId)
          } }
          row.addSuffix(button)
          row.activatableWidget = button
        }
        group.add(row)
      }
    } else {
      group.add(ActionRow(title: "No menu", subtitle: "Load a core or content to populate runtime actions."))
    }
    return group
  }

  private func addToggle(_ group: PreferencesGroup, title: String, key: String, defaultEnabled: Bool) {
    let enabled = runtime.settingValue(key, fallback: defaultEnabled ? "true" : "false") != "false"
    let row = ActionRow(title: title, subtitle: key)
    let toggle = Switch(active: enabled)
    toggle.onActiveChanged { [weak self, weak toggle] in
      guard let toggle else { return }
      self?.perform { runtime in try runtime.setSetting(key: key, value: toggle.active ? "true" : "false") }
    }
    row.addSuffix(toggle)
    row.activatableWidget = toggle
    group.add(row)
  }

  private func addChoice(_ group: PreferencesGroup, title: String, key: String, values: [(String, String)]) {
    let current = runtime.settingValue(key)
    let row = ActionRow(title: title, subtitle: current.isEmpty ? key : "\(key)=\(current)")
    for choice in values {
      row.addSuffix(Button(label: choice.0) { [weak self] in
        self?.perform { runtime in try runtime.setSetting(key: key, value: choice.1) }
      })
    }
    group.add(row)
  }

  private func openROMImporter() {
    let dialog = FileDialog()
    dialog.title = "Import ROM"
    dialog.acceptLabel = "Import"
    dialog.open(parent: window) { [weak self] path in
      guard let path else { return }
      self?.perform { _ = try $0.importGame(at: path) }
    }
  }

  private func perform(_ action: (LinuxRetrofrontRuntime) throws -> Void) {
    do {
      try action(runtime)
    } catch {
      runtime.statusMessage = "Error: \(error)"
    }
    render()
  }
}

func printUsage() {
  print("""
  Usage:
    retrofront-linux [--gui]
    retrofront-linux scan
    retrofront-linux list-cores
    retrofront-linux list-games
    retrofront-linux import <rom-path>
    retrofront-linux launch --rom <rom-path> [--core <core-path>] [--frames <count>]
    retrofront-linux load-core --core <core-path>
    retrofront-linux settings
    retrofront-linux set --key <setting> --value <value>
    retrofront-linux fetch-assets <assets|info|overlays|glui_minimal_assets|all>

  Examples:
    retrofront-linux import ~/ROMs/game.gba
    retrofront-linux launch --rom ~/ROMs/game.gba --frames 600
    retrofront-linux set --key video_filter_mode --value nearest
    retrofront-linux fetch-assets overlays
  """)
}

@MainActor
func runCommand(_ options: CommandLineOptions, runtime: LinuxRetrofrontRuntime) throws {
  switch options.command {
  case nil:
    print(runtime.summaryText)
  case "scan":
    runtime.refreshLibrary()
    print(runtime.summaryText)
  case "list-cores":
    for core in runtime.cores { print("\(core.displayName)\t\(core.path)") }
  case "list-games":
    for game in runtime.games { print("\(game.label)\t\(game.path)") }
  case "import":
    let path = options.romPath ?? options.positional.first
    guard let path else { throw RuntimeError.message("import requires a ROM path") }
    print(try runtime.importGame(at: path).path)
  case "launch":
    let path = options.romPath ?? options.positional.first
    guard let path else { throw RuntimeError.message("launch requires --rom <path>") }
    try runtime.launchGame(at: path, preferredCore: options.corePath)
    try runtime.runFrames(options.frames)
    print(runtime.summaryText)
  case "load-core":
    guard let corePath = options.corePath ?? options.positional.first else { throw RuntimeError.message("load-core requires --core <path>") }
    let info: LibretroSystemInfo
    if let core = runtime.cores.first(where: { $0.path == corePath }) {
      info = try runtime.loadCore(core)
    } else {
      runtime.stop()
      info = try runtime.frontend.loadCore(at: corePath)
      runtime.statusMessage = "Loaded core: \(info.libraryName)"
    }
    print("Loaded \(info.libraryName) \(info.libraryVersion)")
  case "settings":
    for setting in runtime.frontend.settings().sorted(by: { $0.key < $1.key }) { print("\(setting.key)=\(setting.value)") }
  case "set":
    guard let key = options.key ?? options.positional.first else { throw RuntimeError.message("set requires --key <setting>") }
    let value = options.value ?? options.positional.dropFirst().first
    guard let value else { throw RuntimeError.message("set requires --value <value>") }
    try runtime.setSetting(key: key, value: value)
    print(runtime.statusMessage)
  case "fetch-assets":
    let requested = (options.positional.first ?? "all").lowercased()
    let archives: [FrontendAssetArchive]
    if requested == "all" {
      archives = FrontendAssetArchive.allCases
    } else if let archive = FrontendAssetArchive(rawValue: requested) {
      archives = [archive]
    } else {
      throw RuntimeError.message("fetch-assets requires assets, info, overlays, glui_minimal_assets, or all")
    }
    for archive in archives {
      let report = try runtime.downloadAndInstallAsset(archive)
      print("Installed \(archive.fileName): \(report.filesWritten) files")
    }
  default:
    throw RuntimeError.message("Unknown command: \(options.command ?? "")")
  }
}

@MainActor
func main() {
  let options = CommandLineOptions()
  if options.help {
    printUsage()
    return
  }

  do {
    let runtime = try LinuxRetrofrontRuntime()
    if options.gui {
      #if os(Linux)
      setenv("GSK_RENDERER", ProcessInfo.processInfo.environment["GSK_RENDERER"] ?? "cairo", 0)
      if (ProcessInfo.processInfo.environment["WAYLAND_DISPLAY"] ?? "").isEmpty
        && (ProcessInfo.processInfo.environment["DISPLAY"] ?? "").isEmpty {
        throw RuntimeError.message("--gui requires WAYLAND_DISPLAY or DISPLAY; refusing to initialize GTK without a display")
      }
      #endif
      let gui = AdwaitaRetrofrontApp(runtime: runtime)
      gui.run()
    } else {
      try runCommand(options, runtime: runtime)
    }
  } catch {
    FileHandle.standardError.write(Data("retrofront-linux: \(error)\n".utf8))
    exit(1)
  }
}

main()
