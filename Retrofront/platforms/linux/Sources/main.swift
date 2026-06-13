import Foundation

let home = URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
let root = home.appendingPathComponent(".local/share/retrofront/RetroArch", isDirectory: true)
try? FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)

let ok = root.path.withCString { retrofront_runtime_init($0) }
print("Retrofront Linux runtime: \(ok ? "ready" : "failed")")
retrofront_runtime_shutdown()
