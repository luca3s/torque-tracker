use std::collections::VecDeque;

use crate::{app::GlobalEvent, coordinates::CharRect, draw_buffer::DrawBuffer};

use super::{Page, PageResponse};

pub struct HelpPage {}

impl Page for HelpPage {
    fn draw(&mut self, _draw_buffer: &mut DrawBuffer) {}

    fn draw_constant(&mut self, draw_buffer: &mut DrawBuffer) {
        draw_buffer.draw_rect(2, CharRect::PAGE_AREA);
    }

    fn process_key_event(
        &mut self,
        _modifiers: &winit::event::Modifiers,
        _key_event: &winit::event::KeyEvent,
        _events: &mut VecDeque<GlobalEvent>,
    ) -> PageResponse {
        PageResponse::None
    }
}

impl HelpPage {
    pub fn new() -> Self {
        Self {}
    }
}
