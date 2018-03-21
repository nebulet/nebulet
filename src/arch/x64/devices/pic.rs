use x86_64::instructions::port::Port;

pub static mut MASTER: Pic = Pic::new(0x20);
pub static mut SLAVE: Pic = Pic::new(0xA0);

/// This remaps the PIC
pub unsafe fn init() {
    let mut wait_port: Port<u8> = Port::new(0x80);
    let mut wait = || wait_port.write(0);

    // Start initialization
    MASTER.cmd.write(0x11);
    wait();
    SLAVE.cmd.write(0x11);
    wait();

    // Set offsets
    MASTER.data.write(0x20);
    wait();
    SLAVE.data.write(0x28);
    wait();

    // Set up cascade
    MASTER.data.write(4);
    wait();
    SLAVE.data.write(2);
    wait();

    // Set up interrupt mode (1 is 8086/88 mode, 2 is auto EOI)
    MASTER.data.write(1);
    wait();
    SLAVE.data.write(1);
    wait();

    // clear all masks
    MASTER.data.write(0xff);
    SLAVE.data.write(0xff);

    MASTER.ack();
    SLAVE.ack();
}

/// Mostly taken from Redox OS
pub struct Pic {
    pub cmd: Port<u8>,
    pub data: Port<u8>,
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