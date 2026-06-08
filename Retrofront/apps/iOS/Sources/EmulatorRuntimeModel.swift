import Foundation
import RetrofrontSwift
import SwiftUI
import UIKit

@MainActor
final class EmulatorRuntimeModel: ObservableObject {
  @Published private(set) var frontendState: FrontendState = .empty
  @Published private(set) var systemInfo: LibretroSystemInfo?
  @Published private(set) var recentEvents: [FrontendEvent] = []
  @Published private(set) var latestFrame: VideoFrame?
  @Published private(set) var displayImage: UIImage?
  @Published private(set) var loadedGameURL: URL?
  @Published private(set) var coreURL: URL?
  @Published private(set) var isRunning = false
  @Published var selectedTab: AppSection = .library
  @Published var statusMessage = "Loading bundled emulator core…"

  private let frontend: Retrofront?
  private var runTask: Task<Void, Never>?

  init() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend
      try? frontend.setGfxBackend(.software)
      try loadBundledCore(using: frontend)
      refresh()
    } catch {
      frontend = nil
      statusMessage = "Runtime unavailable: \(error)"
    }
  }

  deinit {
    runTask?.cancel()
  }

  var isRuntimeConnected: Bool {
    frontend != nil
  }

  var canRunGame: Bool {
    frontendState == .gameLoaded
  }

  var loadedGameName: String {
    loadedGameURL?.lastPathComponent ?? "No ROM selected"
  }

  func refresh() {
    guard let frontend else { return }
    frontendState = frontend.state
    recentEvents.append(contentsOf: frontend.drainEvents())
    recentEvents = Array(recentEvents.suffix(12))
    if let frame = frontend.latestVideoFrame() {
      latestFrame = frame
      displayImage = Self.image(from: frame)
    }
    if let info = try? frontend.systemInfo() {
      systemInfo = info
    }
  }

  func importROM(from sourceURL: URL) {
    guard let frontend else { return }
    let securityScoped = sourceURL.startAccessingSecurityScopedResource()
    defer {
      if securityScoped { sourceURL.stopAccessingSecurityScopedResource() }
    }

    do {
      let localURL = try copyROMIntoDocuments(from: sourceURL)
      if frontend.state == .empty {
        try loadBundledCore(using: frontend)
      }
      try frontend.loadGame(at: localURL.path)
      loadedGameURL = localURL
      selectedTab = .play
      refresh()
      statusMessage = "Loaded \(localURL.lastPathComponent). Tap Play to run frames."
    } catch {
      statusMessage = "ROM load failed: \(error)"
    }
  }

  func setButton(_ button: JoypadButton, pressed: Bool) {
    guard let frontend else { return }
    try? frontend.setJoypadButton(button, pressed: pressed)
  }

  func toggleRunning() {
    isRunning ? stop() : play()
  }

  func play() {
    guard canRunGame, runTask == nil else { return }
    isRunning = true
    statusMessage = "Running \(loadedGameName)."
    runTask = Task { [weak self] in
      while !Task.isCancelled {
        await self?.runOneFrameFromLoop()
        try? await Task.sleep(nanoseconds: 16_666_667)
      }
    }
  }

  func stop() {
    runTask?.cancel()
    runTask = nil
    isRunning = false
    statusMessage = canRunGame ? "Paused \(loadedGameName)." : "Ready."
  }

  func runOneFrameFromButton() {
    runOneFrame()
  }

  private func runOneFrameFromLoop() {
    runOneFrame()
  }

  private func runOneFrame() {
    guard let frontend else { return }
    do {
      recentEvents.append(contentsOf: try frontend.runFrame())
      recentEvents = Array(recentEvents.suffix(12))
      if let frame = frontend.latestVideoFrame() {
        latestFrame = frame
        displayImage = Self.image(from: frame)
        statusMessage = "Displayed frame #\(frame.frameNumber) from \(loadedGameName)."
      } else {
        statusMessage = "Core ran; waiting for video output."
      }
      frontendState = frontend.state
    } catch {
      stop()
      statusMessage = "Run failed: \(error)"
    }
  }

  private func loadBundledCore(using frontend: Retrofront) throws {
    let core = [
      Bundle.main.privateFrameworksURL?.appendingPathComponent("mgba_libretro_ios.dylib"),
      Bundle.main.url(forResource: "mgba_libretro_ios", withExtension: "dylib")
    ]
    .compactMap { $0 }
    .first { FileManager.default.fileExists(atPath: $0.path) }
    guard let core else {
      throw RetrofrontError.operationFailed("Bundled core mgba_libretro_ios.dylib was not found in Frameworks or the app bundle")
    }
    coreURL = core
    systemInfo = try frontend.loadCore(at: core.path)
    frontendState = frontend.state
    statusMessage = "Loaded bundled core: \(systemInfo?.libraryName ?? core.lastPathComponent)."
  }

  private func copyROMIntoDocuments(from sourceURL: URL) throws -> URL {
    let documents = try FileManager.default.url(
      for: .documentDirectory,
      in: .userDomainMask,
      appropriateFor: nil,
      create: true)
    let romDirectory = documents.appendingPathComponent("ROMs", isDirectory: true)
    try FileManager.default.createDirectory(at: romDirectory, withIntermediateDirectories: true)
    let destination = romDirectory.appendingPathComponent(sourceURL.lastPathComponent)
    if FileManager.default.fileExists(atPath: destination.path) {
      try FileManager.default.removeItem(at: destination)
    }
    try FileManager.default.copyItem(at: sourceURL, to: destination)
    return destination
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
      intent: .defaultIntent)
    else { return nil }
    return UIImage(cgImage: cgImage)
  }
}

enum AppSection: String, CaseIterable, Identifiable {
  case library = "Library"
  case play = "Play"
  case cores = "Cores"
  case settings = "Settings"

  var id: String { rawValue }

  var symbolName: String {
    switch self {
    case .library:
      return "square.grid.2x2.fill"
    case .play:
      return "play.rectangle.fill"
    case .cores:
      return "cpu.fill"
    case .settings:
      return "gearshape.fill"
    }
  }
}
