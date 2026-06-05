import Foundation

public struct DynamicCoreManager: Sendable {
    public init() {}

    public func installPlan(for source: URL, directories: FrontendDirectories, appStoreDistribution: Bool = false) -> CoreInstallPlan {
        let destination = directories.cores.appendingPathComponent(source.lastPathComponent)
        let extensionName = source.pathExtension.lowercased()
        var notes: [String] = []
        var status: CoreInstallStatus = .importedSigned

        if appStoreDistribution {
            status = .unavailableOnAppStore
            notes.append("iOS App Store builds cannot execute newly downloaded code; ship cores in the app bundle or use an approved signed update.")
        } else if !["dylib", "framework", "so"].contains(extensionName) {
            status = .missingEntitlement
            notes.append("Only libretro dynamic modules are accepted by the loader.")
        } else if !looksCodeSigned(source) {
            status = .importedUnsigned
            notes.append("The file does not expose a detectable code signature marker; iOS will reject dlopen unless the module is signed for this app.")
        } else {
            notes.append("Import is eligible for dlopen on developer/sideloaded builds when the signature and entitlements match the host app.")
        }

        return CoreInstallPlan(source: source, destination: destination, displayName: source.deletingPathExtension().lastPathComponent, status: status, notes: notes)
    }

    public func installCore(from source: URL, directories: FrontendDirectories, replacingExisting: Bool = true) throws -> CoreInstallPlan {
        let plan = installPlan(for: source, directories: directories)
        try FileManager.default.createDirectory(at: directories.cores, withIntermediateDirectories: true)
        if replacingExisting && FileManager.default.fileExists(atPath: plan.destination.path) {
            try FileManager.default.removeItem(at: plan.destination)
        }
        try FileManager.default.copyItem(at: source, to: plan.destination)
        return plan
    }

    private func looksCodeSigned(_ url: URL) -> Bool {
        guard let handle = try? FileHandle(forReadingFrom: url) else { return false }
        defer { try? handle.close() }
        let sample = (try? handle.read(upToCount: 2_000_000)) ?? Data()
        return sample.withUnsafeBytes { raw in
            guard let base = raw.baseAddress else { return false }
            let bytes = base.assumingMemoryBound(to: UInt8.self)
            let markers = ["LC_CODE_SIGNATURE", "CodeResources", "embedded.mobileprovision", "com.apple.cs"]
            return markers.contains { marker in
                let needle = Array(marker.utf8)
                guard !needle.isEmpty, sample.count >= needle.count else { return false }
                for offset in 0...(sample.count - needle.count) {
                    var matched = true
                    for index in 0..<needle.count where bytes[offset + index] != needle[index] { matched = false; break }
                    if matched { return true }
                }
                return false
            }
        }
    }
}

public enum FrontendFeatureMatrix {
    public static let capabilities: [FrontendCapability] = [
        FrontendCapability(id: "library", title: "ROM library, playlists, artwork, CRC scanning", detail: "Local scanner imports ROMs, LPL playlists and side-loaded artwork.", state: .complete),
        FrontendCapability(id: "dynamic-cores", title: "Signed libretro dynamic core loading", detail: "The C host uses dlopen/dlsym for libretro cores and the Swift manager imports user-supplied signed modules into Documents/Cores.", state: .complete),
        FrontendCapability(id: "menu-engines", title: "Native and RetroArch menu engine selection", detail: "SwiftUI exposes a native frontend and records Ozone/XMB/RGUI/MaterialUI preferences for builds that link the full RetroArch renderer.", state: .platformGated),
        FrontendCapability(id: "states", title: "Save states", detail: "Serialization and unserialization are available for cores that implement libretro state support.", state: .complete),
        FrontendCapability(id: "ios-jit", title: "JIT/dynarec cores", detail: "JIT availability is controlled by iOS signing and external enablers; the frontend detects and reports this platform gate rather than bypassing iOS security.", state: .platformGated),
        FrontendCapability(id: "netplay-achievements", title: "Netplay and RetroAchievements", detail: "Account and room models are present; real service integration requires network credentials and service-specific APIs.", state: .externalService)
    ]
}
