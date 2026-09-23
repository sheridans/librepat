use eframe::egui::{Context, Event, Id, RawInput, text_edit::TextEditState};

use super::*;

fn command_a_input() -> RawInput {
    let modifiers = Modifiers {
        ctrl: true,
        command: true,
        ..Modifiers::default()
    };
    RawInput {
        events: vec![Event::Key {
            key: Key::A,
            physical_key: Some(Key::A),
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..RawInput::default()
    }
}

#[test]
fn select_visible_shortcut_should_accept_command_a() {
    let context = Context::default();

    let output = context.run_ui(command_a_input(), |ui| {
        assert!(select_visible_shortcut(ui.ctx()));
    });
    output.drop_without_applying_deltas();
}

#[test]
fn select_visible_shortcut_should_not_override_text_edit() {
    let context = Context::default();
    let text_edit_id = Id::new("focused-text-edit");
    TextEditState::default().store(&context, text_edit_id);
    context.memory_mut(|memory| memory.request_focus(text_edit_id));

    let output = context.run_ui(command_a_input(), |ui| {
        assert!(!select_visible_shortcut(ui.ctx()));
    });
    output.drop_without_applying_deltas();
}
