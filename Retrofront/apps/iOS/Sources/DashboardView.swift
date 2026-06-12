import SwiftUI
import RetrofrontSwift
import UniformTypeIdentifiers
import UIKit

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var selectedTab = 0
    @State private var isPlayViewActive = false
    @State private var isImporterPresented = false

    var body: some View {
        TabView(selection: $selectedTab) {
            LibraryView {
                isImporterPresented = true
            }
                .tabItem { Label("ライブラリ", systemImage: "rectangle.stack.fill") }
                .tag(0)

            PlaylistDashboardView()
                .tabItem { Label("プレイリスト", systemImage: "list.bullet.rectangle.fill") }
                .tag(1)

            CoreDashboardView()
                .tabItem { Label("コア", systemImage: "cpu.fill") }
                .tag(2)

            SettingsView()
                .tabItem { Label("設定", systemImage: "gearshape.fill") }
                .tag(3)
        }
        .accentColor(OneUI.accent)
        .preferredColorScheme(.dark)
        .toolbarBackground(OneUI.surface, for: .tabBar)
        .toolbarBackground(.visible, for: .tabBar)
        .background(OneUI.background.ignoresSafeArea())
        .fullScreenCover(isPresented: $isPlayViewActive) { PlayView() }
        .fileImporter(isPresented: $isImporterPresented, allowedContentTypes: [.data]) { result in
            switch result {
            case .success(let url):
                runtime.importFile(at: url)
            case .failure(let error):
                runtime.statusMessage = "Import failed: \(error)"
            }
        }
        .sheet(isPresented: Binding(get: { runtime.pendingContentURL != nil }, set: { if !$0 { runtime.cancelCoreChoice() } })) {
            CoreChoiceSheet()
        }
        .onReceive(runtime.$launchToken) { token in
            if token > 0 { isPlayViewActive = true }
        }
    }
}

