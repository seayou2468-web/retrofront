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
}
