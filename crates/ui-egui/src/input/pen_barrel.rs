//! The pen's barrel button as the right button.
//!
//! On Windows a pen arrives as touch input: winit reports the contact as `WindowEvent::Touch` and
//! egui-winit maps it to the **left** button, so the pen's barrel button never becomes the
//! secondary click the canvas menus, the Brush Preset picker, right-erase and right-drag zoom
//! read (#11153). The barrel button is no mouse button at all: Windows Ink does not set
//! `VK_RBUTTON` for it and winit passes the pointer messages on untouched.
//!
//! The desktop app's pointer hook (`photocraft_tablet`) reports the barrel as a button: whether it
//! was down when a contact began, and presses and releases while the pen hovers. This module
//! turns those readings into ordinary secondary button events before egui sees the frame:
//!
//! - a contact that began with the barrel down is delivered as a right press, and its release as
//!   a right release, so every right-click path works exactly as it does for a mouse;
//! - a barrel press while hovering is a right click at the pointer (its release follows on the
//!   next frame, as egui wants a press and a release in two inputs);
//! - a barrel pressed *during* a contact keeps its existing meaning (brush resize, right-erase):
//!   that is no click, and converting it would erase a dab nowhere near where the stroke is.
//!
//! A real secondary button in the same frame (a mouse, or a driver that injects real right
//! clicks) wins and nothing is converted, so the same press is never counted twice.

/// The pen barrel, as the platform app's hook reports it, drained once per frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PenBarrelButton {
    /// Whether the barrel was down when the pen contact that has not been read yet began (`None`:
    /// no contact since the last read).
    pub contact: Option<bool>,
    /// A barrel press while the pen hovered.
    pub hover_pressed: bool,
    /// A barrel release while the pen hovered.
    pub hover_released: bool,
}

impl PenBarrelButton {
    /// Whether anything happened worth rewriting (`contact == Some(false)` is a plain contact).
    fn is_empty(&self) -> bool {
        self.contact.is_none() && !self.hover_pressed && !self.hover_released
    }
}

/// What [`BarrelClicks::rewrite`] needs to remember between frames.
#[derive(Clone, Debug, Default)]
pub(crate) struct BarrelClicks {
    /// The pen contact being delivered as the right button, until its release.
    converted: bool,
    /// A hover click whose press went out; its release follows next frame.
    hover_release: Option<egui::Pos2>,
}

impl BarrelClicks {
    /// Rewrite this frame's events: the pen's barrel becomes the right button. `pointer` is where
    /// the pen is (the frame's last pointer move), for a hover click.
    pub(crate) fn rewrite(&mut self, events: &mut Vec<egui::Event>, barrel: PenBarrelButton, pointer: Option<egui::Pos2>) {
        // The release half of a hover click, before anything else: egui takes a press and a
        // release in the same or in consecutive frames, never in one event.
        if let Some(pos) = self.hover_release.take() {
            events.push(button_event(pos, false));
        }
        if barrel.is_empty() && !self.converted {
            return;
        }
        // The OS's own right button (a mouse, or a driver injecting real right clicks) is the
        // click: converting the pen's contact as well would send two.
        let real_secondary = events.iter().any(|e| matches!(e, egui::Event::PointerButton { button: egui::PointerButton::Secondary, .. }));
        if barrel.contact == Some(true) && !real_secondary {
            if let Some(event) = events.iter_mut().find(|e| matches!(e, egui::Event::PointerButton { button: egui::PointerButton::Primary, pressed: true, .. })) {
                if let egui::Event::PointerButton { button, .. } = event {
                    *button = egui::PointerButton::Secondary;
                }
                self.converted = true;
            }
        }
        if self.converted {
            // The contact's release, so the right press is a click and not a stuck button.
            if let Some(event) = events.iter_mut().find(|e| matches!(e, egui::Event::PointerButton { button: egui::PointerButton::Primary, pressed: false, .. })) {
                if let egui::Event::PointerButton { button, .. } = event {
                    *button = egui::PointerButton::Secondary;
                }
                self.converted = false;
            }
        }
        // A barrel press while hovering: a right click wherever the pen is.
        if barrel.hover_pressed && !real_secondary && !self.converted {
            if let Some(pos) = pointer {
                events.push(button_event(pos, true));
                self.hover_release = Some(pos);
            }
        }
        if barrel.hover_released && self.hover_release.is_some() {
            self.hover_release = None;
            events.push(button_event(pointer.unwrap_or_default(), false));
        }
    }

    /// A pen contact is being delivered as the right button.
    #[cfg(test)]
    pub(crate) fn converts(&self) -> bool {
        self.converted
    }
}

fn button_event(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton { pos, button: egui::PointerButton::Secondary, pressed, modifiers: egui::Modifiers::default() }
}

#[cfg(test)]
mod tests;
