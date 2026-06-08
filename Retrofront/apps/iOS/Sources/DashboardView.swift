import RetrofrontSwift
import SwiftUI
import UniformTypeIdentifiers

struct DashboardView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

  var body: some View {
    TabView(selection: $runtime.selectedTab) {
      LibraryView()
        .tabItem { Label(AppSection.library.rawValue, systemImage: AppSection.library.symbolName) }
        .tag(AppSection.library)

      PlaySurfaceView()
        .tabItem { Label(AppSection.play.rawValue, systemImage: AppSection.play.symbolName) }
        .tag(AppSection.play)

      CoreStatusView()
        .tabItem { Label(AppSection.cores.rawValue, systemImage: AppSection.cores.symbolName) }
        .tag(AppSection.cores)

      SettingsView()
        .tabItem { Label(AppSection.settings.rawValue, systemImage: AppSection.settings.symbolName) }
        .tag(AppSection.settings)
    }
    .tint(.orange)
  }
}

struct LibraryView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel
  @State private var isImporterPresented = false

  var body: some View {
    NavigationStack {
      ScrollView {
        VStack(alignment: .leading, spacing: 20) {
          HeroCard(
            title: "Retrofront",
            subtitle: runtime.statusMessage,
            systemImage: runtime.isRuntimeConnected ? "checkmark.seal.fill" : "exclamationmark.triangle.fill"
          )

          SectionHeader(title: "ROM Library", subtitle: "Choose a ROM from Files. Retrofront copies it into Documents, loads the bundled mGBA libretro core, then opens the Play tab.")

          Button {
            isImporterPresented = true
          } label: {
            Label("Choose ROM", systemImage: "folder.badge.plus")
              .font(.headline)
              .frame(maxWidth: .infinity)
              .padding()
          }
          .buttonStyle(.borderedProminent)
          .disabled(!runtime.isRuntimeConnected)

          ContentStatusCard(
            icon: runtime.canRunGame ? "gamecontroller.fill" : "rectangle.stack.badge.plus",
            title: runtime.loadedGameName,
            message: runtime.canRunGame ? "Ready to run on the Play tab." : "No ROM is loaded yet."
          )
        }
        .padding(20)
      }
      .background(AppTheme.background)
      .navigationTitle("Retrofront")
      .toolbar {
        Button("Refresh") { runtime.refresh() }
      }
      .fileImporter(
        isPresented: $isImporterPresented,
        allowedContentTypes: [.data, .item],
        allowsMultipleSelection: false
      ) { result in
        switch result {
        case .success(let urls):
          if let url = urls.first {
            runtime.importROM(from: url)
          }
        case .failure(let error):
          runtime.statusMessage = "ROM picker failed: \(error.localizedDescription)"
        }
      }
    }
  }
}

struct PlaySurfaceView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

  var body: some View {
    NavigationStack {
      VStack(spacing: 18) {
        ZStack {
          RoundedRectangle(cornerRadius: 28, style: .continuous)
            .fill(.black.gradient)

          if let image = runtime.displayImage {
            Image(uiImage: image)
              .resizable()
              .interpolation(.none)
              .scaledToFit()
              .clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
              .padding(8)
          } else {
            VStack(spacing: 12) {
              Image(systemName: "display")
                .font(.system(size: 54, weight: .semibold))
                .foregroundStyle(.orange)
              Text("Core video output")
                .font(.headline)
              Text(runtime.canRunGame ? "Press Play to run frames." : "Choose a ROM from Library.")
                .foregroundStyle(.secondary)
            }
          }
        }
        .aspectRatio(4 / 3, contentMode: .fit)
        .shadow(radius: 18)

        if let frame = runtime.latestFrame {
          Text("Latest frame: \(frame.width)×\(frame.height), #\(frame.frameNumber)")
            .font(.footnote.monospacedDigit())
            .foregroundStyle(.secondary)
        }

        HStack(spacing: 14) {
          Button { runtime.stop() } label: {
            ControlPill(title: "Pause", symbol: "pause.fill")
          }
          .disabled(!runtime.isRunning)

          Button { runtime.toggleRunning() } label: {
            ControlPill(title: runtime.isRunning ? "Running" : "Play", symbol: runtime.isRunning ? "stop.fill" : "play.fill")
          }
          .disabled(!runtime.canRunGame)

          Button { runtime.runOneFrameFromButton() } label: {
            ControlPill(title: "Frame", symbol: "forward.frame.fill")
          }
          .disabled(!runtime.canRunGame)
        }

        VirtualGamepadView()
          .disabled(!runtime.canRunGame)
          .opacity(runtime.canRunGame ? 1 : 0.45)

        Spacer()
      }
      .padding(20)
      .background(AppTheme.background)
      .navigationTitle(runtime.loadedGameName)
    }
  }
}

struct CoreStatusView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

  var body: some View {
    NavigationStack {
      List {
        Section("Runtime") {
          LabeledContent("Rust frontend core", value: runtime.isRuntimeConnected ? "Connected" : "Unavailable")
          LabeledContent("Session", value: String(describing: runtime.frontendState))
          LabeledContent("Bundled dylib", value: runtime.coreURL?.lastPathComponent ?? "Missing")
          LabeledContent("Loaded ROM", value: runtime.loadedGameName)
        }
        Section("libretro") {
          LabeledContent("Core", value: runtime.systemInfo?.libraryName ?? "Not loaded")
          LabeledContent("Version", value: runtime.systemInfo?.libraryVersion ?? "—")
          LabeledContent("Extensions", value: runtime.systemInfo?.validExtensions.joined(separator: ", ") ?? "—")
          LabeledContent("Needs full path", value: runtime.systemInfo?.needsFullPath == true ? "Yes" : "No")
        }
        Section("Video") {
          LabeledContent("Frame", value: runtime.latestFrame.map { "\($0.width)×\($0.height) #\($0.frameNumber)" } ?? "—")
          LabeledContent("Renderer", value: "Software RGBA → SwiftUI Image")
        }
      }
      .navigationTitle("Cores")
      .toolbar { Button("Refresh") { runtime.refresh() } }
    }
  }
}

struct SettingsView: View {
  var body: some View {
    NavigationStack {
      List {
        Section("Input") {
          Toggle("Virtual gamepad overlay", isOn: .constant(true))
          Toggle("Haptics", isOn: .constant(true))
        }
        Section("Video") {
          Toggle("Integer scaling", isOn: .constant(false))
          Toggle("Low-latency mode", isOn: .constant(true))
        }
      }
      .navigationTitle("Settings")
    }
  }
}

private struct VirtualGamepadView: View {
  var body: some View {
    HStack(alignment: .center) {
      DPadView()
      Spacer(minLength: 24)
      VStack(spacing: 14) {
        HStack(spacing: 14) {
          GamepadButton(title: "Y", button: .y)
          GamepadButton(title: "X", button: .x)
        }
        HStack(spacing: 14) {
          GamepadButton(title: "B", button: .b)
          GamepadButton(title: "A", button: .a)
        }
        HStack(spacing: 12) {
          GamepadButton(title: "Select", button: .select, compact: true)
          GamepadButton(title: "Start", button: .start, compact: true)
        }
      }
    }
  }
}

private struct DPadView: View {
  var body: some View {
    Grid(horizontalSpacing: 8, verticalSpacing: 8) {
      GridRow {
        Color.clear.frame(width: 48, height: 48)
        GamepadButton(title: "▲", button: .up)
        Color.clear.frame(width: 48, height: 48)
      }
      GridRow {
        GamepadButton(title: "◀", button: .left)
        Color.clear.frame(width: 48, height: 48)
        GamepadButton(title: "▶", button: .right)
      }
      GridRow {
        Color.clear.frame(width: 48, height: 48)
        GamepadButton(title: "▼", button: .down)
        Color.clear.frame(width: 48, height: 48)
      }
    }
  }
}

private struct GamepadButton: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel
  let title: String
  let button: JoypadButton
  var compact = false

  var body: some View {
    Text(title)
      .font(compact ? .footnote.bold() : .headline.bold())
      .frame(width: compact ? 74 : 48, height: 48)
      .background(.thinMaterial, in: Capsule())
      .simultaneousGesture(
        DragGesture(minimumDistance: 0)
          .onChanged { _ in runtime.setButton(button, pressed: true) }
          .onEnded { _ in runtime.setButton(button, pressed: false) }
      )
  }
}

private struct HeroCard: View {
  let title: String
  let subtitle: String
  let systemImage: String

  var body: some View {
    VStack(alignment: .leading, spacing: 16) {
      Image(systemName: systemImage)
        .font(.system(size: 36, weight: .bold))
        .foregroundStyle(.orange)
      Text(title)
        .font(.largeTitle.bold())
      Text(subtitle)
        .font(.body)
        .foregroundStyle(.secondary)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(24)
    .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 28, style: .continuous))
  }
}

private struct SectionHeader: View {
  let title: String
  let subtitle: String

  var body: some View {
    VStack(alignment: .leading, spacing: 6) {
      Text(title).font(.title2.bold())
      Text(subtitle).foregroundStyle(.secondary)
    }
  }
}

private struct ContentStatusCard: View {
  let icon: String
  let title: String
  let message: String

  var body: some View {
    VStack(spacing: 12) {
      Image(systemName: icon).font(.system(size: 42)).foregroundStyle(.orange)
      Text(title).font(.headline)
      Text(message).multilineTextAlignment(.center).foregroundStyle(.secondary)
    }
    .frame(maxWidth: .infinity)
    .padding(28)
    .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 24, style: .continuous))
  }
}

private struct ControlPill: View {
  let title: String
  let symbol: String

  var body: some View {
    Label(title, systemImage: symbol)
      .font(.headline)
      .padding(.horizontal, 18)
      .padding(.vertical, 12)
      .background(.thinMaterial, in: Capsule())
  }
}

private enum AppTheme {
  static let background = LinearGradient(
    colors: [Color(.systemBackground), Color.orange.opacity(0.12)],
    startPoint: .top,
    endPoint: .bottom
  )
}
