import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var selectedTab = 0
    @State private var isPlayViewActive = false
    @State private var isFilePickerPresented = false

    var body: some View {
        ZStack {
            Color(white: 0.05).ignoresSafeArea()

            TabView(selection: $selectedTab) {
                ModernHomeView(isPlayViewActive: $isPlayViewActive, isFilePickerPresented: $isFilePickerPresented)
                    .tabItem {
                        Label("Home", systemImage: "house.fill")
                    }.tag(0)

                ModernLibraryView(isFilePickerPresented: $isFilePickerPresented)
                    .tabItem {
                        Label("Library", systemImage: "gamecontroller.fill")
                    }.tag(1)

                ModernSettingsView()
                    .tabItem {
                        Label("Settings", systemImage: "gearshape.2.fill")
                    }.tag(2)
            }
            .tint(.cyan)
        }
        .fullScreenCover(isPresented: $isPlayViewActive) {
            PlayView()
        }
        .fileImporter(isPresented: $isFilePickerPresented, allowedContentTypes: [.item]) { result in
            if case .success(let url) = result {
                runtime.importFile(at: url)
            }
        }
        .onReceive(runtime.$frontendState) { newState in
            if newState == .gameLoaded {
                isPlayViewActive = true
            }
        }
    }
}

struct ModernHomeView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPlayViewActive: Bool
    @Binding var isFilePickerPresented: Bool

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    Text("Continue Playing")
                        .font(.title2.bold())
                        .padding(.horizontal)

                    if let game = runtime.loadedGameURL {
                        Button {
                            isPlayViewActive = true
                        } label: {
                            HStack {
                                Image(systemName: "play.circle.fill")
                                    .font(.largeTitle)
                                VStack(alignment: .leading) {
                                    Text(game.lastPathComponent)
                                        .font(.headline)
                                    Text(runtime.systemInfo?.libraryName ?? "Unknown Core")
                                        .font(.subheadline)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                            }
                            .padding()
                            .background(Color(white: 0.15))
                            .cornerRadius(12)
                        }
                        .padding(.horizontal)
                        .buttonStyle(.plain)
                    } else {
                        Text("No game loaded")
                            .foregroundStyle(.secondary)
                            .padding()
                            .frame(maxWidth: .infinity)
                            .background(Color(white: 0.1))
                            .cornerRadius(12)
                            .padding(.horizontal)
                    }

                    Text("Quick Actions")
                        .font(.title2.bold())
                        .padding(.horizontal)
                        .padding(.top)

                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 15) {
                        NavigationLink {
                            CoreListView()
                        } label: {
                            ActionCard(title: "Load Core", icon: "cpu", color: .purple) {}
                        }

                        ActionCard(title: "Import ROM", icon: "plus.circle", color: .green) {
                            isFilePickerPresented = true
                        }
                    }
                    .padding(.horizontal)

                    if !runtime.availableGames.isEmpty {
                        Text("Recently Added")
                            .font(.title2.bold())
                            .padding(.horizontal)
                            .padding(.top)

                        ForEach(runtime.availableGames.prefix(5), id: \.path) { game in
                            Button {
                                runtime.loadGame(at: URL(fileURLWithPath: game.path))
                            } label: {
                                HStack {
                                    Image(systemName: "doc.fill")
                                    Text(game.label)
                                    Spacer()
                                    Image(systemName: "chevron.right").font(.caption).foregroundStyle(.secondary)
                                }
                                .padding()
                                .background(Color(white: 0.15))
                                .cornerRadius(10)
                            }
                            .padding(.horizontal)
                            .buttonStyle(.plain)
                        }
                    }
                }
                .padding(.vertical)
            }
            .navigationTitle("Retrofront")
            .background(Color(white: 0.05))
        }
    }
}

struct CoreListView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        List {
            ForEach(runtime.availableCores, id: \.path) { core in
                Button {
                    runtime.loadCore(core)
                    dismiss()
                } label: {
                    VStack(alignment: .leading) {
                        Text(core.displayName)
                            .font(.headline)
                        Text(core.systemName)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .navigationTitle("Select Core")
    }
}

struct ActionCard: View {
    let title: String
    let icon: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 12) {
                Image(systemName: icon)
                    .font(.system(size: 30))
                    .foregroundStyle(color)
                Text(title)
                    .font(.headline)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 20)
            .background(
                LinearGradient(colors: [color.opacity(0.22), Color.white.opacity(0.06)], startPoint: .topLeading, endPoint: .bottomTrailing)
            )
            .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 18, style: .continuous)
                    .stroke(color.opacity(0.45), lineWidth: 1)
            )
            .shadow(color: color.opacity(0.18), radius: 16, x: 0, y: 8)
        }
        .buttonStyle(.plain)
    }
}

struct ModernLibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isFilePickerPresented: Bool

    var body: some View {
        NavigationStack {
            List {
                Section("My Games") {
                    if runtime.availableGames.isEmpty {
                        Text("No games found in Roms folder").foregroundStyle(.secondary)
                    } else {
                        ForEach(runtime.availableGames, id: \.path) { game in
                            Button {
                                runtime.loadGame(at: URL(fileURLWithPath: game.path))
                            } label: {
                                Text(game.label)
                            }
                        }
                    }
                }

                Section("Cores") {
                    ForEach(runtime.availableCores, id: \.path) { core in
                        Button {
                            runtime.loadCore(core)
                        } label: {
                            VStack(alignment: .leading) {
                                Text(core.displayName)
                                    .font(.headline)
                                Text(core.systemName)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }

                Section("Import") {
                    Button {
                        isFilePickerPresented = true
                    } label: {
                        Label("Add Content or Core", systemImage: "square.and.arrow.down")
                    }
                    Text(".dylib files are copied into the configured Cores directory; other files go to the content directory.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("Library")
            .background(Color(white: 0.05))
            .scrollContentBackground(.hidden)
        }
    }
}

struct ModernSettingsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        NavigationStack {
            RustMenuView()
                .navigationTitle("")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .navigationBarLeading) {
                        Button {
                            runtime.menuPop()
                        } label: {
                            Label("Back", systemImage: "chevron.left")
                        }
                    }
                }
        }
    }
}

struct RustMenuView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        Group {
            switch runtime.currentMenu?.driver ?? .xmb {
            case .rgui:
                RguiMenuView(menu: runtime.currentMenu, action: runtime.menuAction)
            case .materialui:
                MaterialMenuView(menu: runtime.currentMenu, action: runtime.menuAction)
            case .xmb:
                XmbMenuView(menu: runtime.currentMenu, action: runtime.menuAction)
            case .ozone:
                OzoneMenuView(menu: runtime.currentMenu, action: runtime.menuAction)
            }
        }
    }
}

private struct XmbMenuView: View {
    let menu: MenuList?
    let action: (UInt32) -> Void

    var body: some View {
        ZStack {
            LinearGradient(
                colors: [Color(red: 0.04, green: 0.05, blue: 0.09), Color(red: 0.02, green: 0.15, blue: 0.22)],
                startPoint: .top,
                endPoint: .bottom
            )
            .ignoresSafeArea()

            Circle()
                .fill(.white.opacity(0.18))
                .blur(radius: 70)
                .frame(width: 420, height: 420)
                .offset(x: -170, y: -250)

            VStack(alignment: .leading, spacing: 24) {
                Text(menu?.title ?? "Main Menu")
                    .font(.system(size: 28, weight: .light))
                    .foregroundStyle(.white.opacity(0.88))
                    .padding(.horizontal, 28)
                    .padding(.top, 30)

                HStack(alignment: .top, spacing: 34) {
                    ForEach(Array((menu?.entries ?? []).prefix(7).enumerated()), id: \.offset) { index, entry in
                        Button { action(entry.actionId) } label: {
                            VStack(spacing: 10) {
                                MenuEntryIcon(kind: entry.kind, family: "xmb", selected: index == 0)
                                Text(entry.label)
                                    .font(.caption.weight(index == 0 ? .semibold : .regular))
                                    .foregroundStyle(index == 0 ? .white : .white.opacity(0.62))
                                    .multilineTextAlignment(.center)
                                    .frame(width: 86)
                            }
                        }
                        .buttonStyle(.plain)
                        .disabled(entry.actionId == 0)
                    }
                }
                .padding(.horizontal, 32)

                ScrollView {
                    VStack(alignment: .leading, spacing: 7) {
                        ForEach(menu?.entries ?? [], id: \.actionId) { entry in
                            XmbSubRow(entry: entry) { action(entry.actionId) }
                        }
                    }
                    .padding(.horizontal, 62)
                    .padding(.bottom, 30)
                }
            }
        }
    }
}

private struct OzoneMenuView: View {
    let menu: MenuList?
    let action: (UInt32) -> Void

    var body: some View {
        HStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 18) {
                Text("RETROFRONT")
                    .font(.system(size: 18, weight: .bold))
                    .foregroundStyle(.white)
                Text(menu?.title.uppercased() ?? "MAIN MENU")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.white.opacity(0.55))
                Divider().background(.white.opacity(0.2))
                ForEach(Array((menu?.entries ?? []).prefix(9).enumerated()), id: \.offset) { index, entry in
                    Button { action(entry.actionId) } label: {
                        HStack(spacing: 12) {
                            MenuEntryIcon(kind: entry.kind, family: "ozone", selected: index == 0)
                            Text(entry.label)
                                .font(.subheadline.weight(index == 0 ? .bold : .regular))
                                .foregroundStyle(index == 0 ? .white : .white.opacity(0.72))
                        }
                    }
                    .buttonStyle(.plain)
                    .disabled(entry.actionId == 0)
                }
                Spacer()
            }
            .frame(width: 245)
            .padding(24)
            .background(Color(red: 0.10, green: 0.11, blue: 0.13))

            VStack(alignment: .leading, spacing: 18) {
                Text(menu?.title ?? "Main Menu")
                    .font(.system(size: 30, weight: .bold))
                    .foregroundStyle(.white)
                    .padding(.top, 26)
                    .padding(.horizontal, 30)

                ScrollView {
                    LazyVStack(spacing: 1) {
                        ForEach(menu?.entries ?? [], id: \.actionId) { entry in
                            OzoneRow(entry: entry) { action(entry.actionId) }
                        }
                    }
                    .padding(.horizontal, 30)
                    .padding(.bottom, 30)
                }
            }
            .background(Color(red: 0.15, green: 0.16, blue: 0.18))
        }
        .ignoresSafeArea()
    }
}

private struct MaterialMenuView: View {
    let menu: MenuList?
    let action: (UInt32) -> Void

    var body: some View {
        ZStack {
            Color(red: 0.94, green: 0.95, blue: 0.97).ignoresSafeArea()
            VStack(spacing: 0) {
                HStack {
                    Text(menu?.title ?? "Main Menu")
                        .font(.system(size: 22, weight: .medium))
                        .foregroundStyle(.white)
                    Spacer()
                    Text(menu?.theme ?? "materialui")
                        .font(.caption)
                        .foregroundStyle(.white.opacity(0.72))
                }
                .padding(.horizontal, 20)
                .frame(height: 64)
                .background(Color(red: 0.12, green: 0.47, blue: 0.74))

                ScrollView {
                    LazyVStack(spacing: 8) {
                        ForEach(menu?.entries ?? [], id: \.actionId) { entry in
                            MaterialRow(entry: entry) { action(entry.actionId) }
                        }
                    }
                    .padding(14)
                }
            }
        }
    }
}

private struct RguiMenuView: View {
    let menu: MenuList?
    let action: (UInt32) -> Void

    var body: some View {
        ZStack {
            Color(red: 0.02, green: 0.05, blue: 0.12).ignoresSafeArea()
            VStack(alignment: .leading, spacing: 0) {
                Text("┌─ \(menu?.title.uppercased() ?? "MAIN MENU") ".padding(toLength: 42, withPad: "─", startingAt: 0) + "┐")
                    .font(rguiFont)
                    .foregroundStyle(.cyan)
                ForEach(menu?.entries ?? [], id: \.actionId) { entry in
                    Button { action(entry.actionId) } label: {
                        Text(rguiLine(for: entry))
                            .font(rguiFont)
                            .foregroundStyle(entry.actionId == 0 ? .gray : .white)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .buttonStyle(.plain)
                    .disabled(entry.actionId == 0)
                }
                Spacer()
                Text("└" + String(repeating: "─", count: 45) + "┘")
                    .font(rguiFont)
                    .foregroundStyle(.cyan)
            }
            .padding(18)
        }
    }

    private var rguiFont: Font { .system(size: 16, weight: .regular, design: .monospaced) }

    private func rguiLine(for entry: MenuEntry) -> String {
        let marker = entry.kind == .submenu ? "▶" : " "
        let value = entry.value.isEmpty ? "" : "  " + entry.value
        return "│ \(marker) \(entry.label)\(value)".padding(toLength: 46, withPad: " ", startingAt: 0) + "│"
    }
}

private struct XmbSubRow: View {
    let entry: MenuEntry
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack {
                Text(entry.label)
                    .font(.system(size: 19, weight: .regular))
                    .foregroundStyle(.white.opacity(entry.actionId == 0 ? 0.42 : 0.92))
                Spacer()
                if !entry.value.isEmpty {
                    Text(entry.value)
                        .font(.caption)
                        .foregroundStyle(.white.opacity(0.55))
                        .lineLimit(1)
                }
            }
            .padding(.vertical, 7)
            .padding(.horizontal, 16)
            .background(Color.white.opacity(0.10))
        }
        .buttonStyle(.plain)
        .disabled(entry.actionId == 0)
    }
}

private struct OzoneRow: View {
    let entry: MenuEntry
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 14) {
                MenuEntryIcon(kind: entry.kind, family: "ozone", selected: false)
                VStack(alignment: .leading, spacing: 3) {
                    Text(entry.label)
                        .font(.headline)
                        .foregroundStyle(.white.opacity(entry.actionId == 0 ? 0.45 : 0.95))
                    if !entry.sublabel.isEmpty {
                        Text(entry.sublabel)
                            .font(.caption)
                            .foregroundStyle(.white.opacity(0.45))
                            .lineLimit(1)
                    }
                }
                Spacer()
                if entry.kind == .submenu { Image(systemName: "chevron.right").foregroundStyle(.white.opacity(0.35)) }
            }
            .padding(.vertical, 12)
            .padding(.horizontal, 14)
            .background(Color.white.opacity(0.035))
        }
        .buttonStyle(.plain)
        .disabled(entry.actionId == 0)
    }
}

private struct MaterialRow: View {
    let entry: MenuEntry
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 18) {
                MenuEntryIcon(kind: entry.kind, family: "material", selected: false)
                VStack(alignment: .leading, spacing: 2) {
                    Text(entry.label)
                        .font(.body)
                        .foregroundStyle(.black.opacity(entry.actionId == 0 ? 0.35 : 0.87))
                    if !entry.sublabel.isEmpty {
                        Text(entry.sublabel)
                            .font(.caption)
                            .foregroundStyle(.black.opacity(0.48))
                            .lineLimit(1)
                    }
                }
                Spacer()
                if !entry.value.isEmpty {
                    Text(entry.value)
                        .font(.caption)
                        .foregroundStyle(Color(red: 0.12, green: 0.47, blue: 0.74))
                        .lineLimit(1)
                }
            }
            .padding(16)
            .background(RoundedRectangle(cornerRadius: 3).fill(.white).shadow(color: .black.opacity(0.12), radius: 2, y: 1))
        }
        .buttonStyle(.plain)
        .disabled(entry.actionId == 0)
    }
}

private struct MenuEntryIcon: View {
    let kind: MenuEntryKind
    let family: String
    let selected: Bool

    var body: some View {
        Image(systemName: iconName)
            .font(.system(size: family == "rgui" ? 14 : 17, weight: .bold))
            .foregroundStyle(family == "material" ? tint : .black)
            .frame(width: iconSize, height: iconSize)
            .background(background)
    }

    private var iconSize: CGFloat { family == "ozone" ? 30 : 42 }

    @ViewBuilder
    private var background: some View {
        if family == "material" {
            Color.clear
        } else {
            Circle().fill(selected ? Color.white : tint)
        }
    }

    private var iconName: String {
        switch kind {
        case .action: return "play.fill"
        case .submenu: return family == "xmb" ? "square.grid.3x3.fill" : "folder.fill"
        case .toggle: return "switch.2"
        case .setting: return "slider.horizontal.3"
        }
    }

    private var tint: Color {
        switch kind {
        case .action: return .green
        case .submenu: return .cyan
        case .toggle: return .orange
        case .setting: return .purple
        }
    }
}

struct PlayView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack {
            ZStack {
                Color.black
                if let image = runtime.displayImage {
                    Image(uiImage: image)
                        .resizable()
                        .interpolation(.none)
                        .scaledToFit()
                } else {
                    VStack {
                        Image(systemName: "gamecontroller").font(.system(size: 50))
                        Text("No Video").font(.headline)
                    }.foregroundStyle(.white)
                }
            }
            .aspectRatio(runtime.aspectRatio, contentMode: .fit)
            .cornerRadius(12)
            .padding()

            Spacer()

            VirtualController()

            HStack(spacing: 40) {
                Button {
                    runtime.toggleRunning()
                } label: {
                    Image(systemName: runtime.isRunning ? "pause.circle.fill" : "play.circle.fill")
                        .font(.system(size: 60))
                }

                Button {
                    runtime.stop()
                    dismiss()
                } label: {
                    Image(systemName: "stop.circle.fill")
                        .font(.system(size: 60))
                        .foregroundStyle(.red)
                }
            }
            .padding(.bottom, 30)
        }
        .navigationTitle(runtime.loadedGameURL?.lastPathComponent ?? "Play")
        .background(Color.black)
        .onAppear {
            runtime.play()
        }
        .onDisappear {
            runtime.stop()
        }
    }
}

struct VirtualController: View {
    var body: some View {
        VStack {
            HStack {
                Dpad()
                Spacer()
                ActionButtons()
            }
            .padding(40)
        }
    }
}

struct Dpad: View {
    var body: some View {
        VStack(spacing: 5) {
            DPadButton(icon: "chevron.up", button: .up)
            HStack(spacing: 5) {
                DPadButton(icon: "chevron.left", button: .left)
                Circle().frame(width: 40, height: 40).opacity(0.1)
                DPadButton(icon: "chevron.right", button: .right)
            }
            DPadButton(icon: "chevron.down", button: .down)
        }
    }
}

struct DPadButton: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let icon: String
    let button: JoypadButton

    var body: some View {
        Button {
            // Visual feedback
        } label: {
            Image(systemName: icon)
                .frame(width: 44, height: 44)
                .background(Circle().fill(.white.opacity(0.1)))
        }
        .simultaneousGesture(
            DragGesture(minimumDistance: 0)
                .onChanged { _ in runtime.setJoypadButton(button, pressed: true) }
                .onEnded { _ in runtime.setJoypadButton(button, pressed: false) }
        )
    }
}

struct ActionButtons: View {
    var body: some View {
        VStack(spacing: 10) {
            HStack(spacing: 10) {
                ActionButton(label: "Y", color: .green, button: .y)
                ActionButton(label: "X", color: .blue, button: .x)
            }
            HStack(spacing: 10) {
                ActionButton(label: "B", color: .red, button: .b)
                ActionButton(label: "A", color: .yellow, button: .a)
            }
        }
    }
}

struct ActionButton: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    let label: String
    let color: Color
    let button: JoypadButton

    var body: some View {
        Button {
            // Visual feedback
        } label: {
            Text(label)
                .font(.headline)
                .frame(width: 50, height: 50)
                .background(Circle().fill(color.opacity(0.3)))
                .overlay(Circle().stroke(color, lineWidth: 2))
        }
        .simultaneousGesture(
            DragGesture(minimumDistance: 0)
                .onChanged { _ in runtime.setJoypadButton(button, pressed: true) }
                .onEnded { _ in runtime.setJoypadButton(button, pressed: false) }
        )
    }
}
