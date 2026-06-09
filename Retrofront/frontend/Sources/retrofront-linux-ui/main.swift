import Foundation
import RetrofrontSwift

struct TerminalDashboard {
  let frontend: Retrofront

  func render() {
    print("""
    One UI Dark
    ───────────
    State       \(frontend.state)
    Library     ROMs only
    Core        choose after ROM when needed
    Play screen portrait / landscape auto
    Quick menu  core settings, display, controls, states
    """)

    if let menu = frontend.currentMenuList() {
      print("\n\(menu.title)")
      for entry in menu.entries {
        let value = entry.value.isEmpty ? "" : "  \(entry.value)"
        print("• \(entry.label) — \(entry.sublabel)\(value)")
      }
    }
  }
}

do {
  let frontend = try Retrofront()
  TerminalDashboard(frontend: frontend).render()
} catch {
  fputs("UI failed to start: \(error)\n", stderr)
  exit(1)
}
