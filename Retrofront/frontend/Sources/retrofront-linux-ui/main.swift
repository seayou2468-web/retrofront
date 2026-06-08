import Foundation
import RetrofrontSwift

struct TerminalDashboard {
  let frontend: Retrofront

  func render() {
    print("""
    ┌────────────────────────────────────────────┐
    │ Retrofront Linux                           │
    │ Empty frontend UI connected to Rust runtime │
    └────────────────────────────────────────────┘
    """)
    print("Runtime state : \(frontend.state)")
    print("Emulator core : not loaded")
    print("Game content  : not loaded")
    print("Next step     : pass a libretro core to future loader UI")
  }
}

do {
  let frontend = try Retrofront()
  TerminalDashboard(frontend: frontend).render()
} catch {
  fputs("Retrofront Linux UI failed to start: \(error)\n", stderr)
  exit(1)
}
