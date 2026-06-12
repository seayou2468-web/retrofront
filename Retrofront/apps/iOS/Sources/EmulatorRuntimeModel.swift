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
  @Published private(set) var overlayInfo: OverlayInfo?
  @Published private(set) var availableOverlays: [OverlayChoice] = []
  @Published private(set) var settings: [RetrofrontSetting] = []
  @Published private(set) var pendingCoreChoices: [CoreInfo] = []
  @Published private(set) var pendingContentURL: URL?
  @Published private(set) var launchToken: UInt = 0
  @Published private(set) var menuToken: UInt = 0
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
