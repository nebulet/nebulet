use x86_64::instructions::port::Port;

pub static mut MASTER: Pic = Pic::new(0x20);
pub static mut SLAVE: Pic = Pic::new(0xA0);

/// This remaps the PIC
pub unsafe fn init() {
    // Start initialization
    MASTER.cmd.write(0x11);
    SLAVE.cmd.write(0x11);

    // Set offsets
    MASTER.data.write(0x20);
    SLAVE.data.write(0x28);

    // Set up cascade
    MASTER.data.write(4);
    SLAVE.data.write(2);

    // Set up interrupt mode (1 is 8086/88 mode, 2 is auto EOI)
    MASTER.data.write(1);
    SLAVE.data.write(1);

    let (master_mask, slave_mask) = mask_in![
        0, 1, 8
    ];
    // Set masks
    MASTER.data.write(master_mask);
    SLAVE.data.write(slave_mask);

    // Ack remaining interrupts
    MASTER.ack();
    SLAVE.ack();
}

macro mask_in($($irq:expr),*) {{
    let mut master_mask: u8 = !0;
    let mut slave_mask: u8 = !0;
    $(
        let irq: u8 = $irq;
        if irq < 8 {
            master_mask &= !(1 << irq);
        } else {
            slave_mask &= !(1 << (irq - 8));
        }
    )*
    (master_mask, slave_mask)
}}

/// Mostly taken from Redox OS
pub struct Pic {
    cmd: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    pub const fn new(port: u16) -> Pic {
        Pic {
            cmd: Port::new(port),
            data: Port::new(port + 1),
        }
    }

    pub unsafe fn ack(&mut self) {
        self.cmd.write(0x20);
    }

    pub unsafe fn mask_set(&mut self, irq: u8) {
        assert!(irq < 8);

        let mut mask = self.data.read();
        mask |= 1 << irq;
        self.data.write(mask);
    }

    pub unsafe fn mask_clear(&mut self, irq: u8) {
        assert!(irq < 8);

        let mut mask = self.data.read();
        mask &= !(1 << irq);
        self.data.write(mask);
    }
}