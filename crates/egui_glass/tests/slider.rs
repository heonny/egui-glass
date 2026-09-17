use egui::{accesskit, Context, Event, Key, Modifiers, RawInput, Response};
use egui_glass::GlassSlider;

fn frame(ctx: &Context, value: &mut f32, events: Vec<Event>, enabled: bool) -> (egui::FullOutput, Response) {
    let mut response = None;
    let output = ctx.run_ui(RawInput { events, ..Default::default() }, |ui| {
        ui.add_enabled_ui(enabled, |ui| {
            response = Some(GlassSlider::new(value, 0.0..=100.0).text("Volume").step_by(1.0).width(240.0).show(ui));
        });
    });
    (output, response.unwrap())
}

#[test]
fn exposes_a_named_slider_with_value_and_bounds() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let (output, _) = frame(&ctx, &mut 40.0, vec![], true);
    let update = output.platform_output.accesskit_update.unwrap();
    let (_, node) = update.nodes.iter().find(|(_, node)| node.role() == accesskit::Role::Slider).unwrap();
    assert_eq!(node.label(), Some("Volume"));
    assert_eq!(node.numeric_value(), Some(40.0));
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(100.0));
}

#[test]
fn arrow_key_changes_value_and_reports_changed() {
    let ctx = Context::default();
    let mut value = 40.0;
    let (_, response) = frame(&ctx, &mut value, vec![], true);
    response.request_focus();
    let key = Event::Key { key: Key::ArrowRight, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE };
    let (_, response) = frame(&ctx, &mut value, vec![key], true);
    assert_eq!(value, 41.0);
    assert!(response.changed());
}

#[test]
fn pointer_drag_changes_value_but_disabled_slider_does_not() {
    for enabled in [true, false] {
        let ctx = Context::default();
        let mut value = 40.0;
        let (_, response) = frame(&ctx, &mut value, vec![], enabled);
        let point = egui::pos2(response.rect.right() - 2.0, response.rect.bottom() - 16.0);
        let events = vec![Event::PointerMoved(point), Event::PointerButton {
            pos: point, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE,
        }];
        let (_, response) = frame(&ctx, &mut value, events, enabled);
        if enabled {
            assert!(value > 90.0);
            assert!(response.changed());
        } else {
            assert_eq!(value, 40.0);
            assert!(!response.changed());
        }
    }
}

#[test]
fn out_of_range_values_are_clamped() {
    let ctx = Context::default();
    let mut value = 150.0;
    frame(&ctx, &mut value, vec![], true);
    assert_eq!(value, 100.0);
}

#[test]
fn fixed_and_reversed_ranges_keep_the_thumb_inside_the_track() {
    for (range, mut value) in [(5.0..=5.0, 5.0), (100.0..=0.0, 25.0)] {
        let ctx = Context::default();
        let mut track = None;
        let output = ctx.run_ui(RawInput::default(), |ui| {
            track = Some(GlassSlider::new(&mut value, range.clone()).width(240.0).show(ui).rect);
        });
        let thumb = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Callback(callback) => Some(callback.rect),
            _ => None,
        }).expect("enabled slider paints a GPU glass thumb");
        assert!(thumb.is_finite());
        assert!(track.unwrap().contains(thumb.center()));
        if *range.start() == *range.end() { assert_eq!(value, 5.0); }
    }
}

#[test]
fn numeric_editor_updates_the_same_value() {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut value = 40.0;
    let (output, _) = frame(&ctx, &mut value, vec![], true);
    let update = output.platform_output.accesskit_update.unwrap();
    let (id, _) = update.nodes.iter().find(|(_, node)| node.role() == accesskit::Role::SpinButton).unwrap();
    let event = Event::AccessKitActionRequest(accesskit::ActionRequest {
        action: accesskit::Action::SetValue,
        target_tree: accesskit::TreeId::ROOT,
        target_node: *id,
        data: Some(accesskit::ActionData::NumericValue(72.0)),
    });
    let (_, response) = frame(&ctx, &mut value, vec![event], true);
    assert_eq!(value, 72.0);
    assert!(response.changed());
}
