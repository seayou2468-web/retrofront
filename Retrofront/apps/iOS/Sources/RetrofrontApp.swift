import CRetrofrontCore
import SwiftUI

@main
struct RetrofrontApp: App {
  @StateObject private var frontend = RetrofrontModel()

  var body: some Scene {
    WindowGroup {
      StudioHomeView()
        .environmentObject(frontend)
        .task { frontend.bootstrap() }
    }
  }
}

@MainActor
final class RetrofrontModel: ObservableObject {
  @Published var status = "Ready"
  @Published var stateText = "No core loaded"
  @Published var cores: [Row] = []
  @Published var games: [Row] = []
  @Published var settings: [Row] = []
  @Published var apiRows: [Row] = []
  @Published var storageRows: [Row] = []
  @Published var featureRows: [Row] = []

  private var frontend: OpaquePointer?
  private var rootURL: URL {
    FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
      .appendingPathComponent("RetroArch", isDirectory: true)
  }

  deinit {
    if let frontend { rf_frontend_destroy(frontend) }
  }

  func bootstrap() {
    if frontend == nil { frontend = rf_frontend_create() }
    createRetroArchLayout()
    configureCore()
    refreshAll(message: "Bootstrapped native iOS shell")
  }

  func refreshAll(message: String? = nil) {
    guard let frontend else { return }
    rf_frontend_scan_configured_cores(frontend)
    scanGames()
    stateText = stateLabel(rf_frontend_state(frontend))
    cores = collectCores(frontend)
    games = collectGames(frontend)
    settings = collectSettings(frontend)
    storageRows = storageMap()
    apiRows = libretroMatrix()
    featureRows = replacementReadiness()
    if let message { status = message }
  }

  func scanGames() {
    guard let frontend else { return }
    let extensions = String(cString: rf_frontend_all_extensions(frontend))
    withCString(rootURL.appendingPathComponent("Roms").path) { pathPtr in
      withCString(extensions) { extPtr in
        rf_frontend_scan_games(frontend, pathPtr, extPtr)
      }
    }
  }

  func runFrame() {
    guard let frontend else { return }
    status = rf_frontend_run_frame(frontend) ? "Ran one frame" : lastError()
    refreshAll()
  }

  func reset() {
    guard let frontend else { return }
    status = rf_frontend_reset(frontend) ? "Reset active content" : lastError()
    refreshAll()
  }

  func saveState() {
    guard let frontend else { return }
    status = rf_frontend_save_state(frontend, 0) ? "Saved state slot 0" : lastError()
  }

  func loadState() {
    guard let frontend else { return }
    status = rf_frontend_load_state(frontend, 0) ? "Loaded state slot 0" : lastError()
  }

  func saveSRAM() {
    guard let frontend else { return }
    status = rf_frontend_save_sram(frontend) ? "Saved SRAM" : lastError()
  }

  func stop() {
    guard let frontend else { return }
    _ = rf_frontend_save_sram(frontend)
    rf_frontend_unload_game(frontend)
    status = "Stopped content and flushed SRAM"
    refreshAll()
  }

  private func configureCore() {
    guard let frontend else { return }
    let layout = makeLayoutURLs()
    withCString(rootURL.path) { rf_frontend_set_base_dir(frontend, $0) }
    for (key, url) in layout {
      withCString(key) { keyPtr in
        withCString(url.path) { valuePtr in
          _ = rf_frontend_set_setting(frontend, keyPtr, valuePtr)
        }
      }
    }
    withCString(layout["libretro_info_path"]!.path) { rf_frontend_set_info_dir(frontend, $0) }
    withCString(rootURL.appendingPathComponent("config/retroarch.cfg").path) {
      _ = rf_frontend_load_settings(frontend, $0)
    }
    if let bundleFrameworks = Bundle.main.privateFrameworksURL?.path {
      withCString(bundleFrameworks) { rf_frontend_scan_cores(frontend, $0) }
    }
    rf_frontend_save_settings(frontend)
  }

  private func createRetroArchLayout() {
    for url in Array(makeLayoutURLs().values) + [rootURL.appendingPathComponent("config", isDirectory: true)] {
      try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
    }
  }

  private func makeLayoutURLs() -> [String: URL] {
    [
      "libretro_directory": Bundle.main.privateFrameworksURL ?? rootURL.appendingPathComponent("cores", isDirectory: true),
      "libretro_info_path": rootURL.appendingPathComponent("info", isDirectory: true),
      "assets_directory": rootURL.appendingPathComponent("assets", isDirectory: true),
      "menu_assets_directory": rootURL.appendingPathComponent("assets", isDirectory: true),
      "overlay_directory": rootURL.appendingPathComponent("overlays", isDirectory: true),
      "content_directory": rootURL.appendingPathComponent("Roms", isDirectory: true),
      "core_assets_directory": rootURL.appendingPathComponent("downloads", isDirectory: true),
      "savefile_directory": rootURL.appendingPathComponent("saves", isDirectory: true),
      "savestate_directory": rootURL.appendingPathComponent("states", isDirectory: true),
      "system_directory": rootURL.appendingPathComponent("system", isDirectory: true),
      "screenshot_directory": rootURL.appendingPathComponent("screenshots", isDirectory: true),
      "playlist_directory": rootURL.appendingPathComponent("playlists", isDirectory: true),
      "cache_directory": FileManager.default.temporaryDirectory,
    ]
  }

  private func collectCores(_ frontend: OpaquePointer) -> [Row] {
    (0..<rf_frontend_cores_count(frontend)).compactMap { index in
      var info = RfCoreInfo()
      guard rf_frontend_get_core_info(frontend, index, &info) else { return nil }
      return Row(title: string(info.display_name), subtitle: "\(string(info.system_name)) • \(string(info.supported_extensions))")
    }
  }

  private func collectGames(_ frontend: OpaquePointer) -> [Row] {
    (0..<rf_frontend_games_count(frontend)).compactMap { index in
      var info = RfGameEntry()
      guard rf_frontend_get_game_info(frontend, index, &info) else { return nil }
      return Row(title: string(info.label), subtitle: string(info.path))
    }
  }

  private func collectSettings(_ frontend: OpaquePointer) -> [Row] {
    (0..<rf_frontend_settings_count(frontend)).compactMap { index in
      var entry = RfSettingEntry()
      guard rf_frontend_get_setting_at(frontend, index, &entry) else { return nil }
      return Row(title: string(entry.key), subtitle: string(entry.value))
    }.sorted { $0.title < $1.title }
  }

  private func storageMap() -> [Row] {
    makeLayoutURLs().sorted { $0.key < $1.key }.map { Row(title: $0.key, subtitle: $0.value.path) }
  }

  private func libretroMatrix() -> [Row] {
    [
      Row(title: "Core lifecycle", subtitle: "load/unload, reset, run, no-game support, API version"),
      Row(title: "Environment", subtitle: "options v0/v1/v2/intl, messages, geometry, directories, VFS v4"),
      Row(title: "Video/audio", subtitle: "software frame ingest, audio sample/batch, frame stepping, telemetry"),
      Row(title: "Input", subtitle: "joypad bitmasks, keyboard callback, descriptors, rumble, sensors, overlays"),
      Row(title: "Storage", subtitle: "SRAM, savestates, system/save/state/cache/playlist directories"),
      Row(title: "Platform", subtitle: "performance, MIDI, LED, location/camera safe stubs, proc-address"),
    ]
  }

  private func replacementReadiness() -> [Row] {
    [
      Row(title: "Native iOS startup", subtitle: "SwiftUI app owns launch; Rust core is a small staticlib instead of Slint/Skia app shell"),
      Row(title: "Core policy", subtitle: "No embedded core filtering: Xcode copy phase still copies every dylib/framework"),
      Row(title: "Bundle size", subtitle: "Removed MoltenVK/OpenGLES/GLKit and Slint UI staticlib from iOS target"),
      Row(title: "Files app", subtitle: "Document sharing/import enabled for ROM, save, state, archive data"),
      Row(title: "RetroArch layout", subtitle: "Documents/RetroArch contains Roms, saves, states, system, overlays, assets, playlists"),
    ]
  }

  private func lastError() -> String {
    guard let frontend, let error = rf_frontend_last_error(frontend) else { return "Operation failed" }
    return String(cString: error)
  }
}

struct Row: Identifiable {
  let id = UUID()
  let title: String
  let subtitle: String
}

struct StudioHomeView: View {
  @EnvironmentObject private var model: RetrofrontModel

  var body: some View {
    NavigationStack {
      ScrollView {
        VStack(spacing: 18) {
          hero
          metrics
          commandDeck
          section("Library", rows: model.games.isEmpty ? [Row(title: "Drop ROMs in Files", subtitle: "On My iPhone/Retrofront/RetroArch/Roms")] : model.games, tint: .cyan)
          section("Cores", rows: model.cores.isEmpty ? [Row(title: "No cores discovered", subtitle: "Bundled Frameworks are scanned at startup")] : model.cores, tint: .purple)
          section("Feature Matrix", rows: model.featureRows, tint: .green)
          section("libretro API", rows: model.apiRows, tint: .orange)
          section("Storage Map", rows: model.storageRows, tint: .blue)
          section("Settings", rows: Array(model.settings.prefix(40)), tint: .gray)
        }
        .padding(20)
      }
      .background(LinearGradient(colors: [.black, Color(red: 0.05, green: 0.09, blue: 0.16), Color(red: 0.04, green: 0.20, blue: 0.31)], startPoint: .topLeading, endPoint: .bottomTrailing))
      .navigationTitle("Retrofront Studio")
      .toolbar { Button("Refresh") { model.refreshAll(message: "Refreshed") } }
    }
  }

  private var hero: some View {
    VStack(alignment: .leading, spacing: 12) {
      Text("Retrofront Studio")
        .font(.system(size: 42, weight: .black, design: .rounded))
      Text("Native iOS libretro frontend • RetroArch-compatible storage • exhaustive runtime control surface")
        .font(.headline)
        .foregroundStyle(.white.opacity(0.78))
      Text(model.status)
        .font(.subheadline.weight(.bold))
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(.white.opacity(0.14), in: Capsule())
    }
    .foregroundStyle(.white)
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(24)
    .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 32, style: .continuous))
  }

  private var metrics: some View {
    LazyVGrid(columns: [GridItem(.adaptive(minimum: 150), spacing: 12)], spacing: 12) {
      metric("State", model.stateText, .blue)
      metric("Cores", "\(model.cores.count)", .purple)
      metric("Games", "\(model.games.count)", .cyan)
      metric("Settings", "\(model.settings.count)", .green)
    }
  }

  private var commandDeck: some View {
    VStack(alignment: .leading, spacing: 12) {
      Text("Command Deck").font(.title2.bold()).foregroundStyle(.white)
      LazyVGrid(columns: [GridItem(.adaptive(minimum: 150), spacing: 10)], spacing: 10) {
        action("Run frame", system: "play.fill", model.runFrame)
        action("Reset", system: "arrow.clockwise", model.reset)
        action("Save state", system: "square.and.arrow.down", model.saveState)
        action("Load state", system: "square.and.arrow.up", model.loadState)
        action("Save SRAM", system: "externaldrive.fill", model.saveSRAM)
        action("Stop", system: "stop.fill", model.stop)
      }
    }
    .padding(18)
    .background(.white.opacity(0.08), in: RoundedRectangle(cornerRadius: 26, style: .continuous))
  }

  private func metric(_ title: String, _ value: String, _ color: Color) -> some View {
    VStack(alignment: .leading, spacing: 8) {
      Text(title.uppercased()).font(.caption.bold()).foregroundStyle(color)
      Text(value).font(.title2.bold()).foregroundStyle(.white).lineLimit(1)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(18)
    .background(.white.opacity(0.09), in: RoundedRectangle(cornerRadius: 24, style: .continuous))
  }

  private func action(_ title: String, system: String, _ perform: @escaping () -> Void) -> some View {
    Button(action: perform) {
      Label(title, systemImage: system)
        .font(.headline)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 14)
    }
    .buttonStyle(.borderedProminent)
    .tint(.indigo)
  }

  private func section(_ title: String, rows: [Row], tint: Color) -> some View {
    VStack(alignment: .leading, spacing: 10) {
      Text(title).font(.title2.bold()).foregroundStyle(.white)
      ForEach(rows) { row in
        HStack(spacing: 12) {
          Circle().fill(tint.gradient).frame(width: 12, height: 12)
          VStack(alignment: .leading, spacing: 3) {
            Text(row.title).font(.headline).foregroundStyle(.white).lineLimit(1)
            Text(row.subtitle).font(.caption).foregroundStyle(.white.opacity(0.65)).lineLimit(2)
          }
          Spacer()
        }
        .padding(14)
        .background(.white.opacity(0.07), in: RoundedRectangle(cornerRadius: 18, style: .continuous))
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
  }
}

private func string(_ pointer: UnsafePointer<CChar>?) -> String {
  guard let pointer else { return "" }
  return String(cString: pointer)
}

private func withCString<Result>(_ string: String, _ body: (UnsafePointer<CChar>) -> Result) -> Result {
  string.withCString(body)
}

private func stateLabel(_ state: UInt32) -> String {
  switch state {
  case 1: return "Core loaded"
  case 2: return "Game loaded"
  default: return "No core loaded"
  }
}
