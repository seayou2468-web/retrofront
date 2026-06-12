import Foundation
import RetrofrontSwift
import UIKit

public enum FrontendAssetArchive: String, CaseIterable, Identifiable, Sendable {
  case assets
  case info
  case overlays
  case autoconfig
  case cheats
  case databaseRdb = "database-rdb"

  public var id: String { rawValue }
  public var fileName: String { "\(rawValue).zip" }
  public var displayName: String { fileName }
  public var downloadURL: URL {
    URL(string: "https://buildbot.libretro.com/assets/frontend/\(fileName)")!
  }
}

extension EmulatorRuntimeModel {
  public func refreshAvailableCores() {
    guard let frontend else { return }
    if let frameworksURL = Bundle.main.privateFrameworksURL {
      frontend.scanCores(in: frameworksURL.path)
    }
    if let resourceURL = Bundle.main.resourceURL {
      frontend.scanCores(in: resourceURL.path)
      frontend.scanCores(in: resourceURL.appendingPathComponent("dylibs").path)
    }
    frontend.scanCores(in: storageLayout.retroArchSettings.first(where: { $0.key == "libretro_directory" })?.url.path ?? retroArchRoot.appendingPathComponent("cores", isDirectory: true).path)
    frontend.scanCores(in: retroArchRoot.appendingPathComponent("cores", isDirectory: true).path)
    frontend.scanCores(in: retroArchRoot.appendingPathComponent("Cores", isDirectory: true).path)
    frontend.scanConfiguredCores()
    availableCores = frontend.availableCores()
  }

  public func refreshGames() {
    guard let frontend else { return }
    refreshAvailableCores()
    let contentDir = URL(fileURLWithPath: frontend.setting("content_directory") ?? storageLayout.contentDirectory.path)
    try? FileManager.default.createDirectory(at: contentDir, withIntermediateDirectories: true)
    let exts = frontend.allSupportedExtensions().joined(separator: "|")
    frontend.scanGames(in: contentDir.path, extensions: exts)
    availableGames = sortedGames(frontend.availableGames())
  }

  public func rescanLibrary() {
    refreshAvailableCores()
    refreshGames()
    refresh()
    statusMessage = "Library refreshed"
  }

  public func importFile(at url: URL) {
    importGame(at: url)
  }

  public func importGame(at url: URL) {
    guard let frontend else { return }
    let destinationDir = URL(fileURLWithPath: frontend.setting("content_directory") ?? storageLayout.contentDirectory.path)
    try? FileManager.default.createDirectory(at: destinationDir, withIntermediateDirectories: true)
    let destination = destinationDir.appendingPathComponent(url.lastPathComponent)
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

  func installBundledAssetsIfNeeded(_ frontend: Retrofront) {
    var installedAnyArchive = false
    for archive in FrontendAssetArchive.allCases where bundledArchiveNeedsInstall(archive) {
      installBundledAsset(archive, frontend: frontend, updateStatus: false)
      installedAnyArchive = true
    }
    if installedAnyArchive {
      refreshOverlayChoices()
    }
    applyBundleCoreDirectories(frontend)
  }

  func bundledArchiveNeedsInstall(_ archive: FrontendAssetArchive) -> Bool {
    let probes: [URL]
    switch archive {
    case .assets:
      probes = [
        storageLayout.assetsDirectory.appendingPathComponent("materialui", isDirectory: true),
        storageLayout.assetsDirectory.appendingPathComponent("ozone", isDirectory: true),
        storageLayout.assetsDirectory.appendingPathComponent("xmb", isDirectory: true),
        storageLayout.assetsDirectory.appendingPathComponent("rgui", isDirectory: true)
      ]
    case .info:
      probes = [storageLayout.infoDirectory.appendingPathComponent("mgba_libretro.info")]
    case .overlays:
      probes = [storageLayout.overlaysDirectory.appendingPathComponent("gamepads", isDirectory: true)]
    case .autoconfig:
      probes = [storageLayout.autoconfigDirectory.appendingPathComponent("udev", isDirectory: true)]
    case .cheats:
      probes = [storageLayout.cheatsDirectory]
    case .databaseRdb:
      probes = [storageLayout.databaseDirectory]
    }
    return probes.contains { probe in
      var isDirectory: ObjCBool = false
      guard FileManager.default.fileExists(atPath: probe.path, isDirectory: &isDirectory) else { return true }
      guard isDirectory.boolValue else { return false }
      let contents = (try? FileManager.default.contentsOfDirectory(atPath: probe.path)) ?? []
      return contents.isEmpty
    }
  }

  public func installBundledAssets() {
    for archive in FrontendAssetArchive.allCases {
      installBundledAsset(archive)
    }
  }

  public func installBundledAsset(_ archive: FrontendAssetArchive) {
    guard let frontend else { return }
    installBundledAsset(archive, frontend: frontend, updateStatus: true)
  }

  func installBundledAssets(_ frontend: Retrofront, updateStatus: Bool) {
    for archive in FrontendAssetArchive.allCases {
      installBundledAsset(archive, frontend: frontend, updateStatus: updateStatus)
    }
  }

  func installBundledAsset(_ archive: FrontendAssetArchive, frontend: Retrofront, updateStatus: Bool) {
    guard let zipURL = Bundle.main.url(forResource: archive.rawValue, withExtension: "zip") else {
      if updateStatus { statusMessage = "\(archive.fileName) was not found in the app bundle" }
      return
    }
    installArchive(frontend, archive: archive, zipURL: zipURL, sourceLabel: "bundled", updateStatus: updateStatus)
  }

  public func downloadAndInstallAsset(_ archive: FrontendAssetArchive) {
    guard let frontend else { return }
    statusMessage = "Downloading \(archive.fileName)…"
    Task { [weak self, archive, frontend] in
      do {
        let (temporaryURL, _) = try await URLSession.shared.download(from: archive.downloadURL)
        let cacheURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + "-" + archive.fileName)
        if FileManager.default.fileExists(atPath: cacheURL.path) { try FileManager.default.removeItem(at: cacheURL) }
        try FileManager.default.moveItem(at: temporaryURL, to: cacheURL)
        await MainActor.run {
          self?.installArchive(frontend, archive: archive, zipURL: cacheURL, sourceLabel: "downloaded", updateStatus: true)
          try? FileManager.default.removeItem(at: cacheURL)
        }
      } catch {
        await MainActor.run { self?.statusMessage = "Download failed for \(archive.fileName): \(error)" }
      }
    }
  }

  func installArchive(_ frontend: Retrofront, archive: FrontendAssetArchive, zipURL: URL, sourceLabel: String, updateStatus: Bool) {
    let installRoot = installDestination(for: archive)
    do {
      try FileManager.default.createDirectory(at: installRoot, withIntermediateDirectories: true)
      let report = try frontend.installAssetsZip(from: zipURL.path, to: installRoot.path)
      applyBundleCoreDirectories(frontend)
      refreshOverlayChoices()
      loadConfiguredOverlay(frontend)
      refresh()
      if updateStatus {
        statusMessage = "Installed \(sourceLabel) \(archive.fileName): \(report.filesWritten) files into \(installRoot.lastPathComponent)"
      }
    } catch {
      if updateStatus { statusMessage = "Install failed for \(archive.fileName): \(error)" }
    }
  }

  func installDestination(for archive: FrontendAssetArchive) -> URL {
    // Libretro buildbot frontend zips contain the contents of each RetroArch
    // directory. Extract each archive directly into the matching configured
    // directory so ozone/materialui/xmb/rgui assets and overlays resolve from
    // the same paths RetroArch writes to retroarch.cfg.
    switch archive {
    case .assets: return storageLayout.assetsDirectory
    case .info: return storageLayout.infoDirectory
    case .overlays: return storageLayout.overlaysDirectory
    case .autoconfig: return storageLayout.autoconfigDirectory
    case .cheats: return storageLayout.cheatsDirectory
    case .databaseRdb: return storageLayout.databaseDirectory
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
    refreshAvailableCores()
    guard FileManager.default.fileExists(atPath: url.path) else {
      statusMessage = "Game file missing: \(url.lastPathComponent)"
      refreshGames()
      return
    }
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
      statusMessage = "No compatible core found for .\(plan.contentExtension). Load a matching bundled core first."
    @unknown default:
      statusMessage = "Unsupported launch plan for .\(plan.contentExtension)."
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

  func doLaunch(_ url: URL, preferredCore: String?) {
    guard let frontend else { return }
    do {
      stop()
      try frontend.launchContent(at: url.path, preferredCore: preferredCore)
      loadedGameURL = url
      systemInfo = try? frontend.systemInfo()
      corePath = preferredCore ?? frontend.planContentLaunch(path: url.path)?.selectedCorePath
      refresh()
      launchToken &+= 1
      statusMessage = "Loaded game: \(url.lastPathComponent)"
    } catch {
      statusMessage = "Game load failed: \(error)"
    }
  }
}
