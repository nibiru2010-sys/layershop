//! The canvas's right-drag zoom (General > Right-Drag Zooms), through the real canvas:
//!
//! Holding the right button and dragging vertically zooms about the point pressed: down zooms in,
//! up zooms out. The preference turns it off, a right-click that doesn't move must stay a
//! right-click, and the Hand keeps its right-drag pan.

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
    // 50 ms a frame: a drag's steps stay under the click distance and the reach below, where the
    // default quarter-second step makes every gesture look slow.
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

/// A right-button drag of `dy` points (down is positive) from the middle of the canvas.
fn right_drag(h: &mut Harness<'static, PhotocraftApp>, dy: f32) {
    let p = screen(h, 1000.0, 750.0);
    move_to(h, p);
    press(h, p, PointerButton::Secondary, true);
    for i in 1..=8 {
        move_to(h, p + vec2(0.0, dy * (i as f32 / 8.0)));
    }
    press(h, p + vec2(0.0, dy), PointerButton::Secondary, false);
    h.run_steps(2);
}

fn set_pref(h: &mut Harness<'static, PhotocraftApp>, path: &str, value: bool) {
    h.state_mut().run("prefs.set", json!({"path": path, "value": value})).unwrap();
    h.run_steps(2);
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
