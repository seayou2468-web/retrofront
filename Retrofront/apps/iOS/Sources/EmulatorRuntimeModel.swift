import Foundation
import RetrofrontSwift

@MainActor
final class EmulatorRuntimeModel: ObservableObject {
  @Published private(set) var frontendState: FrontendState = .empty
  @Published private(set) var systemInfo: LibretroSystemInfo?
  @Published private(set) var recentEvents: [FrontendEvent] = []
  @Published private(set) var latestFrame: VideoFrame?
  @Published var selectedTab: AppSection = .library
  @Published var statusMessage = "Ready without an emulator core."

  private let frontend: Retrofront?

  init() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend
      try? frontend.setGfxBackend(.openGL)
      frontendState = frontend.state
      statusMessage = "Runtime connected. Rust gfx is ready for libretro frames."
    } catch {
      frontend = nil
      statusMessage = "Runtime unavailable: \(error)"
    }
  }

  var isRuntimeConnected: Bool {
    frontend != nil
  }

  var canRunGame: Bool {
    frontendState == .gameLoaded
  }

  func refresh() {
    guard let frontend else { return }
    frontendState = frontend.state
    recentEvents = frontend.drainEvents()
    latestFrame = frontend.latestVideoFrame()
    if let info = try? frontend.systemInfo() {
      systemInfo = info
    }
  }

  func runOneFrameFromButton() {
    guard let frontend else { return }
    do {
      recentEvents = try frontend.runFrame()
      latestFrame = frontend.latestVideoFrame()
      frontendState = frontend.state
      statusMessage = latestFrame.map { "Displayed frame \($0.frameNumber) via Rust gfx." }
        ?? "Ran one libretro frame; waiting for video output."
    } catch {
      statusMessage = "Run failed: \(error)"
    }
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
