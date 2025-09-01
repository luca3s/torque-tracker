use winit::keyboard::{Key, NamedKey};

use crate::{
    app::{EventQueue, GlobalEvent, PlaybackType},
    coordinates::{CharPosition, CharRect, FONT_SIZE, PixelRect},
    draw_buffer::DrawBuffer,
    ui::pages::PagesEnum,
};

use super::{Dialog, DialogResponse};

enum Action {
    Menu(Menu),
    // TODO: maybe fold this into the more general event variant
    Page(PagesEnum),
    Event(GlobalEvent),
    // TODO: should be removed when it's all implemented
    NotYetImplemented,
}

// Main missing, because it cant be opened from a menu
enum Menu {
    File,
    Playback,
    Sample,
    Instrument,
    Settings,
}

pub struct PageMenu {
    name: &'static str,
    rect: CharRect,
    selected: usize,
    pressed: bool,
    buttons: &'static [(&'static str, Action)],
    sub_menu: Option<Box<Self>>,
}

impl Dialog for PageMenu {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        let top_left = self.rect.top_left();
        let top = top_left.y();
        let left = top_left.x();

        draw_buffer.draw_rect(Self::BACKGROUND_COLOR, self.rect);
        draw_buffer.draw_string(self.name, top_left + CharPosition::new(7, 2), 3, 2);
        // self.draw_outer_box(draw_buffer);
        draw_buffer.draw_out_border(self.rect, Self::TOPLEFT_COLOR, Self::TOPLEFT_COLOR, 2);
        draw_buffer.draw_out_border(
            CharRect::new(
                self.rect.top() + 1,
                self.rect.bot() - 1,
                self.rect.left() + 1,
                self.rect.right() - 1,
            ),
            Self::BOTRIGHT_COLOR,
            Self::BOTRIGHT_COLOR,
            1,
        );
        for (num, (name, _)) in self.buttons.iter().enumerate() {
            let text_color = match self.selected == num {
                true => 11,
                false => 0,
            };

            draw_buffer.draw_string(
                name,
                top_left + CharPosition::new(4, (3 * num) + 5),
                text_color,
                Self::BACKGROUND_COLOR,
            );
            let top = top + (3 * num) + 4;
            let (top_left, bot_right) =
                match (self.pressed || self.sub_menu.is_some()) && self.selected == num {
                    true => (Self::BOTRIGHT_COLOR, Self::TOPLEFT_COLOR),
                    false => (Self::TOPLEFT_COLOR, Self::BOTRIGHT_COLOR),
                };
            let rect = CharRect::new(top, top + 2, left + 2, left + self.rect.width() - 2);
            draw_buffer.draw_out_border(rect, top_left, bot_right, 1);
            Self::draw_button_corners(rect, draw_buffer);
        }
        if let Some(sub) = self.sub_menu.as_ref() {
            sub.draw(draw_buffer);
        }
    }

    fn process_input(
        &mut self,
        key_event: &winit::event::KeyEvent,
        modifiers: &winit::event::Modifiers,
        event: &mut EventQueue<'_>,
    ) -> DialogResponse {
        if key_event.state.is_pressed() && key_event.logical_key == Key::Named(NamedKey::Escape) {
            if self.sub_menu.is_some() {
                self.sub_menu = None;
                event.push(GlobalEvent::ConstRedraw);
                return DialogResponse::RequestRedraw;
            } else {
                return DialogResponse::Close;
            }
        }

        if let Some(sub) = self.sub_menu.as_mut() {
            return sub.process_input(key_event, modifiers, event);
        }

        if key_event.logical_key == Key::Named(NamedKey::Enter) {
            if key_event.state.is_pressed() {
                self.pressed = true;
                return DialogResponse::RequestRedraw;
            } else if self.pressed {
                self.pressed = false;
                match &self.buttons[self.selected].1 {
                    Action::Menu(menu) => {
                        let menu = match menu {
                            Menu::File => Self::file(),
                            Menu::Playback => Self::playback(),
                            Menu::Sample => Self::sample(),
                            Menu::Instrument => Self::instrument(),
                            Menu::Settings => Self::settings(),
                        };
                        self.sub_menu = Some(Box::new(menu));
                        return DialogResponse::RequestRedraw;
                    }
                    Action::Page(page) => {
                        event.push(GlobalEvent::GoToPage(*page));
                        return DialogResponse::Close;
                    }
                    Action::NotYetImplemented => {
                        println!("Not yet implementes");
                        return DialogResponse::RequestRedraw;
                    }
                    Action::Event(global_event) => {
                        event.push(global_event.clone());
                        return DialogResponse::Close;
                    }
                }
            }
        }

        if key_event.state.is_pressed() {
            if key_event.logical_key == Key::Named(NamedKey::ArrowUp) && self.selected > 0 {
                self.selected -= 1;
                self.pressed = false;
                return DialogResponse::RequestRedraw;
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowDown)
                && self.selected < self.buttons.len() - 1
            {
                self.selected += 1;
                self.pressed = false;
                return DialogResponse::RequestRedraw;
            }
        }

        DialogResponse::None
    }
}

impl PageMenu {
    const BACKGROUND_COLOR: u8 = 2;
    const TOPLEFT_COLOR: u8 = 3;
    const BOTRIGHT_COLOR: u8 = 1;
    const fn new(
        name: &'static str,
        pos: CharPosition,
        width: usize,
        buttons: &'static [(&'static str, Action)],
    ) -> Self {
        let rect = CharRect::new(
            pos.y(),
            pos.y() + 5 + (3 * buttons.len()),
            pos.x(),
            pos.x() + width + 3,
        );

        Self {
            name,
            rect,
            selected: 0,
            pressed: false,
            buttons,
            sub_menu: None,
        }
    }

    fn draw_button_corners(rect: CharRect, draw_buffer: &mut DrawBuffer) {
        let framebuffer = &mut draw_buffer.framebuffer;
        let pixel_rect = PixelRect::from(rect);
        // draw top right corner
        for y in 0..FONT_SIZE {
            for x in y..FONT_SIZE {
                framebuffer[pixel_rect.top() + y][pixel_rect.right() - FONT_SIZE + x + 1] =
                    Self::BOTRIGHT_COLOR;
            }
        }
        // draw botleft corner
        for y in 0..FONT_SIZE {
            for x in 0..(FONT_SIZE - y) {
                framebuffer[pixel_rect.bot() - y][pixel_rect.left() + x] = Self::BOTRIGHT_COLOR;
            }
        }
    }

    pub const fn main() -> Self {
        Self::new(
            "Main Menu",
            CharPosition::new(6, 11),
            29,
            &[
                ("File Menu...", Action::Menu(Menu::File)),
                ("Playback Menu...", Action::Menu(Menu::Playback)),
                (
                    "View Patterns        (F2)",
                    Action::Page(PagesEnum::Pattern),
                ),
                ("Sample Menu...", Action::Menu(Menu::Sample)),
                ("Instrument Menu...", Action::Menu(Menu::Instrument)),
                (
                    "View Orders/Panning (F11)",
                    Action::Page(PagesEnum::OrderList),
                ),
                (
                    "View Variables      (F12)",
                    Action::Page(PagesEnum::SongDirectoryConfig),
                ),
                ("Message Editor (Shift-F9)", Action::NotYetImplemented),
                ("Settings Menu...", Action::Menu(Menu::Settings)),
                ("Help!                (F1)", Action::Page(PagesEnum::Help)),
            ],
        )
    }

    pub const fn file() -> Self {
        Self::new(
            "File Menu",
            CharPosition::new(25, 13),
            26,
            &[
                ("Load...           (F9)", Action::NotYetImplemented),
                ("New...        (Ctrl-N)", Action::NotYetImplemented),
                ("Save Current  (Ctrl-S)", Action::NotYetImplemented),
                ("Save As...       (F10)", Action::NotYetImplemented),
                ("Export...  (Shift-F10)", Action::NotYetImplemented),
                ("Message Log (Ctrl-F11)", Action::NotYetImplemented),
                (
                    "Quit          (Ctrl-Q)",
                    Action::Event(GlobalEvent::CloseRequested),
                ),
            ],
        )
    }

    pub const fn playback() -> Self {
        Self::new(
            "Playback Menu",
            CharPosition::new(25, 13),
            31,
            &[
                ("Show Infopage          (F5)", Action::NotYetImplemented),
                (
                    "Play Song         (Ctrl-F5)",
                    Action::Event(GlobalEvent::Playback(PlaybackType::Song)),
                ),
                (
                    "Play Pattern           (F6)",
                    Action::Event(GlobalEvent::Playback(PlaybackType::Pattern)),
                ),
                (
                    "Play from Order  (Shift-F6)",
                    Action::Event(GlobalEvent::Playback(PlaybackType::FromOrder)),
                ),
                ("Play from Mark/Cursor  (F7)", Action::NotYetImplemented),
                (
                    "Stop                   (F8)",
                    Action::Event(GlobalEvent::Playback(PlaybackType::Stop)),
                ),
                ("Reinit Soundcard   (Ctrl-I)", Action::NotYetImplemented),
                ("Driver Screen    (Shift-F5)", Action::NotYetImplemented),
                ("Calculate Length   (Ctrl-P)", Action::NotYetImplemented),
            ],
        )
    }

    pub const fn sample() -> Self {
        Self::new(
            "Sample Menu",
            CharPosition::new(25, 20),
            29,
            &[
                (
                    "Sample List          (F3)",
                    Action::Page(PagesEnum::SampleList),
                ),
                ("Sample Library  (Ctrl-F3)", Action::NotYetImplemented),
            ],
        )
    }

    pub const fn instrument() -> Self {
        Self::new(
            "Instrument Menu",
            CharPosition::new(20, 23),
            33,
            &[
                ("Instrument List          (F4)", Action::NotYetImplemented),
                ("Instrument Library  (Ctrl-F4)", Action::NotYetImplemented),
            ],
        )
    }

    pub const fn settings() -> Self {
        Self::new(
            "Settings Menu",
            CharPosition::new(22, 25),
            38,
            &[
                (
                    "Preferences             (Shift-F5)",
                    Action::NotYetImplemented,
                ),
                (
                    "MIDI Configuration      (Shift-F1)",
                    Action::NotYetImplemented,
                ),
                (
                    "System Configuration     (Ctrl-F1)",
                    Action::NotYetImplemented,
                ),
                (
                    "Palette Editor          (Ctrl-F12)",
                    Action::NotYetImplemented,
                ),
                (
                    "Font Editor            (Shift-F12)",
                    Action::NotYetImplemented,
                ),
                (
                    "Toggle Fullscreen (Ctrl-Alt-Enter)",
                    Action::NotYetImplemented,
                ),
            ],
        )
    }
}
