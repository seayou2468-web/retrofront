// swift-tools-version: 5.9
import PackageDescription
import Foundation

let package = Package(
  name: "Retrofront",
  platforms: [
    .iOS(.v17),
    .macOS(.v13),
  ],
  products: [
    .library(
      name: "RetrofrontSwift",
      targets: ["RetrofrontSwift"]
    ),
    .executable(
      name: "retrofront",
      targets: ["retrofront-cli"]
    ),
  ],
  targets: [

    // =========================
    // C Bridge Layer (Rust ↔ Swift)
    // =========================
    .target(
      name: "CRetrofrontCore",
      path: "apps/frontend/CRetrofrontCore",
      publicHeadersPath: "."
    ),

    // =========================
    // Swift core (minimal dependency layer)
    // =========================
    .target(
      name: "RetrofrontSwift",
      dependencies: ["CRetrofrontCore"],
      path: "apps/frontend/Sources/RetrofrontSwift",
      linkerSettings: [
        // ❌ unsafeFlags完全排除（SwiftPMの最適化を壊すため）
        .linkedLibrary("retrofront_core")
      ]
    ),

    // =========================
    // CLI (Swift依存を最小化)
    // =========================
    .executableTarget(
      name: "retrofront-cli",
      dependencies: ["CRetrofrontCore"],
      path: "apps/frontend/Sources/retrofront-cli"
    ),

    // =========================
    // Tests
    // =========================
    .testTarget(
      name: "RetrofrontSwiftTests",
      dependencies: ["RetrofrontSwift"],
      path: "apps/frontend/Tests/RetrofrontSwiftTests"
    ),
  ]
)
