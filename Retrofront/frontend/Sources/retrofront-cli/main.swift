import Foundation
import RetrofrontSwift

let args = CommandLine.arguments
if args.count < 2 {
  print("Usage: retrofront <libretro-core> [game]")
  exit(64)
}

do {
  let frontend = try Retrofront()
  let info = try frontend.loadCore(at: args[1])
  print("Loaded core: \(info.libraryName) \(info.libraryVersion)")
  if args.count >= 3 {
    try frontend.loadGame(at: args[2])
    let events = try frontend.runFrame()
    print("Ran one frame; events=\(events.count)")
  }
} catch {
  fputs("retrofront: \(error)\n", stderr)
  exit(1)
}
