# Learnings from Retrofront UI/Core Overhaul

- Ported RetroArch's core discovery and metadata management logic to Rust.
- Implemented a flexible Menu Engine in Rust that drives a modern SwiftUI frontend.
- Standardized directory management for cores, info files, and games.
- Expanded the C ABI to support complex data structures (CoreInfo, MenuList, GameEntry) for Swift consumption.
- Used a centralized 'with_active_frontend' pattern in Rust to handle libretro callbacks that need access to the frontend state.
