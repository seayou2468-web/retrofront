import XCTest

@testable import RetrofrontSwift

final class RetrofrontTests: XCTestCase {
  func testInitialStateIsEmpty() throws {
    let frontend = try Retrofront()
    XCTAssertEqual(frontend.state, .empty)
  }
}
