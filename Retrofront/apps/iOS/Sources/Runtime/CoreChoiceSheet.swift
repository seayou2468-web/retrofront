import SwiftUI
import RetrofrontSwift

struct NowPlayingView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Binding var isPlayViewActive: Bool

    var body: some View {
        AppScreen(title: "Play", subtitle: "現在のゲームセッション") {
            VStack(alignment: .leading, spacing: 18) {
                ZStack {
                    RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous)
                        .fill(Color.black)
                        .aspectRatio(16.0 / 10.0, contentMode: .fit)
                    if let image = runtime.displayImage {
                        Image(uiImage: image)
                            .resizable()
                            .interpolation(.none)
                            .scaledToFit()
                            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
                    } else {
                        VStack(spacing: 10) {
                            Image(systemName: "display")
                                .font(.system(size: 34, weight: .semibold))
                            Text(runtime.loadedGameURL == nil ? "No game loaded" : "Ready to render")
                                .font(.headline)
                        }
                        .foregroundColor(.white.opacity(0.78))
                    }
                }

                VStack(alignment: .leading, spacing: 6) {
                    Text(runtime.loadedGameURL?.lastPathComponent ?? "ゲーム未選択")
                        .font(.title3.bold())
                        .foregroundColor(OneUI.ink)
                    Text(runtime.systemInfo?.libraryName ?? "Libraryからゲームを選択すると起動します。")
                        .font(.subheadline)
                        .foregroundColor(OneUI.secondary)
                }

                Button {
                    if runtime.loadedGameURL != nil { isPlayViewActive = true }
                } label: {
                    Label(runtime.loadedGameURL == nil ? "Select a game from Library" : "Open Player", systemImage: "play.circle.fill")
                        .font(.headline)
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 15)
                        .background(Capsule().fill(runtime.loadedGameURL == nil ? OneUI.muted : OneUI.accent))
                }
                .buttonStyle(.plain)
                .disabled(runtime.loadedGameURL == nil)
            }
            .padding(18)
            .background(OneUI.surface)
            .clipShape(RoundedRectangle(cornerRadius: OneUI.radius, style: .continuous))
            .shadow(color: .black.opacity(0.05), radius: 18, y: 10)
        }
    }
}

struct CoreChoiceSheet: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @Environment(\.presentationMode) private var presentationMode

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 12) {
                    ForEach(runtime.pendingCoreChoices, id: \.path) { core in
                        Button {
                            runtime.launchPendingContent(with: core)
                            presentationMode.wrappedValue.dismiss()
                        } label: {
                            CoreRow(core: core)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(18)
            }
            .background(OneUI.background)
            .navigationBarTitle("Select Core", displayMode: .inline)
            .navigationBarItems(leading: Button("Cancel") { runtime.cancelCoreChoice() })
        }
        .navigationViewStyle(.stack)
    }
}

