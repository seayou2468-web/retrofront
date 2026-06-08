import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers

struct DashboardView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

  var body: some View {
    TabView {
      LibraryView()
        .tabItem {
          Label("Library", systemImage: "books.vertical.fill")
        }

      CoresView()
        .tabItem {
          Label("Cores", systemImage: "cpu.fill")
        }

      SettingsView()
        .tabItem {
          Label("Settings", systemImage: "gearshape.fill")
        }
    }
  }
}

struct LibraryView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel
  @State private var isFilePickerPresented = false

  var body: some View {
    NavigationStack {
      List {
        Section("Quick Start") {
          Button {
            isFilePickerPresented = true
          } label: {
            Label("Open ROM", systemImage: "plus.circle.fill")
          }
        }

        Section("Current Status") {
          LabeledContent("Core", value: runtime.systemInfo?.libraryName ?? "None")
          LabeledContent("Game", value: runtime.loadedGameURL?.lastPathComponent ?? "None")

          if runtime.frontendState == .gameLoaded {
            NavigationLink("Go to Play") {
                PlayView()
            }
            .foregroundStyle(.blue)
          }
        }
      }
      .navigationTitle("Library")
      .fileImporter(isPresented: $isFilePickerPresented, allowedContentTypes: [.item]) { result in
        if case .success(let url) = result {
          runtime.loadGame(at: url)
        }
      }
    }
  }
}

struct CoresView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

  var body: some View {
    NavigationStack {
      List {
        Section("Available Cores") {
          if runtime.availableCores.isEmpty {
            Text("No cores found.")
              .foregroundStyle(.secondary)
          } else {
            ForEach(runtime.availableCores) { core in
              Button {
                runtime.loadCore(core)
              } label: {
                HStack {
                  VStack(alignment: .leading) {
                    Text(core.displayName)
                    Text(core.locationDescription).font(.caption).foregroundStyle(.secondary)
                  }
                  Spacer()
                  if runtime.coreURL == core.url {
                    Image(systemName: "checkmark.circle.fill")
                      .foregroundStyle(.green)
                  }
                }
              }
            }
          }
        }
      }
      .navigationTitle("Cores")
      .toolbar {
        Button("Refresh") {
            runtime.refreshAvailableCores()
        }
      }
    }
  }
}

struct PlayView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

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

      Spacer()

      HStack(spacing: 40) {
        Button {
          runtime.toggleRunning()
        } label: {
          Image(systemName: runtime.isRunning ? "pause.circle.fill" : "play.circle.fill")
            .font(.system(size: 60))
        }

        Button {
          runtime.stop()
        } label: {
          Image(systemName: "stop.circle.fill")
            .font(.system(size: 60))
            .foregroundStyle(.red)
        }
      }
      .padding(.bottom, 50)
    }
    .navigationTitle(runtime.loadedGameURL?.lastPathComponent ?? "Play")
    .onDisappear {
        runtime.stop()
    }
  }
}

struct SettingsView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

  var body: some View {
    NavigationStack {
      List {
        if let systemInfo = runtime.systemInfo {
            Section("Core: \(systemInfo.libraryName)") {
                if runtime.coreOptions.isEmpty {
                    Text("No options available.").foregroundStyle(.secondary)
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
        } else {
            Section("Core Options") {
                Text("Load a core to see its options.").foregroundStyle(.secondary)
            }
        }

        Section("App Info") {
            LabeledContent("Version", value: "0.1.0")
            LabeledContent("Backend", value: "bgfx")
        }
      }
      .navigationTitle("Settings")
    }
  }
}
