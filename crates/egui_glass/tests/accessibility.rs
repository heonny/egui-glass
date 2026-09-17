use egui::{accesskit, Context, Event, FullOutput, Key, Modifiers, RawInput, Response, Shape};
use egui_glass::GlassButton;

fn frame(ctx: &Context, events: Vec<Event>, enabled: bool) -> (FullOutput, Response) {
    let mut response = None;
    let output = ctx.run_ui(RawInput { events, ..Default::default() }, |ui| {
        ui.add_enabled_ui(enabled, |ui| {
            response = Some(GlassButton::new("Continue").show(ui));
        });
    });
    (output, response.unwrap())
}

fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }
}

#[test]
fn screen_reader_receives_button_role_label_and_actions() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let (output, response) = frame(&ctx, vec![], true);
    let update = output.platform_output.accesskit_update.unwrap();
    let (_, node) = update.nodes.iter().find(|(id, _)| *id == response.id.accesskit_id()).unwrap();
    assert_eq!(node.role(), accesskit::Role::Button);
    assert_eq!(node.label(), Some("Continue"));
    assert!(node.supports_action(accesskit::Action::Focus));
    assert!(node.supports_action(accesskit::Action::Click));
}

#[test]
fn keyboard_focus_has_a_visible_outline() {
    let ctx = Context::default();
    let (_, response) = frame(&ctx, vec![], true);
    response.request_focus();
    let (output, response) = frame(&ctx, vec![], true);
    assert!(response.has_focus());
    assert!(output.shapes.iter().any(|shape| matches!(
        &shape.shape,
        Shape::Rect(rect) if rect.rect == response.rect && rect.stroke.width > 0.0
    )), "focused glass button must paint an outline");
}

#[test]
fn focused_button_activates_with_enter_and_space() {
    for key_code in [Key::Enter, Key::Space] {
        let ctx = Context::default();
        let (_, response) = frame(&ctx, vec![], true);
        response.request_focus();
        let (_, response) = frame(&ctx, vec![key(key_code, Modifiers::NONE)], true);
        assert!(response.clicked(), "{key_code:?} should activate a focused button");
    }
}

#[test]
fn disabled_button_is_reported_and_cannot_activate() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let (_, response) = frame(&ctx, vec![], true);
    response.request_focus();
    let (output, response) = frame(&ctx, vec![key(Key::Enter, Modifiers::NONE)], false);
    assert!(!response.clicked());
    let update = output.platform_output.accesskit_update.unwrap();
    let (_, node) = update.nodes.iter().find(|(id, _)| *id == response.id.accesskit_id()).unwrap();
    assert!(node.is_disabled());
    assert_eq!(node.label(), Some("Continue"));
}

#[test]
fn icon_button_exposes_its_accessible_name() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut id = None;
    let output = ctx.run_ui(RawInput::default(), |ui| {
        id = Some(GlassButton::new("‹").icon().accessible_name("Back").show(ui).id);
    });
    let update = output.platform_output.accesskit_update.unwrap();
    let (_, node) = update.nodes.iter().find(|(node_id, _)| *node_id == id.unwrap().accesskit_id()).unwrap();
    assert_eq!(node.role(), accesskit::Role::Button);
    assert_eq!(node.label(), Some("Back"));
}

#[test]
fn assistive_click_activates_only_enabled_buttons() {
    for enabled in [true, false] {
        let ctx = Context::default();
        ctx.enable_accesskit();
        let (_, response) = frame(&ctx, vec![], enabled);
        let action = Event::AccessKitActionRequest(accesskit::ActionRequest {
            action: accesskit::Action::Click,
            target_tree: accesskit::TreeId::ROOT,
            target_node: response.id.accesskit_id(),
            data: None,
        });
        let (_, response) = frame(&ctx, vec![action], enabled);
        assert_eq!(response.clicked(), enabled);
    }
}

#[test]
fn tab_navigation_skips_disabled_buttons_and_reverses() {
    let ctx = Context::default();
    let mut ids = Vec::new();
    let mut run = |events: Vec<Event>, modifiers: Modifiers| {
        ids.clear();
        let _ = ctx.run_ui(RawInput { events, modifiers, ..Default::default() }, |ui| {
            ids.push(GlassButton::new("First").show(ui).id);
            ui.add_enabled_ui(false, |ui| {
                ids.push(GlassButton::new("Disabled").show(ui).id);
            });
            ids.push(GlassButton::new("Last").show(ui).id);
        });
        ids.clone()
    };
    let ids = run(vec![], Modifiers::NONE);
    ctx.memory_mut(|memory| memory.request_focus(ids[0]));
    run(vec![key(Key::Tab, Modifiers::NONE)], Modifiers::NONE);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(ids[2]));
    run(vec![key(Key::Tab, Modifiers::SHIFT)], Modifiers::SHIFT);
    // Backward focus is resolved against the previous pass's widget order.
    run(vec![], Modifiers::NONE);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(ids[0]));
}
