use std::num::NonZeroU8;

#[derive(Clone, Copy, Debug)]
pub struct Event {
    note: Option<NonZeroU8>,
    instr: Option<NonZeroU8>,
    vol_pan: u8,
    command: Option<(NonZeroU8, u8)>,
}

// #[derive(Default)]
// pub struct Row {
//     events: Vec<(u8, Event)>,
// }
pub type Row = Vec<(u8, Event)>;

pub struct Pattern {
    // rows: Vec<Row>,
    rows: Box<[Row]>
}

impl Pattern {
    pub fn set_length(&mut self, new_len: usize) {
        let vec = match new_len.cmp(&self.rows.len()) {
            std::cmp::Ordering::Less => {
                let mut vec: Vec<Row> = Vec::with_capacity(new_len);
                for row in &mut self.rows[0..new_len] {
                    vec.push(std::mem::take(row));
                }
                vec
            },
            std::cmp::Ordering::Equal => return,
            std::cmp::Ordering::Greater => {
                let mut vec: Vec<Row> = Vec::with_capacity(new_len);
                for row in self.rows.iter_mut() {
                    vec.push(std::mem::take(row));
                }
                for _ in    self.rows.len()..new_len {
                    vec.push(Vec::new());
                }
                vec
            },
        };

        self.rows = vec.into_boxed_slice();
    }

    pub fn set_event(&mut self, row: usize, channel: u8, event: Event) {
        let new_event = event;
        if let Some((_, event)) = self.rows[row].iter_mut().find(|(c, _)| *c == channel) {
            *event = new_event;
        } else {
            self.rows[row].push((channel, new_event));
        }
    }

    /// if there is no event, does nothing
    pub fn remove_event(&mut self, row: usize, channel: u8) {
        let i = self.rows[row].iter().position(|(c, _)| *c == channel);
        if let Some(i) = i {
            self.rows[row].swap_remove(i);
        }
    }
}

pub enum PatternOperation {
    SetLenght(usize),
    SetEvent {
        row: usize,
        channel: u8,
        event: Event,
    },
    RemoveEvent {
        row: usize,
        channel: u8,
    }
}

impl left_right::Absorb<PatternOperation> for Pattern {
    fn absorb_first(&mut self, operation: &mut PatternOperation, other: &Self) {
        // don't need it mutable
        let operation: &PatternOperation = operation;
        match operation {
            PatternOperation::SetLenght(_) => todo!(),
            PatternOperation::SetEvent { row, channel, event } => self.set_event(*row, *channel, *event),
            PatternOperation::RemoveEvent { row, channel } => self.remove_event(*row, *channel),
        }
    }

    fn sync_with(&mut self, first: &Self) {
        todo!()
    }
}

