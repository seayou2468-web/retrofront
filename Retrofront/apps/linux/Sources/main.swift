import Adwaita
import Foundation
import RetrofrontSwift

final class LinuxRetrofrontRuntime {
  let frontend: Retrofront
  let layout: RetroArchStorageLayout

  init() throws {
    frontend = try Retrofront()
    layout = .current
    try layout.createWritableDirectories()
    try frontend.setBaseDirectory(layout.root.path)
    try? frontend.loadSettings(at: layout.configFile.path)
    applyRetroArchLayout()
    frontend.setInfoDir(layout.infoDirectory.path)
    frontend.scanConfiguredCores()
    frontend.scanGames(in: layout.contentDirectory.path, extensions: frontend.allSupportedExtensions().joined(separator: "|"))
    frontend.saveSettings()
  }

  private func applyRetroArchLayout() {
    for setting in layout.retroArchSettings {
      try? frontend.setSetting(key: setting.key, value: setting.url.path)
    }
  }

  var dashboardText: String {
    let menu = frontend.currentMenuList()
    let menuText = menu?.entries.map { entry in
      let value = entry.value.isEmpty ? "" : "  [\(entry.value)]"
      return "• \(entry.label) — \(entry.sublabel)\(value)"
    }.joined(separator: "\n") ?? "No menu entries"

    return """
    State: \(frontend.state)
    RetroArch root: \(layout.root.path)
    Config: \(layout.configFile.path)
    Cores: \(frontend.availableCores().count)
    ROMs: \(frontend.availableGames().count)

    \(menu?.title ?? "Menu")
    \(menuText)
    """
  }
}

@MainActor
final class AdwaitaDashboard {
  private let runtime: LinuxRetrofrontRuntime

  init(runtime: LinuxRetrofrontRuntime) {
    self.runtime = runtime
  }

  func run() {
    let app = Application(id: "com.retrofront.linux")
    app.onActivate { [runtime] in
      let window = ApplicationWindow(application: app)
      window.title = "Retrofront"
      window.defaultWidth = 980
      window.defaultHeight = 680

      let toolbar = ToolbarView()
      toolbar.addTopBar(HeaderBar())

      let content = Box(orientation: .vertical, spacing: 16)
      content.setMargins(24)

      let title = Label("Retrofront")
        .cssClass(.title1)
        .halign(.start)
      content.append(title)

      let subtitle = Label("Swift Adwaita shell using the shared Swift/Rust runtime")
        .cssClass(.caption)
        .halign(.start)
      content.append(subtitle)

      let scroller = ScrolledWindow()
      let body = TextView()
      body.text = runtime.dashboardText
      body.editable = false
      body.cursorVisible = false
      body.monospace = true
      body.wrapMode = GTK_WRAP_WORD_CHAR
      body.hexpand()
      body.vexpand()
      scroller.child = body
      scroller.hexpand()
      scroller.vexpand()
      content.append(scroller)

      toolbar.setContent(content)
      window.setContent(toolbar)
      window.present()
    }
    app.run()
  }
}

@MainActor
func main() {
  do {
    let runtime = try LinuxRetrofrontRuntime()
    AdwaitaDashboard(runtime: runtime).run()
  } catch {
    FileHandle.standardError.write(Data("retrofront-linux: \(error)\n".utf8))
    exit(1)
  }
}

main()
