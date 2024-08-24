use winit::{
    event_loop::EventLoopProxy,
    keyboard::{Key, NamedKey},
};

use crate::visual::{
    coordinates::{CharPosition, CharRect},
    event_loop::CustomWinitEvent,
    ui::pages::page::PagesEnum,
};

use super::dialog::{Dialog, DialogResponse};

enum PageOrPageMenu {
    Menu(Menu),
    Page(PagesEnum),
    // should be removed when possible
    NotYetImplemented,
}

// Main Missing, because it cant be opened from a menu
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
    event_loop_proxy: EventLoopProxy<CustomWinitEvent>,
}

impl Dialog for PageMenu {
    fn draw(&self, draw_buffer: &mut crate::visual::draw_buffer::DrawBuffer) {
        const BACKGROUND_COLOR: usize = 2;
        const TOPLEFT_COLOR: usize = 3;
        const BOTRIGHT_COLOR: usize = 1;
        let top_left = self.rect.top_left();
        let button_width = self.rect.width() - 3;

        draw_buffer.draw_rect(BACKGROUND_COLOR, self.rect);
        draw_buffer.draw_string(self.name, top_left + CharPosition::new(6, 2), 3, 2);
        draw_buffer.draw_box(self.rect, TOPLEFT_COLOR, BACKGROUND_COLOR, BACKGROUND_COLOR);

        for (num, button) in self.button_names.iter().enumerate() {
            let text_color = match self.selected == num {
                true => 11,
                false => 0,
            };

            let box_colors = match self.pressed && self.selected == num {
                true => (BOTRIGHT_COLOR, TOPLEFT_COLOR),
                false => (TOPLEFT_COLOR, BOTRIGHT_COLOR),
            };

            draw_buffer.draw_string(
                button,
                top_left + CharPosition::new(2, (3 * num) + 4),
                text_color,
                BACKGROUND_COLOR,
            );
            draw_buffer.draw_box(
                CharRect::new(
                    top_left.y() + (3 * num) + 3,
                    top_left.y() + (3 * num) + 5,
                    top_left.x() + 1,
                    top_left.x() + 2 + button_width,
                ),
                BACKGROUND_COLOR,
                box_colors.0,
                box_colors.1,
            );
        }
    }

    fn process_input(
        &mut self,
        key_event: &winit::event::KeyEvent,
        _modifiers: &winit::event::Modifiers,
    ) -> DialogResponse {
        if key_event.logical_key == Key::Named(NamedKey::Enter) {
            if key_event.state.is_pressed() {
                self.pressed = true;
                return DialogResponse::RequestRedraw;
            } else if self.pressed {
                self.pressed = false;
                match &self.button_actions[self.selected] {
                    PageOrPageMenu::Menu(menu) => {
                        let menu = match menu {
                            Menu::File => Self::file(self.event_loop_proxy.clone()),
                            Menu::Playback => Self::playback(self.event_loop_proxy.clone()),
                            Menu::Sample => Self::sample(self.event_loop_proxy.clone()),
                            Menu::Instrument => Self::instrument(self.event_loop_proxy.clone()),
                            Menu::Settings => Self::settings(self.event_loop_proxy.clone()),
                        };

                        let _ = self
                            .event_loop_proxy
                            .send_event(CustomWinitEvent::OpenDialog(Box::new(menu)));
                    }
                    PageOrPageMenu::Page(page) => return DialogResponse::SwitchToPage(*page),
                    PageOrPageMenu::NotYetImplemented => {
                        println!("Not yet implementes");
                        return DialogResponse::RequestRedraw;
                    }
                }
            }
        }

        if key_event.state.is_pressed() {
            if key_event.logical_key == Key::Named(NamedKey::Escape) {
                return DialogResponse::Close;
            } else if key_event.logical_key == Key::Named(NamedKey::ArrowUp) && self.selected > 0 {
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
    const WIDTH: usize = 20;

    const fn new(
        name: &'static str,
        pos: CharPosition,
        width: usize,
        button_names: &'static [&'static str],
        button_actions: &'static [PageOrPageMenu],
        event_loop_proxy: EventLoopProxy<CustomWinitEvent>,
    ) -> Self {
        assert!(button_names.len() == button_actions.len());

        let rect = CharRect::new(
            pos.y(),
            pos.y() + 3 + (3 * button_names.len()),
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
            event_loop_proxy,
        }
    }

    pub const fn main(event_loop_proxy: EventLoopProxy<CustomWinitEvent>) -> Self {
        Self::new(
            "Main Menu",
            CharPosition::new(6, 11),
            25,
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
                PageOrPageMenu::NotYetImplemented, // view patterns
                PageOrPageMenu::Menu(Menu::Sample),
                PageOrPageMenu::Menu(Menu::Instrument),
                PageOrPageMenu::NotYetImplemented, // orders / panning
                PageOrPageMenu::Page(PagesEnum::SongDirectoryConfig),
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::Menu(Menu::Settings),
                PageOrPageMenu::Page(PagesEnum::Help),
            ],
            event_loop_proxy,
        )
    }

    pub const fn file(event_loop_proxy: EventLoopProxy<CustomWinitEvent>) -> Self {
        Self::new(
            "File Menu",
            CharPosition::new(25, 13),
            22,
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
                PageOrPageMenu::NotYetImplemented,
            ],
            event_loop_proxy,
        )
    }

    pub const fn playback(event_loop_proxy: EventLoopProxy<CustomWinitEvent>) -> Self {
        Self::new(
            "Playback Menu",
            CharPosition::new(25, 13),
            27,
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
            event_loop_proxy,
        )
    }

    pub const fn sample(event_loop_proxy: EventLoopProxy<CustomWinitEvent>) -> Self {
        Self::new(
            "Sample Menu",
            CharPosition::new(25, 20),
            25,
            &["Sample List          (F3)", "Sample Library  (Ctrl-F3)"],
            &[
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
            ],
            event_loop_proxy,
        )
    }

    pub const fn instrument(event_loop_proxy: EventLoopProxy<CustomWinitEvent>) -> Self {
        Self::new(
            "Instrument Menu",
            CharPosition::new(20, 23),
            29,
            &[
                "Instrument List          (F4)",
                "Instrument Library  (Ctrl-F4)",
            ],
            &[
                PageOrPageMenu::NotYetImplemented,
                PageOrPageMenu::NotYetImplemented,
            ],
            event_loop_proxy,
        )
    }

    pub const fn settings(event_loop_proxy: EventLoopProxy<CustomWinitEvent>) -> Self {
        Self::new(
            "Settings Menu",
            CharPosition::new(22, 25),
            34,
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
            event_loop_proxy,
        )
    }
}
