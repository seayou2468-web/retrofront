import Foundation
import RetrofrontSwift
import UIKit

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
    let infoProbe = storageLayout.infoDirectory.appendingPathComponent("mgba_libretro.info")
    guard !FileManager.default.fileExists(atPath: infoProbe.path) else {
      applyBundleCoreDirectories(frontend)
      return
    }
    installBundledAssets(frontend, updateStatus: false)
  }

  public func installBundledAssets() {
    guard let frontend else { return }
    installBundledAssets(frontend, updateStatus: true)
  }

  func installBundledAssets(_ frontend: Retrofront, updateStatus: Bool) {
    guard let zipURL = Bundle.main.url(forResource: "assets", withExtension: "zip") else {
      if updateStatus { statusMessage = "assets.zip was not found in the app bundle" }
      return
    }
    let installRoot = storageLayout.root
    do {
      let report = try frontend.installAssetsZip(from: zipURL.path, to: installRoot.path)
      applyBundleCoreDirectories(frontend)
      refreshOverlayChoices()
      loadConfiguredOverlay(frontend)
      refresh()
      if updateStatus { statusMessage = "Installed assets: \(report.filesWritten) files" }
    } catch {
      if updateStatus { statusMessage = "Assets install failed: \(error)" }
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
