use std::{
    collections::VecDeque,
    ops::{AddAssign, Deref, SubAssign},
};

use winit::keyboard::{Key, NamedKey};

use crate::{
    app::GlobalEvent,
    coordinates::{CharPosition, CharRect, PixelRect, FONT_SIZE, WINDOW_SIZE_CHARS},
    draw_buffer::DrawBuffer,
    ui::dialog::slider_dialog::SliderDialog,
};

use super::{NextWidget, StandardResponse, Widget, WidgetResponse};

pub struct BoundNumber<const MIN: i16, const MAX: i16> {
    inner: i16,
}

impl<const MIN: i16, const MAX: i16> AddAssign<i16> for BoundNumber<MIN, MAX> {
    fn add_assign(&mut self, rhs: i16) {
        let new = self.inner.saturating_add(rhs);
        // need to check both low and high bound as you can add a negative number
        if new > MAX {
            self.inner = MAX;
        } else if new < MIN {
            self.inner = MIN;
        } else {
            self.inner = new;
        }
    }
}

impl<const MIN: i16, const MAX: i16> SubAssign<i16> for BoundNumber<MIN, MAX> {
    fn sub_assign(&mut self, rhs: i16) {
        let new = self.inner.saturating_sub(rhs);
        // need to check both low and high bound as you can sub a negative number
        if new > MAX {
            self.inner = MAX;
        } else if new < MIN {
            self.inner = MIN;
        } else {
            self.inner = new;
        }
    }
}

impl<const MIN: i16, const MAX: i16> Deref for BoundNumber<MIN, MAX> {
    type Target = i16;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const MIN: i16, const MAX: i16> BoundNumber<MIN, MAX> {
    const _VALID: () = assert!(MIN <= MAX, "MIN must be less than or equal to MAX");

    /// sets inner to MIN or MAX if its out of bounds
    pub const fn new_saturating(value: i16) -> Self {
        if value > MAX {
            Self { inner: MAX }
        } else if value < MIN {
            Self { inner: MIN }
        } else {
            Self { inner: value }
        }
    }

    /// panics on out of bounds value
    pub const fn new(value: i16) -> Self {
        assert!(value <= MAX);
        assert!(value >= MIN);
        Self { inner: value }
    }

    /// sets to MAX or MIN when the value is out of bounds
    pub fn set_saturating(&mut self, value: i16) {
        if value > MAX {
            self.inner = MAX
        } else if value < MIN {
            self.inner = MIN;
        } else {
            self.inner = value;
        }
    }

    pub fn try_set(&mut self, value: i16) -> Result<(), ()> {
        if MIN < value && value < MAX {
            self.inner = value;
            Ok(())
        } else {
            Err(())
        }
    }

    // dont need any arguments as MIN and MAX are compile time known
    pub const fn get_middle() -> i16 {
        MIN + (MAX / 2)
    }
}

/// Slider needs more Space then is specified in Rect as it draws the current value with an offset of 2 right to the box.
/// currently this always draws 3 chars, but this can only take values between -99 and 999. If values outside of that are needed, this implementation needs to change
pub struct Slider<const MIN: i16, const MAX: i16, R> {
    number: BoundNumber<MIN, MAX>,
    position: CharPosition,
    width: usize,
    next_widget: NextWidget,
    dialog_return: fn(i16) -> GlobalEvent,
    callback: Box<dyn Fn(i16) -> R + Send>,
}

impl<const MIN: i16, const MAX: i16, R> Widget for Slider<MIN, MAX, R> {
    type Response = R;
    fn draw(&self, draw_buffer: &mut DrawBuffer, selected: bool) {
        const BACKGROUND_COLOR: u8 = 0;
        const CURSOR_COLOR: u8 = 2;
        const CURSOR_SELECTED_COLOR: u8 = 3;

        const CURSOR_WIDTH: usize = 4;

        draw_buffer.draw_string(
            &format!("{:03}", *self.number),
            self.position + (self.width + 2, 0),
            1,
            2,
        );

        draw_buffer.draw_rect(
            BACKGROUND_COLOR,
            CharRect::new(
                self.position.y(),
                self.position.y(),
                self.position.x(),
                self.position.x() + self.width,
            ),
        );

        let cursor_pos = if MAX == MIN {
            0
        } else {
            // shift value scale. this is the new MAX
            // MIN -> MAX => 0 -> (MAX-MIN)
            let num_possible_values = usize::from(MAX.abs_diff(MIN));

            // shift value scale as shown below
            // MIN -> MAX => 0 -> (MAX-MIN)
            let value = usize::from(self.number.abs_diff(MIN));

            // first + 1 makes it have a border on the left side
            // rest mostly stole from original source code
            1 + value * (self.width * FONT_SIZE + 1) / num_possible_values
        };
        let color = match selected {
            true => CURSOR_SELECTED_COLOR,
            false => CURSOR_COLOR,
        };

        let cursor_pixel_rect = PixelRect::new(
            // +1 to have a space above
            (self.position.y() * FONT_SIZE) + 1,
            // -2 to make if have a 1. not go into the next line and 2. have a empty row below
            (self.position.y() * FONT_SIZE) + (FONT_SIZE - 2),
            (self.position.x() * FONT_SIZE) + cursor_pos + CURSOR_WIDTH,
            (self.position.x() * FONT_SIZE) + cursor_pos,
        );

        draw_buffer.draw_pixel_rect(color, cursor_pixel_rect);
    }

    fn process_input(
        &mut self,
        modifiers: &winit::event::Modifiers,
        key_event: &winit::event::KeyEvent,
        event: &mut VecDeque<GlobalEvent>,
    ) -> WidgetResponse<R> {
        if !key_event.state.is_pressed() {
            return WidgetResponse::default();
        }

        // move the slider
        // change the internal value and call the callback
        // this seems stupid but allows to reduce code duplication
        if key_event.logical_key == Key::Named(NamedKey::ArrowRight)
            || key_event.logical_key == Key::Named(NamedKey::ArrowLeft)
        {
            // existance of this bracket allows us to only call the callback if the value really was changed
            'move_slider: {
                let direction = if key_event.logical_key == Key::Named(NamedKey::ArrowRight) {
                    // the number is at its max, so increasing it doesnt do anything so we break here
                    if *self.number == MAX {
                        break 'move_slider;
                    }
                    1
                } else if key_event.logical_key == Key::Named(NamedKey::ArrowLeft) {
                    // analog to the ArrowRight branch
                    if *self.number == MIN {
                        break 'move_slider;
                    }
                    -1
                } else {
                    // unreachable as the outer if has already checked this.
                    unreachable!()
                };

                // sets amount according to the modifiers like the original. why the original SchismTracker behaves like this i don't know
                // only reason i can imagine is that if you know the behaviour you can move quite quickly through a slider
                let amount = {
                    let mut amount = 1;
                    if modifiers.state().super_key() {
                        amount *= 2;
                    }
                    if modifiers.state().shift_key() {
                        amount *= 4;
                    }
                    if modifiers.state().alt_key() {
                        amount *= 8;
                    }
                    amount
                };

                self.number += amount * direction;
                return WidgetResponse {
                    standard: StandardResponse::RequestRedraw,
                    extra: Some((self.callback)(*self.number)),
                };
            }
        // set the value directly, by opening a pop-up
        } else if let Key::Character(text) = &key_event.logical_key {
            let mut chars = text.chars();
            if let Some(first_char) = chars.next() {
                if first_char.is_ascii_digit() {
                    let dialog = SliderDialog::new(first_char, self.dialog_return);
                    event.push_back(GlobalEvent::OpenDialog(Box::new(dialog)));
                    return WidgetResponse::default();
                }
            }
        } else {
            return self.next_widget.process_key_event(key_event, modifiers);
        }

        WidgetResponse::default()
    }
}

impl<const MIN: i16, const MAX: i16, R> Slider<MIN, MAX, R> {
    /// next_widget left and right must be None, because they cant be called
    pub fn new(
        inital_value: i16,
        position: CharPosition,
        width: usize,
        next_widget: NextWidget,
        dialog_return: fn(i16) -> GlobalEvent,
        callback: impl Fn(i16) -> R + Send + 'static,
    ) -> Self {
        assert!(MIN <= MAX, "MIN must be less than or equal to MAX");
        // panic is fine, because this object only is generated with compile time values
        assert!(next_widget.left.is_none());
        assert!(next_widget.right.is_none());
        // just put them here so i remember this limitation of the current implementation in the future
        // if this ever fails the code that draws the current value right of the slider needs to be made aware of the lenght needed and not just draw 3 chars
        assert!(MIN >= -99, "draw implementation needs to be redone");
        assert!(MAX <= 999, "draw implementation needs to be redone");

        // need right from the slider additional space to display the current value
        assert!(position.x() + width + 5 < WINDOW_SIZE_CHARS.0);

        Self {
            number: BoundNumber::new(inital_value),
            position,
            width,
            next_widget,
            dialog_return,
            callback: Box::new(callback),
        }
    }

    pub fn try_set(&mut self, value: i16) -> Result<R, ()> {
        self.number.try_set(value).map(|_| (self.callback)(value))
    }
}
