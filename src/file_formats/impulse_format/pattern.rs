pub struct ImpulsePattern {}

impl ImpulsePattern {
    // pub fn new(buf: &[u8]) -> Self {
    //     let lenght = u16::from_le_bytes([buf[0], buf[1]]);
    //     let num_rows = u16::from_le_bytes([buf[2], buf[3]]);
    //     assert!((32..=200).contains(&num_rows));

    //     // bytes 4 - 7 empty

    //     let mut counter = 8;
    //     let channel_variable = buf[8];
    //     if channel_variable == 0 {
    //         // end of row. dont know what that means
    //     }

    //     let channel = (channel_variable - 1) & 63; // "channel is 0 based"????? dont understand the docs
    //                                                // if channel_variable & 128 == 1 {

    //     // }

    //     todo!()
    // }

    pub fn load(buf: &[u8]) {
        // byte length of the pattern itself
        let length = u16::from_le_bytes([buf[0], buf[1]]);
        let num_rows = u16::from_le_bytes([buf[2], buf[3]]);
        // has 8 byte header
        let buf = &buf[..=(usize::from(length) + 8)];
        let mut read_pointer = 8;
        let mut row_num: u8 = 0;
        
        loop {
            if read_pointer >= usize::from(length) + 8 {
                break;
            }

            let mut channel_variable = buf[read_pointer];
            read_pointer += 1;

            let mut maskvariable = 0;
            if channel_variable == 0 {
                println!("end of row {row_num}");
                row_num += 1;
            } else {
                let channel = (channel_variable - 1) & 63; // 64 channels, 0 based
                println!("channel: {channel}");
                if (channel_variable & 0b10000000) != 0 {
                    maskvariable = buf[read_pointer];
                    read_pointer += 1;
                    println!("mask: {maskvariable}");
                }

                if (maskvariable & 0b00000001) != 0 {
                    let note = buf[read_pointer];
                    read_pointer += 1;
                    println!("note: {note}");
                }

                if (maskvariable & 0b00000010) != 0 {
                    let instrument = buf[read_pointer];
                    read_pointer += 1;
                    println!("instrument: {instrument}");
                }

                if (maskvariable & 0b00000100) != 0 {
                    let vol_pan = buf[read_pointer];
                    read_pointer += 1;
                    println!("vol_pan: {vol_pan}");
                }

                if (maskvariable & 0b00001000) != 0 {
                    let command = buf[read_pointer];
                    read_pointer += 1;
                    println!("command: {command}");
                    let cmd_val = buf[read_pointer];
                    read_pointer += 1;
                    println!("cmd val: {cmd_val}");
                }

                if (maskvariable & 0b00010000) != 0 {
                    println!("note = last note for chennel");
                }

                if (maskvariable & 0b00100000) != 0 {
                    println!("instr = last instr for channel");
                }

                if (maskvariable & 0b01000000) != 0 {
                    println!("volpan = last volpan for channel");
                }

                if (maskvariable & 0b10000000) != 0 {
                    println!("cmd = last cmd for channel");
                    println!("cmd val = last cmd val for channel");
                }

            }
        }

        println!("pattern: {buf:?}");
    }
}
