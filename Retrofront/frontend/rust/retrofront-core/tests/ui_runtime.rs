use retrofront_core::{
    renderer::{CommandBuffer, DrawCommand},
    MenuDriver, UiRuntime,
};

#[test]
fn all_menu_drivers_have_navigable_mock_items() {
    for driver in MenuDriver::all() {
        let mut rt = UiRuntime::new(driver);
        rt.begin_frame(1920, 1080, 2.0);
        assert!(!rt.items().is_empty());
        rt.activate();
        assert!(rt.current_screen() != "");
        rt.end_frame();
    }
}

#[test]
fn command_buffer_validates_transform_balance() {
    let mut commands = CommandBuffer::new();
    commands.append(DrawCommand::FillRect {
        rect: [0.0, 0.0, 10.0, 10.0],
        color: [1.0; 4],
    });
    commands.append(DrawCommand::PushTransform([[0.0; 4]; 4]));
    commands.append(DrawCommand::PopTransform);
    assert!(commands.validate().is_ok());
}
