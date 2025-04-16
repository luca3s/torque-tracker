use std::collections::VecDeque;

use winit::keyboard::{Key, NamedKey};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect, PixelRect, FONT_SIZE},
    draw_buffer::DrawBuffer,
    ui::pages::PagesEnum,
};

use super::{Dialog, DialogResponse};

enum PageOrPageMenu {
    Menu(Menu),
    Page(PagesEnum),
    CloseApp,
    // should be removed when possible
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
    button_names: &'static [&'static str],
    button_actions: &'static [PageOrPageMenu],
    sub_menu: Option<Box<Self>>,
}

impl Dialog for PageMenu {
    fn draw(&self, draw_buffer: &mut DrawBuffer) {
        let top_left = self.rect.top_left();
        let top = top_left.y();
        let left = top_left.x();

        draw_buffer.draw_rect(Self::BACKGROUND_COLOR, self.rect);
        draw_buffer.draw_string(self.name, top_left + CharPosition::new(7, 2), 3, 2);
        self.draw_outer_box(draw_buffer);

        for (num, button) in self.button_names.iter().enumerate() {
            let text_color = match self.selected == num {
                true => 11,
                false => 0,
            };

            draw_buffer.draw_string(
                button,
                top_left + CharPosition::new(4, (3 * num) + 5),
                text_color,
                Self::BACKGROUND_COLOR,
            );
            let top = top + (3 * num) + 4;
            Self::draw_button_box(
                CharRect::new(top, top + 2, left + 2, left + self.rect.width() - 2),
                (self.pressed || self.sub_menu.is_some()) && self.selected == num,
                draw_buffer,
            );
        }
        if let Some(sub) = self.sub_menu.as_ref() {
            sub.draw(draw_buffer);
        }
    }

    fn process_input(
        &mut self,
        key_event: &winit::event::KeyEvent,
        modifiers: &winit::event::Modifiers,
        event: &mut VecDeque<GlobalEvent>,
    ) -> DialogResponse {
        if key_event.state.is_pressed() && key_event.logical_key == Key::Named(NamedKey::Escape) {
            if self.sub_menu.is_some() {
                self.sub_menu = None;
                event.push_back(GlobalEvent::ConstRedraw);
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
                match &self.button_actions[self.selected] {
                    PageOrPageMenu::Menu(menu) => {
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
                    PageOrPageMenu::Page(page) => {
                        event.push_back(GlobalEvent::GoToPage(*page));
                        return DialogResponse::Close;
                    }
                    PageOrPageMenu::NotYetImplemented => {
                        println!("Not yet implementes");
                        return DialogResponse::RequestRedraw;
                    }
                    PageOrPageMenu::CloseApp => {
                        event.push_back(GlobalEvent::CloseRequested);
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
                && self.selected < self.button_actions.len() - 1
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
        button_names: &'static [&'static str],
        button_actions: &'static [PageOrPageMenu],
    ) -> Self {
        assert!(button_names.len() == button_actions.len());

        let rect = CharRect::new(
            pos.y(),
            pos.y() + 5 + (3 * button_names.len()),
            pos.x(),
            pos.x() + width + 3,
        );

        Self {
            name,
            rect,
            selected: 0,
            pressed: false,
            button_names,
            button_actions,
            sub_menu: None,
        }
    }

    // TODO: the original draws the inner border in the outer part of the next character, while
    // this currently does it on the inner part of the same character.
    //
    // Not sure what i like more.
    fn draw_outer_box(&self, draw_buffer: &mut DrawBuffer) {
        let pixel_rect = PixelRect::from(self.rect);
        let outer_color = draw_buffer.get_raw_color(Self::TOPLEFT_COLOR);
        let inner_color = draw_buffer.get_raw_color(Self::BOTRIGHT_COLOR);
        let framebuffer = &mut draw_buffer.framebuffer;

        // upper & lower outer line
        for x in pixel_rect.left()..=pixel_rect.right() {
            framebuffer[pixel_rect.top()][x] = outer_color;
            framebuffer[pixel_rect.top() + 1][x] = outer_color;
            framebuffer[pixel_rect.bot()][x] = outer_color;
            framebuffer[pixel_rect.bot() - 1][x] = outer_color;
        }
        // left & right outer line
        for y in pixel_rect.top()..=pixel_rect.bot() {
            framebuffer[y][pixel_rect.left()] = outer_color;
            framebuffer[y][pixel_rect.left() + 1] = outer_color;
            framebuffer[y][pixel_rect.right()] = outer_color;
            framebuffer[y][pixel_rect.right() - 1] = outer_color;
        }
        // upper & lower inner line
        for x in (pixel_rect.left() + FONT_SIZE - 1)..=(pixel_rect.right() - FONT_SIZE + 1) {
            framebuffer[pixel_rect.top() + FONT_SIZE - 1][x] = inner_color;
            framebuffer[pixel_rect.bot() - FONT_SIZE + 1][x] = inner_color;
        }
        // left & right inner line
        for y in (pixel_rect.top() + FONT_SIZE - 1)..=(pixel_rect.bot() - FONT_SIZE + 1) {
            framebuffer[y][pixel_rect.left() + FONT_SIZE - 1] = inner_color;
            framebuffer[y][pixel_rect.right() - FONT_SIZE + 1] = inner_color;
        }
    }

    fn draw_button_box(rect: CharRect, invert_colors: bool, draw_buffer: &mut DrawBuffer) {
        let corner_color = draw_buffer.get_raw_color(Self::BOTRIGHT_COLOR);
        let (topleft, botright) = match invert_colors {
            true => (Self::BOTRIGHT_COLOR, Self::TOPLEFT_COLOR),
            false => (Self::TOPLEFT_COLOR, Self::BOTRIGHT_COLOR),
        };
        let topleft = draw_buffer.get_raw_color(topleft);
        let botright = draw_buffer.get_raw_color(botright);
        let framebuffer = &mut draw_buffer.framebuffer;
        let pixel_rect = PixelRect::from(rect);
        // draw top line
        for x in pixel_rect.left()..=(pixel_rect.right() - FONT_SIZE) {
            framebuffer[pixel_rect.top()][x] = topleft;
        }
        // draw bot line
        for x in (pixel_rect.left() + FONT_SIZE)..=pixel_rect.right() {
            framebuffer[pixel_rect.bot()][x] = botright;
        }
        // draw left line
        for y in (pixel_rect.top())..=(pixel_rect.bot() - FONT_SIZE) {
            framebuffer[y][pixel_rect.left()] = topleft;
        }
        // draw right line
        for y in (pixel_rect.top() + FONT_SIZE)..=pixel_rect.bot() {
            framebuffer[y][pixel_rect.right()] = botright;
        }
        // draw top right corner
        for y in 0..FONT_SIZE {
            for x in y..FONT_SIZE {
                framebuffer[pixel_rect.top() + y][pixel_rect.right() - FONT_SIZE + x + 1] =
                    corner_color;
            }
        }
        // draw botleft corner
        for y in 0..FONT_SIZE {
            for x in 0..(FONT_SIZE - y) {
                framebuffer[pixel_rect.bot() - y][pixel_rect.left() + x] = corner_color;
            }
        }
    }

    pub const fn main() -> Self {
        Self::new(
            "Main Menu",
            CharPosition::new(6, 11),
            29,
            &[
                "File Menu...",
                "Playback Menu...",
                "View Patterns        (F2)",
                "Sample Menu...",
                "Instrument Menu...",
                "View Orders/Panning (F11)",
                "View Variables      (F12)",
                "Message Editor (Shift-F9)",
                "Settings Menu...",
                "Help!                (F1)",
            ],
            &[
                PageOrPageMenu::Menu(Menu::File),
                PageOrPageMenu::Menu(Menu::Playback),
                PageOrPageMenu::Page(PagesEnum::Pattern), // view patterns
                PageOrPageMenu::Menu(Menu::Sample),
                PageOrPageMenu::Menu(Menu::Instrument),
                PageOrPageMenu::NotYetImplemented, // orders / panning
                PageOrPageMenu::Page(PagesEnum::SongDirectoryConfig),
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::Menu(Menu::Settings),
                PageOrPageMenu::Page(PagesEnum::Help),
            ],
            // event_loop_proxy,
        )
    }

    pub const fn file() -> Self {
        Self::new(
            "File Menu",
            CharPosition::new(25, 13),
            26,
            &[
                "Load...           (F9)",
                "New...        (Ctrl-N)",
                "Save Current  (Ctrl-S)",
                "Save As...       (F10)",
                "Export...  (Shift-F10)",
                "Message Log (Ctrl-F11)",
                "Quit          (Ctrl-Q)",
            ],
            &[
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::CloseApp,
            ],
        )
    }

    pub const fn playback() -> Self {
        Self::new(
            "Playback Menu",
            CharPosition::new(25, 13),
            31,
            &[
                "Show Infopage          (F5)",
                "Play Song         (Ctrl-F5)",
                "Play Pattern           (F6)",
                "Play from Order  (Shift-F6)",
                "Play from Mark/Cursor  (F7)",
                "Stop                   (F8)",
                "Reinit Soundcard   (Ctrl-I)",
                "Driver Screen    (Shift-F5)",
                "Calculate Length   (Ctrl-P)",
            ],
            &[
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
            ],
        )
    }

    pub const fn sample() -> Self {
        Self::new(
            "Sample Menu",
            CharPosition::new(25, 20),
            29,
            &["Sample List          (F3)", "Sample Library  (Ctrl-F3)"],
            &[
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
            ],
        )
    }

    pub const fn instrument() -> Self {
        Self::new(
            "Instrument Menu",
            CharPosition::new(20, 23),
            33,
            &[
                "Instrument List          (F4)",
                "Instrument Library  (Ctrl-F4)",
            ],
            &[
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
            ],
        )
    }

    pub const fn settings() -> Self {
        Self::new(
            "Settings Menu",
            CharPosition::new(22, 25),
            38,
            &[
                "Preferences             (Shift-F5)",
                "MIDI Configuration      (Shift-F1)",
                "System Configuration     (Ctrl-F1)",
                "Palette Editor          (Ctrl-F12)",
                "Font Editor            (Shift-F12)",
                "Toggle Fullscreen (Ctrl-Alt-Enter)",
            ],
            &[
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
            ],
        )
    }
}
