import Foundation
import RetrofrontSwift
import UIKit
import Combine
import UniformTypeIdentifiers

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
  @Published private(set) var settings: [RetrofrontSetting] = []
  @Published private(set) var pendingCoreChoices: [CoreInfo] = []
  @Published private(set) var pendingContentURL: URL?
  @Published var statusMessage = "Ready"

  private var frontend: Retrofront?
  private var runTask: Task<Void, Never>?
  private var pixelBuffer: Data?

  public init() {
    setupFrontend()
    refreshAvailableCores()
    refreshGames()
    refreshMenu()
  }

  private var retroArchRoot: URL {
    FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      .appendingPathComponent("RetroArch", isDirectory: true)
  }

  private func setupFrontend() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend

      let root = retroArchRoot
      try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
      let configPath = root.appendingPathComponent("config/retroarch.cfg").path
      try? frontend.setBaseDirectory(root.path)
      applyBundleCoreDirectories(frontend)
      try? frontend.loadSettings(at: configPath)
      applyBundleCoreDirectories(frontend)
      try? frontend.setSetting(key: "content_directory", value: root.path)
      try? frontend.setSetting(key: "core_assets_directory", value: root.appendingPathComponent("downloads").path)
      try? frontend.setGfxBackend(.bgfx)
      frontend.saveSettings()
      refresh()
    } catch {
      statusMessage = "Initialization failed: \(error)"
    }
  }

  private func applyBundleCoreDirectories(_ frontend: Retrofront) {
    if let frameworksURL = Bundle.main.privateFrameworksURL {
      try? frontend.setSetting(key: "libretro_directory", value: frameworksURL.path)
    }
    if let resourceURL = Bundle.main.resourceURL {
      let info = resourceURL.appendingPathComponent("info", isDirectory: true)
      if FileManager.default.fileExists(atPath: info.path) {
        frontend.setInfoDir(info.path)
        try? frontend.setSetting(key: "libretro_info_path", value: info.path)
      }
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
    frontend.scanConfiguredCores()
    availableCores = frontend.availableCores()
  }

  public func refreshGames() {
    guard let frontend else { return }
    let contentDir = URL(fileURLWithPath: frontend.setting("content_directory") ?? retroArchRoot.path)
    try? FileManager.default.createDirectory(at: contentDir, withIntermediateDirectories: true)
    let exts = frontend.allSupportedExtensions().joined(separator: "|")
    frontend.scanGames(in: contentDir.path, extensions: exts)
    availableGames = frontend.availableGames()
  }

  public func importFile(at url: URL) {
    importGame(at: url)
  }

  public func importGame(at url: URL) {
    guard let frontend else { return }
    let destinationDir = URL(fileURLWithPath: frontend.setting("core_assets_directory") ?? retroArchRoot.appendingPathComponent("downloads").path)
    try? FileManager.default.createDirectory(at: destinationDir, withIntermediateDirectories: true)
    let destination = destinationDir.appendingPathComponent(url.lastPathComponent)
    let success = url.startAccessingSecurityScopedResource()
    defer { if success { url.stopAccessingSecurityScopedResource() } }
    do {
      if FileManager.default.fileExists(atPath: destination.path) {
        try FileManager.default.removeItem(at: destination)
      }
      try FileManager.default.copyItem(at: url, to: destination)
      statusMessage = "Imported \(url.lastPathComponent) to downloads"
      refreshGames()
    } catch {
      statusMessage = "Import failed: \(error)"
    }
  }

  public func installBundledAssets() {
    guard let frontend else { return }
    guard let zipURL = Bundle.main.url(forResource: "assets", withExtension: "zip") else {
      statusMessage = "assets.zip was not found in the app bundle"
      return
    }
    let assetsDir = URL(fileURLWithPath: frontend.setting("assets_directory") ?? retroArchRoot.appendingPathComponent("assets").path)
    do {
      let report = try frontend.installAssetsZip(from: zipURL.path, to: assetsDir.path)
      refresh()
      statusMessage = "Installed assets: \(report.filesWritten) files"
    } catch {
      statusMessage = "Assets install failed: \(error)"
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
      statusMessage = "No compatible core found for .\(plan.contentExtension)"
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
      statusMessage = "Loaded game: \(url.lastPathComponent)"
    } catch {
      statusMessage = "Game load failed: \(error)"
    }
  }

  public func setJoypadButton(_ button: JoypadButton, pressed: Bool) {
    try? frontend?.setJoypadButton(button, pressed: pressed)
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
  }

  public func refreshMenu() { currentMenu = frontend?.currentMenuList() }

  public func menuAction(_ actionId: UInt32) {
    guard let frontend else { return }
    if frontend.activateMenuAction(actionId) { refreshMenu() }
  }

  public func menuPop() {
    if frontend?.menuPop() == true { refreshMenu() }
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
