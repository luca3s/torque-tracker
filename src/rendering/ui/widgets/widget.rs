use winit::event::{KeyEvent, Modifiers};

use crate::rendering::DrawBuffer;

pub trait Widget {
    fn draw(&self, buffer: &mut DrawBuffer, selected: bool);
    /// returns a Some(usize) if the next widget gets selected
    fn process_input(&mut self, key_event: &KeyEvent) -> Option<usize>;
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
    left: Option<usize>,
    right: Option<usize>,
    top: Option<usize>,
    bot: Option<usize>,
    tab: Option<usize>,
    shift_tab: Option<usize>,
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
