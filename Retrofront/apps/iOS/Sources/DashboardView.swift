import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers

struct DashboardView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel
  @State private var selectedTab = 0
  @State private var isPlayViewActive = false

  var body: some View {
    ZStack {
        Color(white: 0.05).ignoresSafeArea()

        TabView(selection: $selectedTab) {
          ModernHomeView(isPlayViewActive: $isPlayViewActive)
            .tabItem {
              Label("Home", systemImage: "house.fill")
            }.tag(0)

          ModernLibraryView()
            .tabItem {
              Label("Library", systemImage: "gamecontroller.fill")
            }.tag(1)

          ModernSettingsView()
            .tabItem {
              Label("Settings", systemImage: "gearshape.2.fill")
            }.tag(2)
        }
        .accentColor(.blue)
    }
    .fullScreenCover(isPresented: $isPlayViewActive) {
        PlayView()
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
    @State private var isFilePickerPresented = false

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
            .fileImporter(isPresented: $isFilePickerPresented, allowedContentTypes: [.item]) { result in
                if case .success(let url) = result {
                    runtime.importGame(at: url)
                }
            }
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
            .background(Color(white: 0.15))
            .cornerRadius(16)
            .overlay(
                RoundedRectangle(cornerRadius: 16)
                    .stroke(color.opacity(0.3), lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
    }
}

struct ModernLibraryView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var isFilePickerPresented = false

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
                        Label("Add Content", systemImage: "doc.badge.plus")
                    }
                }
            }
            .navigationTitle("Library")
            .background(Color(white: 0.05))
            .scrollContentBackground(.hidden)
            .fileImporter(isPresented: $isFilePickerPresented, allowedContentTypes: [.item]) { result in
                if case .success(let url) = result {
                    runtime.importGame(at: url)
                }
            }
        }
    }
}

struct ModernSettingsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        NavigationStack {
            List {
                if let menu = runtime.currentMenu {
                    Section(menu.title) {
                        ForEach(menu.entries, id: \.actionId) { entry in
                            Button {
                                // runtime.menuAction(entry.actionId)
                            } label: {
                                HStack {
                                    VStack(alignment: .leading) {
                                        Text(entry.label)
                                            .foregroundStyle(.primary)
                                        if !entry.sublabel.isEmpty {
                                            Text(entry.sublabel).font(.caption).foregroundStyle(.secondary)
                                        }
                                    }
                                    Spacer()
                                    if !entry.value.isEmpty {
                                        Text(entry.value).foregroundStyle(.blue)
                                    }
                                    if entry.kind == .submenu {
                                        Image(systemName: "chevron.right").font(.caption).foregroundStyle(.secondary)
                                    }
                                }
                            }
                        }
                    }
                }

                Section("Core Options") {
                    if runtime.coreOptions.isEmpty {
                        Text("No core options available").foregroundStyle(.secondary)
                    } else {
                        ForEach(runtime.coreOptions, id: \.key) { option in
                            Picker(option.desc, selection: Binding(
                                get: { option.value },
                                set: { runtime.setOption(key: option.key, value: $0) }
                            )) {
                                ForEach(option.values, id: \.value) { val in
                                    Text(val.label).tag(val.value)
                                }
                            }
                        }
                    }
                }

                Section("App Info") {
                    LabeledContent("Version", value: "0.1.0")
                    LabeledContent("Engine", value: "Rust/libretro")
                }
            }
            .navigationTitle("Settings")
            .background(Color(white: 0.05))
            .scrollContentBackground(.hidden)
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
      .aspectRatio(4/3, contentMode: .fit)
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
