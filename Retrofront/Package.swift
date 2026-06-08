// swift-tools-version: 5.9
import PackageDescription
import Foundation

let packageRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path
let rustReleasePath = "\(packageRoot)/target/release"

let package = Package(
  name: "Retrofront",
  platforms: [
    .iOS(.v15),
    .macOS(.v13),
  ],
  products: [
    .library(name: "RetrofrontSwift", targets: ["RetrofrontSwift"]),
    .executable(name: "retrofront", targets: ["retrofront-cli"]),
    .executable(name: "retrofront-linux-ui", targets: ["retrofront-linux-ui"]),
  ],
  targets: [
    .target(
      name: "CRetrofrontCore",
      path: "frontend/CRetrofrontCore",
      publicHeadersPath: "."
    ),
    .target(
      name: "RetrofrontSwift",
      dependencies: ["CRetrofrontCore"],
      path: "frontend/Sources/RetrofrontSwift",
      linkerSettings: [
        .unsafeFlags(["-L", rustReleasePath, "-Xlinker", "-rpath", "-Xlinker", rustReleasePath]),
        .linkedLibrary("retrofront_core"),
      ]
    ),
    .executableTarget(
      name: "retrofront-cli",
      dependencies: ["RetrofrontSwift"],
      path: "frontend/Sources/retrofront-cli"
    ),
    .executableTarget(
      name: "retrofront-linux-ui",
      dependencies: ["RetrofrontSwift"],
      path: "frontend/Sources/retrofront-linux-ui"
    ),
    .testTarget(
      name: "RetrofrontSwiftTests",
      dependencies: ["RetrofrontSwift"],
      path: "frontend/Tests/RetrofrontSwiftTests"
    ),
  ]
)
