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

  private func setupFrontend() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend

      let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      let configPath = docs.appendingPathComponent("retroarch.cfg").path
      try? frontend.setBaseDirectory(docs.path)
      try? frontend.loadSettings(at: configPath)

      if let resourceURL = Bundle.main.resourceURL {
          let bundledInfo = resourceURL.appendingPathComponent("info")
          if FileManager.default.fileExists(atPath: bundledInfo.path) {
              frontend.setInfoDir(bundledInfo.path)
          }
      }

      try? frontend.setGfxBackend(.bgfx)
      frontend.saveSettings()
      refresh()
    } catch {
      statusMessage = "Initialization failed: \(error)"
    }
  }

  public func refreshAvailableCores() {
    guard let frontend else { return }
    if let frameworksURL = Bundle.main.privateFrameworksURL {
        frontend.scanCores(in: frameworksURL.path)
    }
    if let resourceURL = Bundle.main.resourceURL {
        frontend.scanCores(in: resourceURL.path)
    }
    frontend.scanConfiguredCores()
    self.availableCores = frontend.availableCores()
  }

  public func refreshGames() {
      guard let frontend else { return }
      let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      let romsDir = URL(fileURLWithPath: frontend.setting("content_directory") ?? docs.appendingPathComponent("Roms").path)
      try? FileManager.default.createDirectory(at: romsDir, withIntermediateDirectories: true)
      let detectedExtensions = frontend.allSupportedExtensions()
      let exts = detectedExtensions.isEmpty ? "gba|gb|gbc|sfc|smc|nes|bin|gen|md|sms|gg" : detectedExtensions.joined(separator: "|")
      frontend.scanGames(in: romsDir.path, extensions: exts)
      self.availableGames = frontend.availableGames()
  }

  public func importFile(at url: URL) {
      if url.pathExtension.lowercased() == "dylib" {
          importCore(at: url)
      } else {
          importGame(at: url)
      }
  }

  public func importGame(at url: URL) {
      guard let frontend else { return }
      let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      let romsDir = URL(fileURLWithPath: frontend.setting("content_directory") ?? docs.appendingPathComponent("Roms").path)
      try? FileManager.default.createDirectory(at: romsDir, withIntermediateDirectories: true)
      let destination = romsDir.appendingPathComponent(url.lastPathComponent)
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

  public func importCore(at url: URL) {
      guard let frontend else { return }
      let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      let coresDir = URL(fileURLWithPath: frontend.setting("libretro_directory") ?? docs.appendingPathComponent("Cores").path)
      try? FileManager.default.createDirectory(at: coresDir, withIntermediateDirectories: true)
      let destination = coresDir.appendingPathComponent(url.lastPathComponent)
      let success = url.startAccessingSecurityScopedResource()
      defer { if success { url.stopAccessingSecurityScopedResource() } }
      do {
          if FileManager.default.fileExists(atPath: destination.path) {
              try FileManager.default.removeItem(at: destination)
          }
          try FileManager.default.copyItem(at: url, to: destination)
          try? frontend.setSetting(key: "libretro_directory", value: coresDir.path)
          frontend.saveSettings()
          refreshAvailableCores()
          statusMessage = "Imported core: \(url.lastPathComponent)"
      } catch {
          statusMessage = "Core import failed: \(error)"
      }
  }

  public func loadCore(_ core: CoreInfo) {
    guard let frontend else { return }
    do {
      stop()
      systemInfo = try frontend.loadCore(at: core.path)
      corePath = core.path
      refresh()
      statusMessage = "Loaded core: \(systemInfo?.libraryName ?? "Unknown")"
    } catch {
      statusMessage = "Core load failed: \(error)"
    }
  }

  public func loadGame(at url: URL) {
    guard let frontend else { return }
    if frontendState == .empty {
        let ext = url.pathExtension.lowercased()
        if let suitableCore = availableCores.first(where: { $0.supportedExtensions.contains(ext) }) {
            loadCore(suitableCore)
        } else {
            statusMessage = "No suitable core found for .\(ext)"
            return
        }
    }
    do {
      try frontend.loadGame(at: url.path)
      loadedGameURL = url
      refresh()
      statusMessage = "Loaded game: \(url.lastPathComponent)"
    } catch {
      statusMessage = "Game load failed: \(error)"
    }
  }

  public func setJoypadButton(_ button: JoypadButton, pressed: Bool) {
    try? frontend?.setJoypadButton(button, pressed: pressed)
  }

  public func toggleRunning() {
    isRunning ? stop() : play()
  }

  public func play() {
    guard frontendState == .gameLoaded, !isRunning else { return }
    isRunning = true
    runTask = Task.detached(priority: .userInitiated) { [weak self] in
      while !Task.isCancelled {
        guard let self = self else { break }
        let shouldStop: Bool = await autoreleasepool {
            return self.runOneFrame()
        }
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
        pixelBuffer?.withUnsafeMutableBytes { buffer in
          if let base = buffer.baseAddress {
            _ = frontend.copyLatestVideoFrame(to: base, length: buffer.count)
          }
        }
        if let data = pixelBuffer {
          let image = Self.imageFromData(data, width: Int(info.width), height: Int(info.height))
          Task { @MainActor in
              self.displayImage = image
          }
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
        if config.aspectRatio > 0 {
            aspectRatio = Double(config.aspectRatio)
        } else if config.baseHeight > 0 {
            aspectRatio = Double(config.baseWidth) / Double(config.baseHeight)
        }
    }
    refreshMenu()
  }

  public func refreshMenu() {
      currentMenu = frontend?.currentMenuList()
  }

  public func menuPop() {
      if frontend?.menuPop() == true {
          refreshMenu()
      }
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
      bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.last.rawValue),
      provider: provider,
      decode: nil,
      shouldInterpolate: false,
      intent: .defaultIntent
    ) else { return nil }
    return UIImage(cgImage: cgImage)
  }
}
