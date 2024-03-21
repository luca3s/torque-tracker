use winit::event::{KeyEvent, Modifiers};

use crate::visual::draw_buffer::DrawBuffer;

pub trait Widget {
    fn draw(&self, buffer: &mut DrawBuffer, selected: bool);
    /// returns a Some(usize) if the next widget gets selected
    fn process_input(&mut self, modifiers: &Modifiers, key_event: &KeyEvent) -> Option<usize>;
}

// enum NextCommand {
//     Left,
//     Right,
//     Top,
//     Bot,
//     Tab,
// }

#[derive(Default)]
pub struct NextWidget {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
    pub tab: Option<usize>,
    pub shift_tab: Option<usize>,
}

// impl NextWidget {
//     fn get(&self, cmd: &NextCommand) -> Option<usize> {
//         match cmd {
//             NextCommand::Left => self.left,
//             NextCommand::Right => self.right,
//             NextCommand::Top => self.top,
//             NextCommand::Bot => self.bot,
//             NextCommand::Tab => self.tab,
//         }
//     }
// }
