use super::*;

fn pos() -> egui::Pos2 {
    egui::pos2(40.0, 20.0)
}

/// The events egui receives for a pen contact: winit's touch, and the emulated pointer press.
fn contact_start() -> Vec<egui::Event> {
    vec![
        egui::Event::Touch { device_id: egui::TouchDeviceId(1), id: egui::TouchId(7), phase: egui::TouchPhase::Start, pos: pos(), force: Some(0.5) },
        button_event(egui::PointerButton::Primary, true),
    ]
}

fn contact_end() -> Vec<egui::Event> {
    vec![
        egui::Event::Touch { device_id: egui::TouchDeviceId(1), id: egui::TouchId(7), phase: egui::TouchPhase::End, pos: pos(), force: None },
        button_event(egui::PointerButton::Primary, false),
    ]
}

fn button_event(button: egui::PointerButton, pressed: bool) -> egui::Event {
    egui::Event::PointerButton { pos: pos(), button, pressed, modifiers: egui::Modifiers::default() }
}

fn buttons(events: &[egui::Event]) -> Vec<(egui::PointerButton, bool)> {
    events
        .iter()
        .filter_map(|e| match e {
            egui::Event::PointerButton { button, pressed, .. } => Some((*button, *pressed)),
            _ => None,
        })
        .collect()
}

fn barrel(contact: Option<bool>) -> PenBarrelButton {
    PenBarrelButton { contact, hover_pressed: false, hover_released: false }
}

#[test]
fn a_barrel_contact_is_a_right_press_and_release() {
    let mut rewrite = BarrelClicks::default();
    let mut events = contact_start();
    rewrite.rewrite(&mut events, barrel(Some(true)), Some(pos()));
    assert_eq!(buttons(&events), [(egui::PointerButton::Secondary, true)], "the barrel contact is the right button");
    assert!(rewrite.converts());
    let mut events = contact_end();
    rewrite.rewrite(&mut events, barrel(None), Some(pos()));
    assert_eq!(buttons(&events), [(egui::PointerButton::Secondary, false)], "and its release, so it is a click");
    assert!(!rewrite.converts());
}

#[test]
fn a_contact_without_the_barrel_stays_the_left_button() {
    let mut rewrite = BarrelClicks::default();
    let mut events = contact_start();
    rewrite.rewrite(&mut events, barrel(Some(false)), Some(pos()));
    assert_eq!(buttons(&events), [(egui::PointerButton::Primary, true)]);
    assert!(!rewrite.converts());
}

#[test]
fn a_real_right_click_in_the_frame_is_not_doubled() {
    let mut rewrite = BarrelClicks::default();
    let mut events = contact_start();
    events.push(button_event(egui::PointerButton::Secondary, true));
    rewrite.rewrite(&mut events, barrel(Some(true)), Some(pos()));
    let secondary: Vec<_> = buttons(&events).into_iter().filter(|(b, _)| *b == egui::PointerButton::Secondary).collect();
    assert_eq!(secondary, [(egui::PointerButton::Secondary, true)], "the OS's own right button is the click");
    assert!(!rewrite.converts());
}

#[test]
fn the_release_is_converted_in_a_later_frame() {
    let mut rewrite = BarrelClicks::default();
    let mut events = contact_start();
    rewrite.rewrite(&mut events, barrel(Some(true)), Some(pos()));
    let mut moved = vec![egui::Event::PointerMoved(pos())];
    rewrite.rewrite(&mut moved, PenBarrelButton::default(), Some(pos()));
    assert_eq!(buttons(&moved), [], "a move frame has no button events");
    let mut events = contact_end();
    rewrite.rewrite(&mut events, barrel(None), Some(pos()));
    assert_eq!(buttons(&events), [(egui::PointerButton::Secondary, false)]);
}

#[test]
fn a_hover_press_is_a_right_click_over_two_frames() {
    let mut rewrite = BarrelClicks::default();
    let mut events = vec![egui::Event::PointerMoved(pos())];
    rewrite.rewrite(&mut events, PenBarrelButton { hover_pressed: true, ..Default::default() }, Some(pos()));
    assert_eq!(buttons(&events), [(egui::PointerButton::Secondary, true)]);
    let mut events = vec![egui::Event::PointerMoved(pos())];
    rewrite.rewrite(&mut events, PenBarrelButton::default(), Some(pos()));
    assert_eq!(buttons(&events), [(egui::PointerButton::Secondary, false)], "the release follows next frame");
    let mut events = vec![egui::Event::PointerMoved(pos())];
    rewrite.rewrite(&mut events, PenBarrelButton::default(), Some(pos()));
    assert_eq!(buttons(&events), [], "and it is released once");
}

#[test]
fn a_hover_click_needs_the_pointer_position() {
    let mut rewrite = BarrelClicks::default();
    let mut events = Vec::new();
    rewrite.rewrite(&mut events, PenBarrelButton { hover_pressed: true, ..Default::default() }, None);
    assert_eq!(buttons(&events), [], "no click without a position to click at");
}

#[test]
fn a_barrel_pressed_during_a_contact_is_no_click() {
    // The barrel level changed, but no contact began with it: the existing modifiers keep it.
    let mut rewrite = BarrelClicks::default();
    let mut events = vec![egui::Event::PointerMoved(pos())];
    rewrite.rewrite(&mut events, PenBarrelButton::default(), Some(pos()));
    assert_eq!(buttons(&events), []);
    assert!(!rewrite.converts());
}
