import Foundation
import RetrofrontSwift

struct LinuxDashboard {
  let frontend: Retrofront
  let layout: RetroArchLinuxLayout

  func render() {
    let cores = frontend.availableCores()
    print("""
    Retrofront Linux
    ─────────────────
    State              \(frontend.state)
    Config root        \(layout.root.path)
    External cores     \(layout.userCoreDirectory.path)
    Bundled cores      \(layout.bundledCoreDirectory?.path ?? "not packaged")
    Assets             \(layout.assetsDirectory.path)
    Overlays           \(layout.overlaysDirectory.path)
    Core count         \(cores.count)

    Put additional Linux libretro .so cores in External cores. AppImage-bundled cores remain read-only and are scanned separately.
    """)

    if !cores.isEmpty {
      print("\nCores")
      for core in cores.prefix(20) {
        print("• \(core.displayName) — \(URL(fileURLWithPath: core.path).lastPathComponent)")
      }
      if cores.count > 20 { print("• … \(cores.count - 20) more") }
    }

    if let menu = frontend.currentMenuList() {
      print("\n\(menu.title)")
      for entry in menu.entries {
        let value = entry.value.isEmpty ? "" : "  \(entry.value)"
        print("• \(entry.label) — \(entry.sublabel)\(value)")
      }
    }
  }
}

func installPackagedAssets(frontend: Retrofront, layout: RetroArchLinuxLayout) {
  guard let assetDir = layout.bundledAssetDirectory else { return }
  let packages: [(String, URL)] = [
    ("assets", layout.assetsDirectory),
    ("info", layout.infoDirectory),
    ("overlays", layout.overlaysDirectory),
  ]
  for (name, destination) in packages {
    let zip = assetDir.appendingPathComponent("\(name).zip")
    if FileManager.default.fileExists(atPath: zip.path) {
      _ = try? frontend.installAssetsZip(from: zip.path, to: destination.path)
    }
  }
}

do {
  let frontend = try Retrofront()
  let layout = RetroArchLinuxLayout.current
  try layout.apply(to: frontend)
  try? frontend.loadSettings(at: layout.configFile.path)
  try layout.apply(to: frontend)
  installPackagedAssets(frontend: frontend, layout: layout)
  if let bundled = layout.bundledCoreDirectory { frontend.scanCores(in: bundled.path) }
  frontend.scanCores(in: layout.userCoreDirectory.path)
  frontend.scanConfiguredCores()
  frontend.saveSettings()
  LinuxDashboard(frontend: frontend, layout: layout).render()
} catch {
  fputs("Linux UI failed to start: \(error)\n", stderr)
  exit(1)
}
