import SwiftUI

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

  var body: some View {
    NavigationStack {
      ScrollView {
        VStack(alignment: .leading, spacing: 20) {
          HeroCard(
            title: "Retrofront",
            subtitle: runtime.statusMessage,
            systemImage: runtime.isRuntimeConnected ? "checkmark.seal.fill" : "exclamationmark.triangle.fill"
          )

          SectionHeader(title: "Library", subtitle: "No ROMs are loaded in this empty app shell.")
          EmptyStateCard(
            icon: "rectangle.stack.badge.plus",
            title: "Add content later",
            message: "The iOS UI is wired to the frontend runtime, but intentionally ships without emulator cores or games."
          )
        }
        .padding(20)
      }
      .background(AppTheme.background)
      .navigationTitle("Retrofront")
      .toolbar {
        Button("Refresh") { runtime.refresh() }
      }
    }
  }
}

struct PlaySurfaceView: View {
  @EnvironmentObject private var runtime: EmulatorRuntimeModel

  var body: some View {
    NavigationStack {
      VStack(spacing: 18) {
        RoundedRectangle(cornerRadius: 28, style: .continuous)
          .fill(.black.gradient)
          .overlay {
            VStack(spacing: 12) {
              Image(systemName: "display")
                .font(.system(size: 54, weight: .semibold))
                .foregroundStyle(.orange)
              Text("Video output surface")
                .font(.headline)
              Text("Waiting for a game-loaded libretro core.")
                .foregroundStyle(.secondary)
            }
          }
          .aspectRatio(16 / 9, contentMode: .fit)
          .shadow(radius: 18)

        HStack(spacing: 14) {
          ControlPill(title: "Menu", symbol: "line.3.horizontal")
          ControlPill(title: runtime.canRunGame ? "Run" : "Idle", symbol: "play.fill")
          ControlPill(title: "Save", symbol: "square.and.arrow.down")
        }
        Spacer()
      }
      .padding(20)
      .background(AppTheme.background)
      .navigationTitle("Play")
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
          LabeledContent("Emulator core", value: runtime.systemInfo?.libraryName ?? "Not loaded")
        }
        Section("libretro") {
          LabeledContent("Version", value: runtime.systemInfo?.libraryVersion ?? "—")
          LabeledContent("Extensions", value: runtime.systemInfo?.validExtensions.joined(separator: ", ") ?? "—")
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

private struct EmptyStateCard: View {
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
