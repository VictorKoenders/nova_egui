#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]

pub use egui;

pub struct NovaEguiCtx {
    raw_input: egui::RawInput,
    painter: egui_glow::Painter,
    context: egui::Context,
    modifiers: Modifiers,
    screen_size: [u32; 2],
    last_mouse_position: egui::Pos2,
}

#[derive(Default)]
#[expect(clippy::struct_excessive_bools)]
struct Modifiers {
    left_shift: bool,
    right_shift: bool,
    left_ctrl: bool,
    right_ctrl: bool,
    left_alt: bool,
    right_alt: bool,
    left_command: bool,
    right_command: bool,
}

impl Modifiers {
    fn key_down(&mut self, key: nova::input::Key) {
        match key {
            nova::input::Key::LeftShift => self.left_shift = true,
            nova::input::Key::RightShift => self.right_shift = true,
            nova::input::Key::LeftCtrl => self.left_ctrl = true,
            nova::input::Key::RightCtrl => self.right_ctrl = true,
            nova::input::Key::LeftAlt => self.left_alt = true,
            nova::input::Key::RightAlt => self.right_alt = true,
            nova::input::Key::LeftCommand => self.left_command = true,
            nova::input::Key::RightCommand => self.right_command = true,
            _ => {}
        }
    }

    fn key_up(&mut self, key: nova::input::Key) {
        match key {
            nova::input::Key::LeftShift => self.left_shift = false,
            nova::input::Key::RightShift => self.right_shift = false,
            nova::input::Key::LeftCtrl => self.left_ctrl = false,
            nova::input::Key::RightCtrl => self.right_ctrl = false,
            nova::input::Key::LeftAlt => self.left_alt = false,
            nova::input::Key::RightAlt => self.right_alt = false,
            nova::input::Key::LeftCommand => self.left_command = false,
            nova::input::Key::RightCommand => self.right_command = false,
            _ => {}
        }
    }

    const fn to_egui(&self) -> egui::Modifiers {
        egui::Modifiers {
            alt: self.left_alt || self.right_alt,
            ctrl: self.left_ctrl || self.right_ctrl,
            shift: self.left_shift || self.right_shift,
            mac_cmd: self.left_command || self.right_command,
            command: if cfg!(target_os = "macos") {
                self.left_command || self.right_command
            } else {
                self.left_ctrl || self.right_ctrl
            },
        }
    }
}

impl NovaEguiCtx {
    /// Create a new egui context.
    ///
    /// # Panics
    ///
    /// Panics if the `egui_glow` painter cannot be created.
    #[must_use]
    pub fn new(app: &nova::app::App) -> Self {
        let size = app.window.size();
        let mouse_position = app.input.mouse_position();
        Self {
            raw_input: egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(size.0 as f32, size.1 as f32),
                )),
                ..Default::default()
            },
            painter: egui_glow::Painter::new(
                std::sync::Arc::clone(unsafe { app.gfx.gl() }),
                "",
                None,
                false,
            )
            .expect("Could not create egui_glow painter"),
            context: egui::Context::default(),
            modifiers: Modifiers::default(),
            screen_size: size.into(),
            last_mouse_position: egui::Pos2::new(mouse_position.x, mouse_position.y),
        }
    }

    pub fn event(&mut self, event: &nova::input::Event) {
        // self.raw_input.raw_event(event);
        match event {
            nova::input::Event::KeyDown(key) => {
                self.modifiers.key_down(*key);

                if let Some(key) = key.to_egui_key() {
                    self.raw_input.events.push(egui::Event::Key {
                        key,
                        physical_key: Some(key),
                        repeat: false,
                        pressed: true,
                        modifiers: self.modifiers.to_egui(),
                    });
                }
            }
            nova::input::Event::KeyUp(key) => {
                self.modifiers.key_up(*key);
                if let Some(key) = key.to_egui_key() {
                    self.raw_input.events.push(egui::Event::Key {
                        key,
                        physical_key: Some(key),
                        repeat: false,
                        pressed: false,
                        modifiers: self.modifiers.to_egui(),
                    });
                }
            }
            nova::input::Event::TextInput { text } => {
                self.raw_input.events.push(egui::Event::Text(text.clone()));
            }
            nova::input::Event::MouseButtonDown(mouse_button) => {
                if let Some(button) = mouse_button.to_egui_button() {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.last_mouse_position,
                        button,
                        pressed: true,
                        modifiers: self.modifiers.to_egui(),
                    });
                }
            }
            nova::input::Event::MouseButtonUp(mouse_button) => {
                if let Some(button) = mouse_button.to_egui_button() {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.last_mouse_position,
                        button,
                        pressed: false,
                        modifiers: self.modifiers.to_egui(),
                    });
                }
            }
            nova::input::Event::MouseMotion { new_position } => {
                self.last_mouse_position = egui::Pos2::new(new_position.x, new_position.y);
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.last_mouse_position));
            }
            nova::input::Event::ControllerDeviceAdded { .. }
            | nova::input::Event::ControllerDeviceRemoved { .. }
            | nova::input::Event::ControllerButtonDown { .. }
            | nova::input::Event::ControllerButtonUp { .. }
            | nova::input::Event::ControllerAxisMotion { .. } => {}
            nova::input::Event::WindowResized { width, height } => {
                self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
                    egui::Pos2::default(),
                    egui::vec2(*width as f32, *height as f32),
                ));
                self.screen_size = [*width, *height];
            }
        }
    }

    pub fn update(&mut self, app: &nova::app::App) {
        self.raw_input.time = Some(app.timer.total_time().as_secs_f64());
        self.raw_input.modifiers = self.modifiers.to_egui();
    }

    pub fn render(
        &mut self,
        app: &mut nova::app::App,
        mut cb: impl FnMut(&mut nova::app::App, &egui::Context),
    ) {
        let result = self
            .context
            .run(std::mem::take(&mut self.raw_input), |ctx| cb(app, ctx));

        if let Some(url) = result.platform_output.open_url {
            if let Err(e) = webbrowser::open(&url.url) {
                eprintln!("Could not open URL {:?}; {e:?}", url.url);
            }
        }

        let clipped_primitives = self
            .context
            .tessellate(result.shapes, result.pixels_per_point);

        self.painter.paint_and_update_textures(
            self.screen_size,
            result.pixels_per_point,
            &clipped_primitives,
            &result.textures_delta,
        );

        app.gfx.rebind();
    }
}

trait KeyConversion: Sized {
    fn to_egui_key(&self) -> Option<egui::Key>;
}

impl KeyConversion for nova::input::Key {
    fn to_egui_key(&self) -> Option<egui::Key> {
        #[expect(clippy::match_same_arms)]
        Some(match self {
            Self::Space => egui::Key::Space,
            Self::Backspace => egui::Key::Backspace,
            Self::Enter => egui::Key::Enter,
            Self::Tab => egui::Key::Tab,
            Self::CapsLock => return None,
            Self::Escape => egui::Key::Escape,
            Self::LeftShift => return None,
            Self::RightShift => return None,
            Self::LeftCtrl => return None,
            Self::RightCtrl => return None,
            Self::LeftAlt => return None,
            Self::RightAlt => return None,
            Self::LeftCommand => return None,
            Self::RightCommand => return None,
            Self::Delete => egui::Key::Delete,
            Self::Up => egui::Key::ArrowUp,
            Self::Down => egui::Key::ArrowDown,
            Self::Left => egui::Key::ArrowLeft,
            Self::Right => egui::Key::ArrowRight,
            Self::A => egui::Key::A,
            Self::B => egui::Key::B,
            Self::C => egui::Key::C,
            Self::D => egui::Key::D,
            Self::E => egui::Key::E,
            Self::F => egui::Key::F,
            Self::G => egui::Key::G,
            Self::H => egui::Key::H,
            Self::I => egui::Key::I,
            Self::J => egui::Key::J,
            Self::K => egui::Key::K,
            Self::L => egui::Key::L,
            Self::M => egui::Key::M,
            Self::N => egui::Key::N,
            Self::O => egui::Key::O,
            Self::P => egui::Key::P,
            Self::Q => egui::Key::Q,
            Self::R => egui::Key::R,
            Self::S => egui::Key::S,
            Self::T => egui::Key::T,
            Self::U => egui::Key::U,
            Self::V => egui::Key::V,
            Self::W => egui::Key::W,
            Self::X => egui::Key::X,
            Self::Y => egui::Key::Y,
            Self::Z => egui::Key::Z,
            Self::Grave => egui::Key::Backtick,
            Self::Num0 => egui::Key::Num0,
            Self::Num1 => egui::Key::Num1,
            Self::Num2 => egui::Key::Num2,
            Self::Num3 => egui::Key::Num3,
            Self::Num4 => egui::Key::Num4,
            Self::Num5 => egui::Key::Num5,
            Self::Num6 => egui::Key::Num6,
            Self::Num7 => egui::Key::Num7,
            Self::Num8 => egui::Key::Num8,
            Self::Num9 => egui::Key::Num9,
            Self::Minus => egui::Key::Minus,
            Self::Equals => egui::Key::Equals,
            Self::F1 => egui::Key::F1,
            Self::F2 => egui::Key::F2,
            Self::F3 => egui::Key::F3,
            Self::F4 => egui::Key::F4,
            Self::F5 => egui::Key::F5,
            Self::F6 => egui::Key::F6,
            Self::F7 => egui::Key::F7,
            Self::F8 => egui::Key::F8,
            Self::F9 => egui::Key::F9,
            Self::F10 => egui::Key::F10,
            Self::F11 => egui::Key::F11,
            Self::F12 => egui::Key::F12,
        })
    }
}

trait MouseButtonConversion: Sized {
    fn to_egui_button(&self) -> Option<egui::PointerButton>;
}

impl MouseButtonConversion for nova::input::MouseButton {
    fn to_egui_button(&self) -> Option<egui::PointerButton> {
        Some(match self {
            Self::Left => egui::PointerButton::Primary,
            Self::Middle => egui::PointerButton::Middle,
            Self::Right => egui::PointerButton::Secondary,
            Self::X1 => egui::PointerButton::Extra1,
            Self::X2 => egui::PointerButton::Extra2,
        })
    }
}
