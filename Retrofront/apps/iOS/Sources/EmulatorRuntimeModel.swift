import Foundation
import RetrofrontSwift

@MainActor
final class EmulatorRuntimeModel: ObservableObject {
  @Published private(set) var frontendState: FrontendState = .empty
  @Published private(set) var systemInfo: LibretroSystemInfo?
  @Published private(set) var recentEvents: [FrontendEvent] = []
  @Published var selectedTab: AppSection = .library
  @Published var statusMessage = "Ready without an emulator core."

  private let frontend: Retrofront?

  init() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend
      frontendState = frontend.state
      statusMessage = "Runtime connected. Add a libretro core later to start emulation."
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
    if let info = try? frontend.systemInfo() {
      systemInfo = info
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
