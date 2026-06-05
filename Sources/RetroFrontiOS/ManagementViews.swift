#if canImport(SwiftUI)
import SwiftUI
import RetroFrontCore

struct SystemsView: View {
    @EnvironmentObject private var model: FrontendViewModel
    var body: some View {
        List {
            Button("All Systems") { model.selectedSystemID = nil }
            ForEach(ConsoleFamily.allCases) { family in
                let systems = SystemCatalog.systems.filter { $0.family == family }
                if !systems.isEmpty {
                    Section(family.displayName) {
                        ForEach(systems) { system in
                            Button { model.selectedSystemID = system.id } label: {
                                HStack { VStack(alignment: .leading) { Text(system.name); Text(system.fileExtensions.map { ".\($0)" }.joined(separator: ", ")).font(.caption).foregroundStyle(.secondary) }; Spacer(); Text("\(model.games.filter { $0.systemID == system.id }.count)").foregroundStyle(.secondary) }
                            }
                        }
                    }
                }
            }
        }.navigationTitle("Systems")
    }
}

struct CoresView: View {
    @EnvironmentObject private var model: FrontendViewModel
    var body: some View {
        List {
            Section("Installed cores") {
                ForEach(model.cores) { core in
                    NavigationLink { CoreDetailView(core: core) } label: { VStack(alignment: .leading) { Text(core.displayName); Text(core.supportedExtensions.map { ".\($0)" }.joined(separator: ", ")).font(.caption).foregroundStyle(.secondary) } }
                }
            }
            Section("Core management") {
                Label("Import bundled .dylib/.framework cores", systemImage: "square.and.arrow.down")
                Label("Per-core options and remaps", systemImage: "slider.horizontal.3")
                Label("BIOS audit", systemImage: "checkmark.shield")
            }
        }.navigationTitle("Cores").toolbar { Button("Rescan") { Task { await model.scanAll() } } }
    }
}

struct CoreDetailView: View {
    let core: LibretroCore
    var body: some View {
        List {
            Section {
                LabeledContent("Name", value: core.displayName)
                LabeledContent("Version", value: core.version ?? "Unknown")
                LabeledContent("Path", value: core.path.lastPathComponent)
                LabeledContent("Needs full path", value: core.requiresFullPath ? "Yes" : "No")
            }
            Section("Systems") {
                ForEach(core.systemIDs, id: \.self) { Text(SystemCatalog.system(id: $0)?.name ?? $0) }
            }
            Section("Options") {
                if core.options.isEmpty { Text("Core will expose runtime options through the libretro environment callback when available.") }
                else { ForEach(core.options) { Text($0.title) } }
            }
        }.navigationTitle(core.displayName)
    }
}

struct SavesView: View {
    @EnvironmentObject private var model: FrontendViewModel
    var body: some View {
        List {
            ForEach(model.saveStates) { state in
                VStack(alignment: .leading) { Text(model.games.first { $0.id == state.gameID }?.title ?? "Unknown game"); Text("Slot \(state.slot) · \(state.createdAt.formatted())").font(.caption).foregroundStyle(.secondary) }
            }
        }.navigationTitle("Save States").overlay { if model.saveStates.isEmpty { ContentUnavailableView("No save states", systemImage: "tray") } }
    }
}

struct CheatsView: View {
    let game: Game
    @State private var cheats: [CheatCode] = []
    @State private var name = ""
    @State private var code = ""
    var body: some View {
        List {
            Section("Add cheat") {
                TextField("Name", text: $name)
                TextField("Code", text: $code)
                Button("Add") {
                    cheats.append(CheatCode(gameID: game.id, name: name, code: code, enabled: true))
                    name = ""; code = ""
                }
            }
            Section("Codes") {
                ForEach($cheats) { $cheat in Toggle("\(cheat.name) — \(cheat.code)", isOn: $cheat.enabled) }
            }
        }.navigationTitle("Cheats")
    }
}

struct SettingsView: View {
    @EnvironmentObject private var model: FrontendViewModel
    @State private var draft = FrontendSettings()
    var body: some View {
        Form {
            Section("Video") { Picker("Shader", selection: $draft.shaderPreset) { Text("LCD + subtle CRT").tag("LCD + subtle CRT"); Text("Sharp pixels").tag("Sharp pixels"); Text("CRT Royale style").tag("CRT Royale style") }; Toggle("Integer scaling", isOn: $draft.integerScaling); Picker("Aspect ratio", selection: $draft.aspectRatio) { ForEach(AspectRatioMode.allCases) { Text($0.title).tag($0) } } }
            Section("Latency") { Toggle("Rewind", isOn: $draft.rewindEnabled); Stepper("Run-ahead frames: \(draft.runaheadFrames)", value: $draft.runaheadFrames, in: 0...4); Stepper("Fast-forward: \(draft.fastForwardRate, specifier: "%.1f")x", value: $draft.fastForwardRate, in: 1...8, step: 0.5) }
            Section("Cloud & accounts") { Toggle("iCloud sync library/saves/settings", isOn: $draft.iCloudSyncEnabled); TextField("RetroAchievements user", text: $draft.retroAchievementsUser).textInputAutocapitalization(.never); Toggle("Auto-save on background", isOn: $draft.autoSaveOnBackground) }
            Section("Input") { Toggle("Haptics", isOn: $draft.hapticsEnabled); Label("MFi / DualSense / Xbox / Joy-Con via GameController", systemImage: "gamecontroller") }
            Section { Button("Save Settings") { Task { await model.update(settings: draft) } } }
        }.navigationTitle("Settings").onAppear { draft = model.settings }
    }
}
#endif
