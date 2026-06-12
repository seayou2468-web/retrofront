import Foundation
import RetrofrontSwift

extension EmulatorRuntimeModel {
  func setupFrontend() {
    do {
      let frontend = try Retrofront()
      self.frontend = frontend

      let layout = storageLayout
      let root = layout.root
      try layout.createWritableDirectories()
      let configPath = layout.configFile.path
      try? frontend.setBaseDirectory(root.path)
      applyBundleCoreDirectories(frontend)
      try? frontend.loadSettings(at: configPath)
      applyBundleCoreDirectories(frontend)
      applyRetroArchStorageSettings(frontend, layout: layout)
      installBundledAssetsIfNeeded(frontend)
      refreshOverlayChoices()
      applyRendererSetting(frontend)
      applyVideoSettings(frontend)
      loadConfiguredOverlay(frontend)
      frontend.saveSettings()
      refresh()
    } catch {
      statusMessage = "Initialization failed: \(error)"
    }
  }

  func applyBundleCoreDirectories(_ frontend: Retrofront) {
    let layout = storageLayout
    frontend.setInfoDir(layout.infoDirectory.path)
    for setting in layout.retroArchSettings {
      try? frontend.setSetting(key: setting.key, value: setting.url.path)
    }
  }

  func applyRetroArchStorageSettings(_ frontend: Retrofront, layout: RetroArchStorageLayout) {
    for setting in layout.retroArchSettings {
      try? frontend.setSetting(key: setting.key, value: setting.url.path)
    }
  }
}
