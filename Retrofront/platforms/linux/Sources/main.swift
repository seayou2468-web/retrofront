import Foundation

func readString(_ fill: (UnsafeMutablePointer<CChar>?, Int) -> Bool) -> String {
    var buffer = [CChar](repeating: 0, count: 1024)
    let capacity = buffer.count
    _ = buffer.withUnsafeMutableBufferPointer { fill($0.baseAddress, capacity) }
    return String(cString: buffer)
}

func printMenu() {
    let title = readString { retrofront_menu_title($0, $1) }
    let driver = readString { retrofront_menu_driver($0, $1) }
    print("\n== \(title) [\(driver)] ==")
    let selected = Int(retrofront_menu_selected_index())
    for index in 0..<Int(retrofront_menu_entry_count()) {
        let marker = index == selected ? ">" : " "
        let label = readString { retrofront_menu_entry_label(index, $0, $1) }
        let sublabel = readString { retrofront_menu_entry_sublabel(index, $0, $1) }
        print("\(marker) \(label)\(sublabel.isEmpty ? "" : " — \(sublabel)")")
    }
}

func writeSnapshot(_ output: URL) {
    _ = output.path.withCString { retrofront_renderer_write_snapshot_ppm($0) }
    print("snapshot: \(output.path)")
}

let home = URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
let root = home.appendingPathComponent(".local/share/retrofront/RetroArch", isDirectory: true)
try? FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)

let ok = root.path.withCString { retrofront_runtime_init($0) }
guard ok else {
    print("Retrofront Linux runtime: failed")
    exit(1)
}

_ = retrofront_menu_bootstrap()
_ = retrofront_renderer_resize(1280, 720)
let snapshots = root.appendingPathComponent("ui-snapshots", isDirectory: true)
try? FileManager.default.createDirectory(at: snapshots, withIntermediateDirectories: true)

for driver in ["ozone", "xmb", "materialui", "rgui"] {
    driver.withCString { _ = retrofront_menu_set_driver($0) }
    _ = retrofront_menu_bootstrap()
    _ = retrofront_menu_draw()
    printMenu()

    writeSnapshot(snapshots.appendingPathComponent("\(driver)-root.ppm"))

    _ = retrofront_menu_action(1) // Down: Playlists
    _ = retrofront_menu_action(4) // Ok
    _ = retrofront_menu_draw()
    printMenu()
    writeSnapshot(snapshots.appendingPathComponent("\(driver)-playlists.ppm"))

    _ = retrofront_menu_action(4) // Ok: Favorites
    _ = retrofront_menu_draw()
    printMenu()
    writeSnapshot(snapshots.appendingPathComponent("\(driver)-playlist-favorites.ppm"))
}

retrofront_runtime_shutdown()
