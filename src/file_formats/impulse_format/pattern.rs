use crate::playback::pattern::{Event, Pattern};

pub fn load_pattern(buf: &[u8]) -> Pattern {
    // byte length of the pattern itself
    let length = u16::from_le_bytes([buf[0], buf[1]]);
    let num_rows = u16::from_le_bytes([buf[2], buf[3]]);

    let mut pattern = Pattern::new(usize::from(num_rows));

    let mut read_pointer = 8;
    let mut row_num: u8 = 0;

    let mut last_mask = [0; 64];
    let mut last_event = [Event::default(); 64];

    loop {
        if read_pointer >= usize::from(length) + 8 {
            break;
        }

        let channel_variable = buf[read_pointer];
        read_pointer += 1;

        if channel_variable == 0 {
            row_num += 1;
            continue;
        }

        let channel = (channel_variable - 1) & 63; // 64 channels, 0 based
        let channel_id = usize::from(channel);

        let maskvariable = if (channel_variable & 0b10000000) != 0 {
            let val = buf[read_pointer];
            last_mask[channel_id] = val;
            read_pointer += 1;
            val
        } else {
            last_mask[channel_id]
        };

        let mut event = Event::default();

        // Note
        if (maskvariable & 0b00000001) != 0 {
            let note = buf[read_pointer];
            read_pointer += 1;

            event.note = note;
            last_event[channel_id].note = note;
        }

        // Instrument / Sample
        if (maskvariable & 0b00000010) != 0 {
            let instrument = buf[read_pointer];
            read_pointer += 1;

            event.instr = instrument;
            last_event[channel_id].instr = instrument;
        }

        // Volume
        if (maskvariable & 0b00000100) != 0 {
            let vol_pan = buf[read_pointer].try_into().unwrap();
            read_pointer += 1;

            last_event[channel_id].vol = vol_pan;
            event.vol = vol_pan;
        }

        // Effect
        if (maskvariable & 0b00001000) != 0 {
            let command = buf[read_pointer];
            read_pointer += 1;
            let cmd_val = buf[read_pointer];
            read_pointer += 1;

            let cmd = (command.try_into().unwrap(), cmd_val);
            last_event[channel_id].command = Some(cmd);

            event.command = Some(cmd);
        }

        // Same note
        if (maskvariable & 0b00010000) != 0 {
            event.note = last_event[channel_id].note;
        }

        // Same Instr / Sample
        if (maskvariable & 0b00100000) != 0 {
            event.instr = last_event[channel_id].instr;
        }

        // Same volume
        if (maskvariable & 0b01000000) != 0 {
            event.vol = last_event[channel_id].vol;
        }

        // Same Command
        if (maskvariable & 0b10000000) != 0 {
            event.command = last_event[channel_id].command;
        }

        pattern.rows[usize::from(row_num)].push((channel, event));
    }

    pattern
}
