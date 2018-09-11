use x86_64::instructions::port::Port;

pub struct PciBus(u32);

impl PciBus {
    pub fn new(bus: u8) -> PciBus {
        PciBus(bus as u32)
    }

    pub fn scan<F>(&self, f: F) -> usize
        where F: Fn(PciDevice)
    {
        let mut count = 0;
        
        for slot_num in 0..32 {
            let slot = PciSlot {
                bus: self.0,
                slot: slot_num,
            };

            if let Some(device) = slot.get_device() {
                count += 1;
                f(device);
            }
        }

        count
    }
}

#[derive(Debug)]
pub struct PciDevice {
    slot: PciSlot,
    pub vendor: u16,
    pub device: u16,
}

impl PciDevice {
    pub fn base_address(&self) -> u32 {
        let bar_low = self.slot.config_read(0x00, 0x10);
        let bar_high = self.slot.config_read(0x00, 0x12);

        (bar_high as u32) << 16 | bar_low as u32
    }

    pub fn header_type(&self) -> u8 {
        self.slot.config_read(0x00, 0xc + 0x2) as u8
    }

    pub fn read_cmd(&self) -> u16 {
        self.slot.config_read(0x00, 0x04)
    }

    pub fn write_cmd(&mut self, cmd: u16) {
        self.slot.config_write(0x00, 0x04, cmd as u32);
    }
}

#[derive(Debug)]
pub struct PciSlot {
    bus: u32,
    slot: u32,
}

impl PciSlot {
    fn config_read(&self, func: u8, offset: u8) -> u16 {
        let address = (self.bus << 16)
        | (self.slot << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xfc)
        | 0x80000000u32;

        let mut addr_port: Port<u32> = Port::new(0xcf8);
        let config_port: Port<u32> = Port::new(0xcfc);

        unsafe {
            addr_port.write(address);

            ((config_port.read() >> ((offset & 2) * 8)) & 0xffff) as u16
        }
    }

    fn config_write(&mut self, func: u8, offset: u8, val: u32) {
        let address = (self.bus << 16)
        | (self.slot << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xfc)
        | 0x80000000u32;

        let mut addr_port: Port<u32> = Port::new(0xcf8);
        let mut config_port: Port<u32> = Port::new(0xcfc);

        unsafe {
            addr_port.write(address);

            config_port.write(val);
        }
    }

    pub fn get_device(&self) -> Option<PciDevice> {
        let vendor = self.config_read(0, 0);

        match vendor {
            0xffff => None,
            _ => {
                let device = self.config_read(0, 2);

                Some(PciDevice {
                    slot: PciSlot { ..*self },
                    vendor,
                    device,
                })
            },
        }
    }
}

