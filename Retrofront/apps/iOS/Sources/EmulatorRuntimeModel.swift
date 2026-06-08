import Foundation
import RetrofrontSwift
import UIKit
import Combine
import UniformTypeIdentifiers

public struct CoreDescriptor: Identifiable, Hashable {
  public let id = UUID()
  public let url: URL
  public let isBundled: Bool

  public var displayName: String {
    url.deletingPathExtension().lastPathComponent
      .replacingOccurrences(of: "_libretro_ios", with: "")
      .replacingOccurrences(of: "_libretro", with: "")
  }

  public var locationDescription: String {
    isBundled ? "Bundled" : "Imported"
  }
}

@MainActor
public final class EmulatorRuntimeModel: ObservableObject {
  @Published private(set) var frontendState: FrontendState = .empty
  @Published private(set) var systemInfo: LibretroSystemInfo?
  @Published private(set) var coreOptions: [CoreOption] = []
  @Published private(set) var displayImage: UIImage?
  @Published private(set) var isRunning = false
  @Published private(set) var availableCores: [CoreDescriptor] = []
  @Published private(set) var coreURL: URL?
  @Published private(set) var loadedGameURL: URL?
  @Published var statusMessage = "Ready"

  private var frontend: Retrofront?
  private var runTask: Task<Void, Never>?

  public init() {
    setupFrontend()
    refreshAvailableCores()
  }

  private func setupFrontend() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend

      let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      let configPath = docs.appendingPathComponent("retrofront-core-options.cfg").path
      try? frontend.setOptionsConfigPath(configPath)

      try? frontend.setGfxBackend(.bgfx)
      refresh()
    } catch {
      statusMessage = "Initialization failed: \(error)"
    }
  }

  public func refreshAvailableCores() {
    var cores: [CoreDescriptor] = []

    // Bundled cores
    if let frameworksURL = Bundle.main.privateFrameworksURL {
        cores.append(contentsOf: dylibURLs(in: frameworksURL).map { CoreDescriptor(url: $0, isBundled: true) })
    }
    if let resourceURL = Bundle.main.resourceURL {
        cores.append(contentsOf: dylibURLs(in: resourceURL).map { CoreDescriptor(url: $0, isBundled: true) })
    }

    // Imported cores
    let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    let coresDir = docs.appendingPathComponent("Cores")
    if let urls = try? FileManager.default.contentsOfDirectory(at: coresDir, includingPropertiesForKeys: nil) {
        cores.append(contentsOf: urls.filter { $0.pathExtension == "dylib" }.map { CoreDescriptor(url: $0, isBundled: false) })
    }

    self.availableCores = cores
  }

  private func dylibURLs(in directory: URL) -> [URL] {
    let urls = try? FileManager.default.contentsOfDirectory(at: directory, includingPropertiesForKeys: nil)
    return urls?.filter { $0.pathExtension == "dylib" } ?? []
  }

  public func loadCore(_ core: CoreDescriptor) {
    guard let frontend else { return }
    do {
      stop()
      systemInfo = try frontend.loadCore(at: core.url.path)
      coreURL = core.url
      refresh()
      statusMessage = "Loaded core: \(systemInfo?.libraryName ?? "Unknown")"
    } catch {
      statusMessage = "Core load failed: \(error)"
    }
  }

  public func loadGame(at url: URL) {
    guard let frontend else { return }
    do {
      try frontend.loadGame(at: url.path)
      loadedGameURL = url
      refresh()
      statusMessage = "Loaded game: \(url.lastPathComponent)"
    } catch {
      statusMessage = "Game load failed: \(error)"
    }
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
