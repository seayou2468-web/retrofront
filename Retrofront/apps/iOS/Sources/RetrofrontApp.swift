import SwiftUI
import RetrofrontSwift

@main
struct RetrofrontApp: App {
    @StateObject private var runtime = EmulatorRuntimeModel()

    var body: some Scene {
        WindowGroup {
            DashboardView()
                .environmentObject(runtime)
                .preferredColorScheme(.dark)
        }
    }
}
