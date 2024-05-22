use x86_64::port::Port;

macro_rules! intersperse {
    (
        $interspersed:expr;
        {
            $($code:stmt;)*
        }
    ) => {
        #[allow(redundant_semicolons)]
        {
            $(
                $code;
                $interspersed;
            )*
        }
    };
}

const CMD_INIT: u8 = 0x11;
const CMD_END_OF_INTERRUPT: u8 = 0x20;

const MODE_8086: u8 = 0x01;

struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    fn handles_interrupt(&self, interrupt: u8) -> bool {
        (self.offset..self.offset + 8).contains(&interrupt)
    }

    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(CMD_END_OF_INTERRUPT);
    }

    unsafe fn read_mask(&mut self) -> u8 {
        self.data.read()
    }

    unsafe fn write_mask(&mut self, mask: u8) {
        self.data.write(mask);
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> Self {
        Self {
            pics: [
                Pic {
                    offset: offset1,
                    command: Port::new(0x20),
                    data: Port::new(0x21),
                },
                Pic {
                    offset: offset2,
                    command: Port::new(0xA0),
                    data: Port::new(0xA1),
                },
            ],
        }
    }

    pub unsafe fn init(&mut self) {
        // need to add artifical delay, so write garbage data to this port between each write
        let mut wait_port: Port<u8> = Port::new(0x80);

        let saved_masks = self.read_masks();

        intersperse!(
            wait_port.write(0);
            {
                // start init process
                self.pics[0].command.write(CMD_INIT);
                self.pics[1].command.write(CMD_INIT);

                // write offsets to each pic
                self.pics[0].data.write(self.pics[0].offset);
                self.pics[1].data.write(self.pics[1].offset);

                // then configure chaining
                self.pics[0].data.write(4);
                self.pics[1].data.write(2);

                // and finally set mode
                self.pics[0].data.write(MODE_8086);
                self.pics[1].data.write(MODE_8086);
            }
        );

        // and finally restore saved masks from earlier
        self.write_masks(saved_masks[0], saved_masks[1]);
    }

    pub unsafe fn read_masks(&mut self) -> [u8; 2] {
        [self.pics[0].read_mask(), self.pics[1].read_mask()]
    }

    pub unsafe fn write_masks(&mut self, mask1: u8, mask2: u8) {
        self.pics[0].write_mask(mask1);
        self.pics[1].write_mask(mask2);
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt: u8) {
        if self.handles_interrupt(interrupt) {
            if self.pics[1].handles_interrupt(interrupt) {
                self.pics[1].end_of_interrupt();
            }

            self.pics[0].end_of_interrupt();
        }
    }
}
