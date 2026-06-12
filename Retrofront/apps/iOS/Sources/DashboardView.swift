import SwiftUI
import RetrofrontSwift

struct DashboardView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel
    @State private var isRootMenuPresented = true
    @State private var isPlayViewActive = false

    var body: some View {
        RuntimeMenuScreen(isPresented: $isRootMenuPresented) {}
            .onChange(of: isRootMenuPresented) { _, presented in
                if !presented { isRootMenuPresented = true }
            }
            .fullScreenCover(isPresented: $isPlayViewActive) { PlayView() }
            .sheet(isPresented: Binding(get: { runtime.pendingContentURL != nil }, set: { if !$0 { runtime.cancelCoreChoice() } })) {
                CoreChoiceSheet()
            }
            .onReceive(runtime.$launchToken) { token in
                if token > 0 { isPlayViewActive = true }
            }
    }
}
