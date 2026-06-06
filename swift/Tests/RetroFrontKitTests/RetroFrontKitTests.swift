import Testing
@testable import RetroFrontKit

@Test func menuWrapsBackward() {
    var menu = MenuModel()
    menu.moveSelection(by: -1)
    #expect(menu.selectedItem == .quit)
}

@Test func linuxRuntimePolicyIsAccepted() throws {
    let runtime = RetroFrontRuntime(platform: .linux)
    try runtime.validateRuntimePolicy()
}

@Test func controllerSkinHitTestMapsToJoypad() {
    let skin = ControllerSkin()
    let hits = skin.hitTest(x: 0.86, y: 0.72)
    #expect(hits.contains { $0.joypadID == .a })
}
