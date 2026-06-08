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
  @Published private(set) var isRunning = false
  @Published private(set) var availableCores: [CoreInfo] = []
  @Published private(set) var availableGames: [GameEntrySwift] = []
  @Published private(set) var corePath: String?
  @Published private(set) var loadedGameURL: URL?
  @Published private(set) var currentMenu: MenuList?
  @Published var statusMessage = "Ready"

  private var frontend: Retrofront?
  private var runTask: Task<Void, Never>?

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
      let configPath = docs.appendingPathComponent("retrofront-core-options.cfg").path
      try? frontend.setOptionsConfigPath(configPath)

      // Setup Info Dir
      if let resourceURL = Bundle.main.resourceURL {
          frontend.setInfoDir(resourceURL.appendingPathComponent("info").path)
      }

      try? frontend.setGfxBackend(.bgfx)
      refresh()
    } catch {
      statusMessage = "Initialization failed: \(error)"
    }
  }

  public func refreshAvailableCores() {
    guard let frontend else { return }

    // Scan Bundled cores
    if let frameworksURL = Bundle.main.privateFrameworksURL {
        frontend.scanCores(in: frameworksURL.path)
    }
    if let resourceURL = Bundle.main.resourceURL {
        frontend.scanCores(in: resourceURL.path)
    }

    // Scan Imported cores
    let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    let coresDir = docs.appendingPathComponent("Cores")
    try? FileManager.default.createDirectory(at: coresDir, withIntermediateDirectories: true)
    frontend.scanCores(in: coresDir.path)

    self.availableCores = frontend.availableCores()
  }

  public func refreshGames() {
      guard let frontend else { return }
      let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      let romsDir = docs.appendingPathComponent("Roms")
      try? FileManager.default.createDirectory(at: romsDir, withIntermediateDirectories: true)

      // We scan for all common extensions for now
      let exts = "gba|gb|gbc|sfc|smc|nes|bin|gen|md|sms|gg"
      frontend.scanGames(in: romsDir.path, extensions: exts)
      self.availableGames = frontend.availableGames()
  }

  public func importGame(at url: URL) {
      let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      let romsDir = docs.appendingPathComponent("Roms")
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

    // If no core is loaded, try to find one based on extension
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
    runTask = Task {
      while !Task.isCancelled {
        autoreleasepool {
            runOneFrame()
        }
        // Use a more accurate timer if possible, but sleep is okay for now
        try? await Task.sleep(nanoseconds: 16_666_667)
      }
    }
  }

  public func stop() {
    isRunning = false
    runTask?.cancel()
    runTask = nil
  }

  private func runOneFrame() {
    guard let frontend else { return }
    do {
      _ = try frontend.runFrame()
      if let frame = frontend.latestVideoFrame() {
        displayImage = Self.image(from: frame)
      }
    } catch {
      stop()
      statusMessage = "Run error: \(error)"
    }
  }

  public func refresh() {
    guard let frontend else { return }
    frontendState = frontend.state
    coreOptions = frontend.coreOptions()
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

  private static func image(from frame: VideoFrame) -> UIImage? {
    guard frame.width > 0, frame.height > 0, !frame.rgba.isEmpty else { return nil }
    let data = Data(frame.rgba)
    guard let provider = CGDataProvider(data: data as CFData) else { return nil }
    guard let cgImage = CGImage(
      width: Int(frame.width),
      height: Int(frame.height),
      bitsPerComponent: 8,
      bitsPerPixel: 32,
      bytesPerRow: Int(frame.width) * 4,
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
