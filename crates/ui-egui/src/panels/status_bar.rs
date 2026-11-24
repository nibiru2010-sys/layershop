use super::{PhotocraftApp, RichText, Tokens, label, vec2, widgets};

// ----------------------------------------------------------------------------- status bar

/// The status bar's height and the height of the controls in it (zoom field, Fit), which leaves
/// them an even margin above and below inside the bar (#2823).
pub fn status_bar_heights(t: &Tokens) -> (f32, f32) {
    if t.pro { (24.0, 18.0) } else { (30.0, 22.0) }
}

pub fn status_bar(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom("status_bar")
        .exact_size(status_bar_heights(&t).0)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(10, 0)))
        .show(ui, |ui| {
            let control_h = status_bar_heights(&t).1;
            // Read from the right in a right-to-left UI, with the trailing group at the left.
            crate::direction::horizontal_centered(ui, |ui| {
                if t.pro {
                    crate::chrome_ui::status_bar_pro(app, ui);
                    // Background job progress with Cancel, at the far end (#210).
                    ui.with_layout(crate::direction::trailing(), |ui| {
                        crate::jobs_ui::status_progress(app, ui);
                        crate::background_redraw::status(app, ui);
                        crate::chrome_ui::render_path_status(app, ui);
                        // A local AI setup running in the background (File › Set Up Local AI…).
                        crate::ai_setup_ui::status_progress(app, ui);
                    });
                    return;
                }
                if let Some(st) = app.session.active() {
                    let (w, h, mode, bits, layers) =
                        (st.doc.size.width, st.doc.size.height, crate::canvas::mode_label(&st.doc), st.doc.depth.bits(), st.doc.layer_count());
                    widgets::vline(ui, 16.0);
                    label(ui, &crate::i18n::fmt(tl!("{mode} Color · {bits} bit"), &[("mode", mode), ("bits", &bits.to_string())]));
                    widgets::vline(ui, 16.0);
                    label(ui, &format!("{w} × {h} px"));
                    widgets::vline(ui, 16.0);
                    label(ui, &crate::i18n::trn(crate::i18n::current(), layers as u64, "{n} layer", "{n} layers"));
                } else {
                    label(ui, tl!("No document"));
                }
                if !app.ui.status.is_empty() {
                    widgets::vline(ui, 16.0);
                    let is_err = app.ui.status_error || app.ui.status.starts_with("Couldn");
                    ui.label(RichText::new(&app.ui.status).color(if is_err { t.warning } else { t.text_faint }));
                }
                // The zoom field sits in the middle of the bar when the labels leave it room, and
                // right after them when they don't (a long status message, a narrow window): it
                // never overlaps them either way.
                if let Some(i) = app.session.active_index() {
                    let mut pct = app.ui.views[i].zoom * 100.0;
                    const ZOOM_FIELD_W: f32 = 78.0;
                    let centre = ui.max_rect().center().x;
                    let room = if crate::direction::is_rtl() {
                        ui.min_rect().left() - (centre + ZOOM_FIELD_W / 2.0)
                    } else {
                        centre - ZOOM_FIELD_W / 2.0 - ui.min_rect().right()
                    };
                    if room > 0.0 {
                        ui.add_space(room);
                    }
                    let field = widgets::compact_wrapping_value_field(ui, &mut pct, crate::zoom_levels::percent_range(&app.ui.views[i]), "%", vec2(ZOOM_FIELD_W, control_h))
                        .accessible_name(tl!("Zoom"));
                    if field.changed() {
                        let animated = app.session.prefs().general.animated_zoom;
                        crate::zoom_levels::set_from_field(&mut app.ui.views[i], field.dragged() || field.drag_started(), pct, animated);
                    }
                }
                ui.with_layout(crate::direction::trailing(), |ui| {
                    if app.session.active().is_some()
                        && !t.pro
                        && widgets::compact_button(ui, tl!("Fit"), control_h).clicked()
                        && let Some(i) = app.session.active_index()
                    {
                        app.ui.views[i].fit_pending = true;
                    }
                    if app.session.has_jobs() {
                        ui.add_space(6.0);
                        crate::jobs_ui::status_progress(app, ui);
                    }
                    crate::background_redraw::status(app, ui);
                    crate::chrome_ui::render_path_status(app, ui);
                    crate::ai_setup_ui::status_progress(app, ui);
                });
            });
        });
}
