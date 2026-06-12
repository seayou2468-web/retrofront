import Foundation
import RetrofrontSwift
import UIKit
import Combine
import UniformTypeIdentifiers

public struct OverlayChoice: Identifiable, Equatable, Sendable {
  public let id: String
  public let path: String
  public let label: String
}

@MainActor
public final class EmulatorRuntimeModel: ObservableObject {
  @Published var frontendState: FrontendState = .empty
  @Published var systemInfo: LibretroSystemInfo?
  @Published var coreOptions: [CoreOption] = []
  @Published var displayImage: UIImage?
  @Published var aspectRatio: Double = 4.0/3.0
  @Published var isRunning = false
  @Published var availableCores: [CoreInfo] = []
  @Published var availableGames: [GameEntrySwift] = []
  @Published var corePath: String?
  @Published var loadedGameURL: URL?
  @Published var currentMenu: MenuList?
  @Published var overlayInfo: OverlayInfo?
  @Published var availableOverlays: [OverlayChoice] = []
  @Published var settings: [RetrofrontSetting] = []
  @Published var pendingCoreChoices: [CoreInfo] = []
  @Published var pendingContentURL: URL?
  @Published var launchToken: UInt = 0
  @Published var menuToken: UInt = 0
  @Published var statusMessage = "Ready"

  var frontend: Retrofront?
  var runTask: Task<Void, Never>?
  var pixelBuffer: Data?

  public init() {
    setupFrontend()
    refreshAvailableCores()
    refreshGames()
    refreshMenu()
    overlayInfo = frontend?.overlayInfo()
  }

  var storageLayout: RetroArchStorageLayout { .current }

  var retroArchRoot: URL { storageLayout.root }
}
