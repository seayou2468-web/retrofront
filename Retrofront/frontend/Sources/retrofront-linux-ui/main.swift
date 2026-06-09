import Foundation
import RetrofrontSwift

struct LinuxOneUI {
    let frontend: Retrofront

    func start() {
        print("\n\u{1B}[1m--- RETROFRONT (OneUI Linux) ---\u{1B}[0m")
        print("Status: \(frontend.state)")
        print("\n[1] Library")
        print("[2] Settings")
        print("[Q] Quit")

        // Mocking interaction for CLI
        while let input = readLine()?.lowercased() {
            switch input {
            case "1": showLibrary()
            case "2": showSettings()
            case "q": return
            default: print("Unknown command")
            }
        }
    }

    func showLibrary() {
        print("\n\u{1B}[1mLIBRARY\u{1B}[0m")
        let games = frontend.availableGames()
        if games.isEmpty {
            print("No games found. Check your 'roms' directory.")
        } else {
            for (i, game) in games.enumerated() {
                print("[\(i)] \(game.label)")
            }
        }
    }

    func showSettings() {
        print("\n\u{1B}[1mSETTINGS\u{1B}[0m")
        print("[E] Extract Assets")
        print("[B] Back")
        if let choice = readLine()?.lowercased(), choice == "e" {
            _ = frontend.activateMenuAction(21) // ACTION_EXTRACT_ASSETS
            print("Extraction requested.")
        }
    }
}

do {
    let frontend = try Retrofront()
    let ui = LinuxOneUI(frontend: frontend)
    ui.start()
} catch {
    print("Failed to start Retrofront: \(error)")
}
