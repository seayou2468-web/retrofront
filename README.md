# RetroFront

RetroFront is an iOS-first SwiftUI frontend for bundled libretro cores. It provides the application shell expected from a modern multi-emulator frontend: library import/scanning, system-aware game organization, core management, save states, cheats, touch/controller play UI, video/latency settings, BIOS/artwork directories, and a C bridge that hosts libretro-compatible dynamic cores.

## Features

- **Game library**: ROM import, recursive scanning, CRC32 identification, favorites, search, system shelves, artwork matching, playlists, and per-system filtering.
- **Core management**: scans bundled/imported `.dylib`, `.framework`, and `.so` libretro cores; parses `.info` metadata; maps cores to supported systems and extensions.
- **iOS play screen**: SwiftUI player shell with pause menu, touch overlay, controller-state abstraction, fast-forward toggle, reset, shader/aspect-ratio menu entries, netplay and RetroAchievements entry points.
- **Persistence**: JSON-backed library, cores, settings, cheats, and save-state metadata under the app Documents directory.
- **libretro bridge**: `CLibretroHost` loads a core, checks the libretro API version, wires video/audio/input/environment callbacks, runs frames, and serializes save states using the bundled `libretro.h` API.
- **Frontend services**: ROM/core scanner, BIOS audit, local artwork attachment, RetroArch playlist parser, core info parser, and test coverage for core utilities.

## Project layout

```text
Sources/CLibretroHost       C libretro host bridge using Externals/libretro-common/include/libretro.h
Sources/RetroFrontCore      Models, catalog, scanning, persistence, parsers, libretro runtime wrapper
Sources/RetroFrontiOS       SwiftUI iOS app shell and player/frontend UI
Tests/RetroFrontCoreTests   Unit tests for catalog, CRC32, playlist parsing, and core info parsing
```

## Build and test

```sh
swift test
```

On macOS, open the Swift package in Xcode and run the `RetroFrontiOS` app target on an iOS 17+ device/simulator. Add ROMs to `Documents/ROMs`, BIOS files to `Documents/System`, artwork to `Documents/Artwork`, and bundled libretro cores to `Documents/Cores` or import them via the app.

## Legal note

RetroFront does not include commercial games or BIOS files. Users must provide legally obtained content and firmware. RetroFront is not affiliated with console manufacturers, RetroArch, or Libretro.
