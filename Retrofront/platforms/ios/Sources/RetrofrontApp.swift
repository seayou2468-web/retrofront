import SwiftUI

@main
struct RetrofrontApp: App {
    var body: some Scene {
        WindowGroup {
            RetrofrontView()
        }
    }
}

struct RetrofrontView: View {
    @State private var status = "Starting Retrofront"

    var body: some View {
        VStack(spacing: 12) {
            Text("Retrofront")
                .font(.title)
            Text(status)
                .font(.footnote)
                .multilineTextAlignment(.center)
        }
        .padding()
        .onAppear(perform: boot)
    }

    private func boot() {
        let fm = FileManager.default
        let support = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let dataRoot = support.appendingPathComponent("RetroArch", isDirectory: true)
        _ = try? fm.createDirectory(at: dataRoot, withIntermediateDirectories: true)

        let ok = dataRoot.path.withCString { retrofront_runtime_init($0) }
        guard ok else {
            status = "Rust runtime init failed"
            return
        }

        if let zip = Bundle.main.url(forResource: "assets", withExtension: "zip") {
            let count = zip.path.withCString { retrofront_resources_unpack($0) }
            status = "Ready (assets: \(count), cores bundled for device)"
        } else {
            status = "Ready (assets.zip not found)"
        }
    }
}
