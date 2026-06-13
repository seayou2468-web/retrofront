import SwiftUI

@main
struct RetrofrontApp: App {
    var body: some Scene {
        WindowGroup {
            RetrofrontView()
        }
    }
}

private struct MenuRow: Identifiable {
    let id: Int
    let label: String
    let sublabel: String
}

struct RetrofrontView: View {
    @State private var status = "Starting Retrofront"
    @State private var title = "Retrofront"
    @State private var rows: [MenuRow] = []
    @State private var selected = 0
    @State private var driver = "ozone"

    private let drivers = ["ozone", "xmb", "materialui", "rgui"]

    var body: some View {
        VStack(spacing: 14) {
            Picker("Menu Driver", selection: $driver) {
                ForEach(drivers, id: \.self) { Text($0).tag($0) }
            }
            .pickerStyle(.segmented)
            .onChange(of: driver) { value in
                value.withCString { _ = retrofront_menu_set_driver($0) }
                refresh()
            }

            Text(title)
                .font(.largeTitle.bold())
                .frame(maxWidth: .infinity, alignment: .leading)

            ScrollViewReader { proxy in
                List(rows) { row in
                    VStack(alignment: .leading, spacing: 3) {
                        Text(row.label)
                            .font(row.id == selected ? .headline : .body)
                        if !row.sublabel.isEmpty {
                            Text(row.sublabel)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .listRowBackground(row.id == selected ? selectedColor.opacity(0.28) : Color.clear)
                    .id(row.id)
                    .onTapGesture {
                        _ = retrofront_menu_set_selected_index(row.id)
                        _ = retrofront_menu_action(UInt32(4))
                        refresh()
                    }
                }
                .onChange(of: selected) { value in proxy.scrollTo(value, anchor: .center) }
            }

            HStack(spacing: 10) {
                Button("↑") { action(0) }
                Button("↓") { action(1) }
                Button("OK") { action(4) }
                Button("Back") { action(5) }
            }
            .buttonStyle(.borderedProminent)

            Text(status)
                .font(.footnote)
                .foregroundStyle(.secondary)
        }
        .padding()
        .background(backgroundColor)
        .onAppear(perform: boot)
    }

    private var selectedColor: Color {
        switch driver {
        case "xmb": return .blue
        case "materialui": return .green
        case "rgui": return .gray
        default: return .orange
        }
    }

    private var backgroundColor: Color {
        driver == "rgui" ? Color(white: 0.06) : Color(.systemBackground)
    }

    private func boot() {
        let fm = FileManager.default
        let support = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let dataRoot = support.appendingPathComponent("RetroArch", isDirectory: true)
        _ = try? fm.createDirectory(at: dataRoot, withIntermediateDirectories: true)

        let ok = dataRoot.path.withCString { retrofront_runtime_init($0) }
        guard ok else {
            status = "Rust runtime init failed"
            return
        }

        if let zip = Bundle.main.url(forResource: "assets", withExtension: "zip") {
            let count = zip.path.withCString { retrofront_resources_unpack($0) }
            status = "Ready (assets: \(count), real iOS device, C menu driver: \(driver))"
        } else {
            status = "Ready (assets.zip not found, C menu driver: \(driver))"
        }
        _ = retrofront_assets_load_defaults()
        _ = retrofront_menu_bootstrap()
        refresh()
    }

    private func action(_ code: UInt32) {
        _ = retrofront_menu_action(code)
        refresh()
    }

    private func refresh() {
        title = readString { retrofront_menu_title($0, $1) }
        driver = readString { retrofront_menu_driver($0, $1) }
        selected = Int(retrofront_menu_selected_index())
        let count = Int(retrofront_menu_entry_count())
        rows = (0..<count).map { index in
            MenuRow(
                id: index,
                label: readString { retrofront_menu_entry_label(index, $0, $1) },
                sublabel: readString { retrofront_menu_entry_sublabel(index, $0, $1) }
            )
        }
    }

    private func readString(_ fill: (UnsafeMutablePointer<CChar>?, Int) -> Bool) -> String {
        var buffer = [CChar](repeating: 0, count: 1024)
        let capacity = buffer.count
        _ = buffer.withUnsafeMutableBufferPointer { fill($0.baseAddress, capacity) }
        return String(cString: buffer)
    }
}
