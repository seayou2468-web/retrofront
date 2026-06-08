import Foundation
import RetrofrontSwift
import SwiftUI
import UIKit

struct CoreDescriptor: Identifiable, Equatable {
  let url: URL
  let isBundled: Bool

  var id: String { url.path }
  var displayName: String { url.deletingPathExtension().lastPathComponent }
  var locationDescription: String { isBundled ? "Bundled" : "Documents/Cores" }
}

@MainActor
final class EmulatorRuntimeModel: ObservableObject {
  @Published private(set) var frontendState: FrontendState = .empty
  @Published private(set) var systemInfo: LibretroSystemInfo?
  @Published private(set) var recentEvents: [FrontendEvent] = []
  @Published private(set) var latestFrame: VideoFrame?
  @Published private(set) var displayImage: UIImage?
  @Published private(set) var loadedGameURL: URL?
  @Published private(set) var coreURL: URL?
  @Published private(set) var availableCores: [CoreDescriptor] = []
  @Published private(set) var isRunning = false
  @Published var selectedTab: AppSection = .library
  @Published var statusMessage = "Loading emulator cores…"

  private var frontend: Retrofront?
  private var runTask: Task<Void, Never>?

  init() {
    refreshAvailableCores()
    do {
      let frontend = try Retrofront()
      self.frontend = frontend
      try? frontend.setGfxBackend(.software)
      do {
        try loadDefaultCore(using: frontend)
      } catch {
        statusMessage = "No core loaded: \(error)"
      }
      refresh()
    } catch {
      self.frontend = nil
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

  var loadedCoreName: String {
    systemInfo?.libraryName ?? coreURL?.lastPathComponent ?? "No core loaded"
  }

  func refresh() {
    refreshAvailableCores()
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
        try loadDefaultCore(using: frontend)
      }
      try frontend.loadGame(at: localURL.path)
      loadedGameURL = localURL
      selectedTab = .play
      refresh()
      statusMessage =
        "Loaded \(localURL.lastPathComponent) with \(loadedCoreName). Tap Play to run frames."
    } catch {
      statusMessage = "ROM load failed: \(error)"
    }
  }

  func importCore(from sourceURL: URL) {
    guard let frontend else { return }
    let securityScoped = sourceURL.startAccessingSecurityScopedResource()
    defer {
      if securityScoped { sourceURL.stopAccessingSecurityScopedResource() }
    }

    do {
      let localURL = try copyCoreIntoDocuments(from: sourceURL)
      try loadCore(at: localURL, using: frontend)
      refreshAvailableCores()
      selectedTab = .cores
      statusMessage = "Imported and loaded core: \(loadedCoreName)."
    } catch {
      statusMessage = "Core import failed: \(error)"
    }
  }

  func loadCore(_ core: CoreDescriptor) {
    guard let frontend else { return }
    do {
      try loadCore(at: core.url, using: frontend)
      selectedTab = .cores
      statusMessage = "Loaded core: \(loadedCoreName). Choose a ROM to start."
    } catch {
      statusMessage = "Core load failed: \(error)"
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

  private func loadDefaultCore(using frontend: Retrofront) throws {
    refreshAvailableCores()
    guard let core = availableCores.first else {
      throw RetrofrontError.operationFailed(
        "No libretro core dylibs were found in Frameworks, the app bundle, or Documents/Cores")
    }
    try loadCore(at: core.url, using: frontend)
    statusMessage = "Loaded core: \(loadedCoreName)."
  }

  private func loadCore(at url: URL, using frontend: Retrofront) throws {
    stop()
    systemInfo = try frontend.loadCore(at: url.path)
    coreURL = url
    loadedGameURL = nil
    latestFrame = nil
    displayImage = nil
    frontendState = frontend.state
  }

  private func refreshAvailableCores() {
    var cores: [CoreDescriptor] = []
    var seen = Set<String>()

    for url in bundledCoreURLs() {
      if seen.insert(url.path).inserted {
        cores.append(CoreDescriptor(url: url, isBundled: true))
      }
    }

    if let documentsCoreDirectory = try? coreDirectory(create: false) {
      for url in dylibURLs(in: documentsCoreDirectory) {
        if seen.insert(url.path).inserted {
          cores.append(CoreDescriptor(url: url, isBundled: false))
        }
      }
    }

    availableCores = cores.sorted {
      if $0.isBundled != $1.isBundled { return $0.isBundled && !$1.isBundled }
      return $0.displayName.localizedStandardCompare($1.displayName) == .orderedAscending
    }
  }

  private func bundledCoreURLs() -> [URL] {
    var directories: [URL] = []
    if let frameworksURL = Bundle.main.privateFrameworksURL {
      directories.append(frameworksURL)
    }
    if let resourceURL = Bundle.main.resourceURL {
      directories.append(resourceURL)
    }
    return directories.flatMap(dylibURLs)
  }

  private func dylibURLs(in directory: URL) -> [URL] {
    guard
      let urls = try? FileManager.default.contentsOfDirectory(
        at: directory,
        includingPropertiesForKeys: [.isRegularFileKey],
        options: [.skipsHiddenFiles]
      )
    else { return [] }
    return urls.filter { $0.pathExtension == "dylib" }
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

  private func copyCoreIntoDocuments(from sourceURL: URL) throws -> URL {
    guard sourceURL.pathExtension == "dylib" else {
      throw RetrofrontError.operationFailed("Core files must use the .dylib extension")
    }
    let destinationDirectory = try coreDirectory(create: true)
    let destination = destinationDirectory.appendingPathComponent(sourceURL.lastPathComponent)
    if FileManager.default.fileExists(atPath: destination.path) {
      try FileManager.default.removeItem(at: destination)
    }
    try FileManager.default.copyItem(at: sourceURL, to: destination)
    return destination
  }

  private func coreDirectory(create: Bool) throws -> URL {
    let documents = try FileManager.default.url(
      for: .documentDirectory,
      in: .userDomainMask,
      appropriateFor: nil,
      create: true)
    let directory = documents.appendingPathComponent("Cores", isDirectory: true)
    if create {
      try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }
    return directory
  }

  private static func image(from frame: VideoFrame) -> UIImage? {
    guard frame.width > 0, frame.height > 0, !frame.rgba.isEmpty else { return nil }
    let data = Data(frame.rgba)
    guard let provider = CGDataProvider(data: data as CFData) else { return nil }
    guard
      let cgImage = CGImage(
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
      )
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
    case .library: return "books.vertical.fill"
    case .play: return "gamecontroller.fill"
    case .cores: return "cpu.fill"
    case .settings: return "gearshape.fill"
    }
  }
}
