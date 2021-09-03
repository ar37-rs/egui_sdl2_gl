#![warn(clippy::all)]
#![allow(clippy::single_match)]

// Re-export dependencies.
pub use egui;
pub use gl;
pub use sdl2;

mod painter;

pub use painter::Painter;

use {
    egui::*,
    sdl2::{
        event::WindowEvent,
        keyboard::{Keycode, Mod},
        mouse::MouseButton,
        mouse::{Cursor, SystemCursor},
    },
};

#[cfg(not(feature = "clipboard"))]
mod clipboard;

use clipboard::{
    ClipboardContext, // TODO: remove
    ClipboardProvider,
};

pub struct FusedCursor {
    pub cursor: Cursor,
    pub icon: SystemCursor,
}

impl FusedCursor {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::from_system(SystemCursor::Arrow).unwrap(),
            icon: SystemCursor::Arrow,
        }
    }
}

pub struct EguiInputState {
    pub fused_cursor: FusedCursor,
    pub pointer_pos: Pos2,
    pub clipboard: Option<ClipboardContext>,
    pub input: RawInput,
    pub modifiers: Modifiers,
}

pub fn with_sdl2(window: &sdl2::video::Window) -> (Painter, EguiInputState) {
    let painter = painter::Painter::new(window);
    EguiInputState::new(painter)
}

impl EguiInputState {
    pub fn new(painter: Painter) -> (Painter, EguiInputState) {
        let _self = EguiInputState {
            fused_cursor: FusedCursor::new(),
            pointer_pos: Pos2::new(0f32, 0f32),
            clipboard: init_clipboard(),
            input: egui::RawInput {
                screen_rect: Some(painter.screen_rect),
                pixels_per_point: Some(painter.pixels_per_point),
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        };
        (painter, _self)
    }
}

pub fn input_to_egui(
    window: &sdl2::video::Window,
    event: sdl2::event::Event,
    painter: &mut Painter,
    state: &mut EguiInputState,
) {
    use sdl2::event::Event::*;

    let pixels_per_point = painter.pixels_per_point;
    match event {
        // handle when window Resized and SizeChanged.
        Window { win_event, .. } => match win_event {
            WindowEvent::Resized(_, _) | sdl2::event::WindowEvent::SizeChanged(_, _) => {
                if let Ok(display_dpi) = window.subsystem().display_dpi(0) {
                    painter.display_dpi = display_dpi.0
                }
                painter.update_screen_rect(window.drawable_size());
                state.input.screen_rect = Some(painter.screen_rect);
            }
            _ => (),
        },

        //MouseButonLeft pressed is the only one needed by egui
        MouseButtonDown { mouse_btn, .. } => {
            let mouse_btn = match mouse_btn {
                MouseButton::Left => Some(egui::PointerButton::Primary),
                MouseButton::Middle => Some(egui::PointerButton::Middle),
                MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            };
            if let Some(pressed) = mouse_btn {
                state.input.events.push(egui::Event::PointerButton {
                    pos: state.pointer_pos,
                    button: pressed,
                    pressed: true,
                    modifiers: state.modifiers,
                })
            }
        }

        //MouseButonLeft pressed is the only one needed by egui
        MouseButtonUp { mouse_btn, .. } => {
            let mouse_btn = match mouse_btn {
                MouseButton::Left => Some(egui::PointerButton::Primary),
                MouseButton::Middle => Some(egui::PointerButton::Middle),
                MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            };
            if let Some(pressed) = mouse_btn {
                state.input.events.push(egui::Event::PointerButton {
                    pos: state.pointer_pos,
                    button: pressed,
                    pressed: false,
                    modifiers: state.modifiers,
                })
            }
        }

        MouseMotion { x, y, .. } => {
            state.pointer_pos = pos2(x as f32 / pixels_per_point, y as f32 / pixels_per_point);
            state
                .input
                .events
                .push(egui::Event::PointerMoved(state.pointer_pos))
        }

        KeyUp {
            keycode, keymod, ..
        } => {
            if let Some(key) = translate_virtual_key_code(keycode.unwrap()) {
                state.modifiers = Modifiers {
                    alt: (keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                if state.modifiers.command && key == Key::C {
                    println!("copy event");
                    state.input.events.push(Event::Copy)
                } else if state.modifiers.command && key == Key::X {
                    println!("cut event");
                    state.input.events.push(Event::Cut)
                } else if state.modifiers.command && key == Key::V {
                    println!("paste");
                    if let Some(clipboard) = state.clipboard.as_mut() {
                        match clipboard.get_contents() {
                            Ok(contents) => {
                                state.input.events.push(Event::Text(contents));
                            }
                            Err(err) => {
                                eprintln!("Paste error: {}", err);
                            }
                        }
                    }
                } else {
                    state.input.events.push(Event::Key {
                        key,
                        pressed: false,
                        modifiers: state.modifiers,
                    });
                }
            }
        }

        KeyDown {
            keycode, keymod, ..
        } => {
            if let Some(key) = translate_virtual_key_code(keycode.unwrap()) {
                state.modifiers = Modifiers {
                    alt: (keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                state.input.events.push(Event::Key {
                    key,
                    pressed: true,
                    modifiers: state.modifiers,
                });
            }
        }

        TextInput { text, .. } => {
            state.input.events.push(Event::Text(text));
        }

        MouseWheel { x, y, .. } => {
            state.input.scroll_delta = vec2(x as f32, y as f32);
        }

        _ => {
            //dbg!(event);
        }
    }
}

pub fn translate_virtual_key_code(key: sdl2::keyboard::Keycode) -> Option<egui::Key> {
    use Keycode::*;

    Some(match key {
        Left => Key::ArrowLeft,
        Up => Key::ArrowUp,
        Right => Key::ArrowRight,
        Down => Key::ArrowDown,

        Escape => Key::Escape,
        Tab => Key::Tab,
        Backspace => Key::Backspace,
        Space => Key::Space,
        Return => Key::Enter,

        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,

        Kp0 | Num0 => Key::Num0,
        Kp1 | Num1 => Key::Num1,
        Kp2 | Num2 => Key::Num2,
        Kp3 | Num3 => Key::Num3,
        Kp4 | Num4 => Key::Num4,
        Kp5 | Num5 => Key::Num5,
        Kp6 | Num6 => Key::Num6,
        Kp7 | Num7 => Key::Num7,
        Kp8 | Num8 => Key::Num8,
        Kp9 | Num9 => Key::Num9,

        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,

        _ => {
            return None;
        }
    })
}

pub fn translate_cursor(fused: &mut FusedCursor, cursor_icon: egui::CursorIcon) {
    let tmp_icon = match cursor_icon {
        CursorIcon::Default => SystemCursor::Arrow,
        CursorIcon::PointingHand => SystemCursor::Hand,
        CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
        CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
        CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
        CursorIcon::ResizeVertical => SystemCursor::SizeNS,
        CursorIcon::Text => SystemCursor::IBeam,
        CursorIcon::Crosshair => SystemCursor::Crosshair,
        CursorIcon::NotAllowed | CursorIcon::NoDrop => SystemCursor::No,
        CursorIcon::Wait => SystemCursor::Wait,
        //There doesn't seem to be a suitable SDL equivalent...
        CursorIcon::Grab | CursorIcon::Grabbing => SystemCursor::Hand,

        _ => SystemCursor::Arrow,
    };

    if tmp_icon != fused.icon {
        fused.cursor = Cursor::from_system(tmp_icon).unwrap();
        fused.icon = tmp_icon;
        fused.cursor.set()
    }
}

pub fn init_clipboard() -> Option<ClipboardContext> {
    match ClipboardContext::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            eprintln!("Failed to initialize clipboard: {}", err);
            None
        }
    }
}

pub fn copy_to_clipboard(egui_state: &mut EguiInputState, copy_text: String) {
    if let Some(clipboard) = egui_state.clipboard.as_mut() {
        let result = clipboard.set_contents(copy_text);
        if result.is_err() {
            dbg!("Unable to set clipboard content.");
        }
    }
}
