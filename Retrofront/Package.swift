// swift-tools-version: 6.1
import PackageDescription
import Foundation

let packageRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent().path
let rustReleasePath = "\(packageRoot)/target/release"

let package = Package(
  name: "Retrofront",
  platforms: [
    .iOS(.v18),
    .macOS(.v13),
  ],
  products: [
    .library(name: "RetrofrontSwift", targets: ["RetrofrontSwift"]),
    .executable(name: "retrofront-linux", targets: ["RetrofrontLinux"]),
  ],
  dependencies: [
    .package(url: "https://github.com/makoni/swift-adwaita.git", .upToNextMinor(from: "1.1.0")),
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
      name: "RetrofrontLinux",
      dependencies: ["RetrofrontSwift", .product(name: "Adwaita", package: "swift-adwaita")],
      path: "apps/linux/Sources"
    ),
    .testTarget(
      name: "RetrofrontSwiftTests",
      dependencies: ["RetrofrontSwift"],
      path: "frontend/Tests/RetrofrontSwiftTests"
    ),
  ]
)
