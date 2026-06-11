import XCTest

@testable import RetrofrontSwift

final class RetrofrontTests: XCTestCase {
  func testInitialStateIsEmpty() throws {
    let frontend = try Retrofront()
    XCTAssertEqual(frontend.state, .empty)
  }

  func testJoypadButtonStateCanBeUpdatedBeforeCoreLoad() throws {
    let frontend = try Retrofront()
    try frontend.setJoypadButton(.a, pressed: true)
    try frontend.setJoypadButton(.a, pressed: false)
  }

  func testRetroArchStyleSettingsAreExposed() throws {
    let frontend = try Retrofront()
    try frontend.setBaseDirectory("/tmp/RetrofrontTests")
    XCTAssertEqual(frontend.setting("libretro_directory"), "/tmp/RetrofrontTests/cores")
    XCTAssertTrue(frontend.settings().contains { $0.key == "core_options_path" })
  }

  func testContentLaunchPlanReportsMissingCore() throws {
    let frontend = try Retrofront()
    try frontend.setBaseDirectory("/tmp/RetrofrontTestsNoCores")
    let plan = frontend.planContentLaunch(path: "/tmp/game.gba")
    XCTAssertEqual(plan?.decision, .noCore)
    XCTAssertEqual(plan?.contentExtension, "gba")
  }

}
