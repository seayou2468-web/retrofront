import Foundation
import RetrofrontSwift
import CGtk

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
    Retrofront Linux GTK
    ====================

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

final class GtkDashboard {
  private let runtime: LinuxRetrofrontRuntime

  init(runtime: LinuxRetrofrontRuntime) {
    self.runtime = runtime
  }

  func run() {
    gtk_init(nil, nil)

    let window = rf_gtk_window_new()
    rf_gtk_window_set_title(window, "Retrofront")
    rf_gtk_window_set_default_size(window, 920, 640)
    rf_gtk_window_quit_on_destroy(window)

    let box = rf_gtk_box_new_vertical(0)
    rf_gtk_container_add(window, box)

    let title = gtk_label_new("Retrofront")
    gtk_widget_set_margin_top(title, 18)
    gtk_widget_set_margin_bottom(title, 8)
    rf_gtk_label_set_xalign(title, 0.0)
    rf_gtk_box_pack_start(box, title, 0, 0, 0)

    let subtitle = gtk_label_new("GTK shell using the shared Swift/Rust frontend runtime")
    gtk_widget_set_margin_bottom(subtitle, 12)
    rf_gtk_label_set_xalign(subtitle, 0.0)
    rf_gtk_box_pack_start(box, subtitle, 0, 0, 0)

    let textView = gtk_text_view_new()
    rf_gtk_text_view_set_editable(textView, 0)
    rf_gtk_text_view_set_cursor_visible(textView, 0)
    if let buffer = rf_gtk_text_view_get_buffer(textView) {
      gtk_text_buffer_set_text(buffer, runtime.dashboardText, -1)
    }

    let scroller = gtk_scrolled_window_new(nil, nil)
    rf_gtk_container_add(scroller, textView)
    rf_gtk_box_pack_start(box, scroller, 1, 1, 0)

    gtk_widget_show_all(window)
    gtk_main()
  }
}

do {
  let runtime = try LinuxRetrofrontRuntime()
  GtkDashboard(runtime: runtime).run()
} catch {
  fputs("retrofront-linux: \(error)\n", stderr)
  exit(1)
}
