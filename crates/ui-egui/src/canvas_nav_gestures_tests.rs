//! The two navigation gestures of this fork, through the real canvas:
//!
//! - General â€º Double-Click Centres the View: double-clicking the canvas with the Hand puts the
//!   point clicked in the middle of the window, keeping the zoom;
//! - General â€º Right-Drag Zooms: holding the right button (a pen's barrel button comes through as
//!   one) and dragging vertically zooms about the point pressed â€” down zooms in, up zooms out.
//!
//! Both have a preference that turns them off, and a right-click that doesn't move must stay a
//! right-click.

use egui::{Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use serde_json::json;

use crate::PhotocraftApp;
use crate::canvas::ViewXform;
use crate::state::Tool;

const PHOTO: [u32; 2] = [2000, 1500];

fn app_with_photo() -> PhotocraftApp {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": PHOTO[0], "height": PHOTO[1], "background": "white"})).unwrap();
    app.sync_views();
    app
}

fn harness(app: PhotocraftApp) -> Harness<'static, PhotocraftApp> {
    // 50 ms a frame: a double-click is two clicks inside egui's 300 ms window, and the default
    // quarter-second step would put them past it.
    let mut h = Harness::builder().with_size(vec2(900.0, 600.0)).with_step_dt(0.05).build_ui_state(
        |ui, app: &mut PhotocraftApp| {
            let ctx = ui.ctx().clone();
            if !ctx.fonts(|f| f.families().contains(&egui::FontFamily::Name("medium".into()))) {
                return;
            }
            crate::shortcuts::handle(app, &ctx);
            egui::CentralPanel::default().show(ui, |ui| crate::canvas::document_area(app, ui));
        },
        app,
    );
    PhotocraftApp::setup_context(&h.ctx, crate::theme::ThemeKind::ALL[0]);
    h.run_steps(4);
    let v = &mut h.state_mut().ui.views[0];
    v.zoom = 1.0;
    v.center = [1000.0, 750.0];
    v.fit_pending = false;
    v.fill_pending = false;
    h.run_steps(2);
    h
}

fn centre(h: &Harness<'static, PhotocraftApp>) -> [f32; 2] {
    h.state().ui.views[0].center
}

fn zoom(h: &Harness<'static, PhotocraftApp>) -> f32 {
    h.state().ui.views[0].zoom
}

fn screen(h: &Harness<'static, PhotocraftApp>, x: f32, y: f32) -> Pos2 {
    let app = h.state();
    let v = &app.ui.views[0];
    let xf = ViewXform {
        rect: crate::rulers::content_rect(app, app.last_canvas_rect),
        zoom: v.zoom,
        center: v.center,
        flip: app.ui.view.flip_horizontal,
        rotation: v.rotation,
        aspect: app.ui.view.display_aspect(),
    };
    xf.to_screen(x, y)
}

fn press(h: &mut Harness<'static, PhotocraftApp>, p: Pos2, button: PointerButton, down: bool) {
    h.event(egui::Event::PointerButton { pos: p, button, pressed: down, modifiers: Modifiers::NONE });
    h.run_steps(1);
}

fn move_to(h: &mut Harness<'static, PhotocraftApp>, p: Pos2) {
    h.event(egui::Event::PointerMoved(p));
    h.run_steps(1);
}

/// Two primary clicks in a row at the same place: a double-click.
fn double_click(h: &mut Harness<'static, PhotocraftApp>, x: f32, y: f32) {
    let p = screen(h, x, y);
    move_to(h, p);
    for _ in 0..2 {
        press(h, p, PointerButton::Primary, true);
        press(h, p, PointerButton::Primary, false);
    }
    h.run_steps(2);
}

/// A right-button drag of `dy` points (down is positive) from the middle of the canvas.
fn right_drag(h: &mut Harness<'static, PhotocraftApp>, dy: f32) {
    let p = screen(h, 1000.0, 750.0);
    move_to(h, p);
    press(h, p, PointerButton::Secondary, true);
    let q = pos2_moved(p, dy);
    for i in 1..=8 {
        let t = i as f32 / 8.0;
        move_to(h, p + vec2(0.0, dy * t));
    }
    press(h, q, PointerButton::Secondary, false);
    h.run_steps(2);
}

fn pos2_moved(p: Pos2, dy: f32) -> Pos2 {
    p + vec2(0.0, dy)
}

fn set_pref(h: &mut Harness<'static, PhotocraftApp>, path: &str, value: bool) {
    h.state_mut().run("prefs.set", json!({"path": path, "value": value})).unwrap();
    h.run_steps(2);
}

#[test]
fn hand_double_click_centres_the_view_on_the_point_clicked() {
    let mut h = harness(app_with_photo());
    h.state_mut().ui.tool = Tool::Hand;
    h.run_steps(2);
    assert_eq!(centre(&h), [1000.0, 750.0]);
    // Inside the window: at 100 % the 868 x 542 canvas shows 434 x 271 points around the centre.
    double_click(&mut h, 700.0, 600.0);
    let c = centre(&h);
    assert!((c[0] - 700.0).abs() < 1.0 && (c[1] - 600.0).abs() < 1.0, "centred on the click: {c:?}");
    assert_eq!(zoom(&h), 1.0, "the zoom is untouched");
}

#[test]
fn hand_double_click_keeps_the_view_when_the_preference_is_off() {
    let mut h = harness(app_with_photo());
    set_pref(&mut h, "general.centerOnDoubleClick", false);
    h.state_mut().ui.tool = Tool::Hand;
    h.run_steps(2);
    double_click(&mut h, 700.0, 600.0);
    assert_eq!(centre(&h), [1000.0, 750.0], "the preference is off: nothing moves");
}

#[test]
fn right_drag_down_zooms_in_and_up_zooms_out() {
    let mut h = harness(app_with_photo());
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);
    right_drag(&mut h, 120.0);
    let z = zoom(&h);
    assert!(z > 1.4 && z < 1.9, "a drag down zooms in: {z}");
    // Back to 100 %, then a drag up: out.
    h.state_mut().ui.views[0].zoom = 1.0;
    h.state_mut().ui.views[0].center = [1000.0, 750.0];
    h.run_steps(2);
    right_drag(&mut h, -120.0);
    let z = zoom(&h);
    assert!(z < 0.75 && z > 0.5, "a drag up zooms out: {z}");
}

#[test]
fn the_zoomed_point_stays_under_the_press() {
    let mut h = harness(app_with_photo());
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);
    let p = screen(&h, 700.0, 500.0);
    move_to(&mut h, p);
    press(&mut h, p, PointerButton::Secondary, true);
    for i in 1..=8 {
        move_to(&mut h, p + vec2(0.0, 160.0 * (i as f32 / 8.0)));
    }
    let app = h.state();
    let v = &app.ui.views[0];
    let after = ViewXform {
        rect: crate::rulers::content_rect(app, app.last_canvas_rect),
        zoom: v.zoom,
        center: v.center,
        flip: app.ui.view.flip_horizontal,
        rotation: v.rotation,
        aspect: app.ui.view.display_aspect(),
    }
    .to_screen(700.0, 500.0);
    assert!((after - p).length() < 2.0, "the pressed point anchors the zoom: {after:?} vs {p:?}");
}

#[test]
fn a_right_click_that_barely_moves_does_not_zoom() {
    let mut h = harness(app_with_photo());
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);
    right_drag(&mut h, 8.0);
    assert_eq!(zoom(&h), 1.0, "inside a right-click's reach the button is still a click");
}

#[test]
fn right_drag_zooms_nothing_when_the_preference_is_off() {
    let mut h = harness(app_with_photo());
    set_pref(&mut h, "general.rightDragZooms", false);
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);
    right_drag(&mut h, 120.0);
    assert_eq!(zoom(&h), 1.0, "the preference is off: no zoom");
}

#[test]
fn the_hand_keeps_its_right_drag_pan() {
    let mut h = harness(app_with_photo());
    h.state_mut().ui.tool = Tool::Hand;
    h.run_steps(2);
    right_drag(&mut h, 120.0);
    assert_eq!(zoom(&h), 1.0, "the Hand's right drag pans, it does not zoom");
}

#[test]
fn drag_zoom_maps_the_drag_and_clamps_to_the_range() {
    let photo = PHOTO;
    assert!(crate::canvas::drag_zoom(1.0, 240.0, photo) > 2.7, "one e-fold down");
    assert!(crate::canvas::drag_zoom(1.0, -240.0, photo) < 0.37, "one e-fold up");
    assert_eq!(crate::canvas::drag_zoom(1.0, 0.0, photo), 1.0);
    assert_eq!(crate::canvas::drag_zoom(1.0, 1.0e6, photo), crate::zoom_levels::MAX, "clamped at the top");
    assert_eq!(crate::canvas::drag_zoom(1.0, -1.0e6, photo), crate::zoom_levels::min(photo), "clamped at the bottom");
    assert_eq!(crate::canvas::drag_zoom(f32::NAN, 10.0, photo), crate::canvas::drag_zoom(1.0, 10.0, photo), "a bad start is 100 %");
}
