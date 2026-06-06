import Foundation
import RetroFrontKit

#if os(Linux)
let runtime = RetroFrontRuntime(platform: .linux)
#elseif os(iOS) && !targetEnvironment(simulator)
let runtime = RetroFrontRuntime(platform: .iOSDevice)
#else
fatalError("RetroFront supports Linux and real iOS devices only; iOS Simulator is intentionally excluded.")
#endif

do {
    try runtime.validateRuntimePolicy()
    print("RetroFront Swift frontend ready: platform=\(runtime.platform.rawValue) menu=\(runtime.menu.selectedItem.rawValue)")
    print("Use the Rust host via Cargo binary or link libretrofront_core.a/cdylib into this Swift target for production UI rendering.")
} catch {
    FileHandle.standardError.write(Data("retrofront-swift: \(error)\n".utf8))
    exit(1)
}
