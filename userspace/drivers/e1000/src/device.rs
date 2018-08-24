use sip::driver::{Dma, physical_map};
use sip::Result;
use std::mem;

const CTRL: u32 = 0x00;
const CTRL_LRST: u32 = 1 << 3;
const CTRL_ASDE: u32 = 1 << 5;
const CTRL_SLU: u32 = 1 << 6;
const CTRL_ILOS: u32 = 1 << 7;
const CTRL_RST: u32 = 1 << 26;
const CTRL_VME: u32 = 1 << 30;
const CTRL_PHY_RST: u32 = 1 << 31;

const STATUS: u32 = 0x08;

const FCAL: u32 = 0x28;
const FCAH: u32 = 0x2C;
const FCT: u32 = 0x30;
const FCTTV: u32 = 0x170;

const ICR: u32 = 0xC0;

const IMS: u32 = 0xD0;
const IMS_TXDW: u32 = 1;
const IMS_TXQE: u32 = 1 << 1;
const IMS_LSC: u32 = 1 << 2;
const IMS_RXSEQ: u32 = 1 << 3;
const IMS_RXDMT: u32 = 1 << 4;
const IMS_RX: u32 = 1 << 6;
const IMS_RXT: u32 = 1 << 7;

const RCTL: u32 = 0x100;
const RCTL_EN: u32 = 1 << 1;
const RCTL_UPE: u32 = 1 << 3;
const RCTL_MPE: u32 = 1 << 4;
const RCTL_LPE: u32 = 1 << 5;
const RCTL_LBM: u32 = 1 << 6 | 1 << 7;
const RCTL_BAM: u32 = 1 << 15;
const RCTL_BSIZE1: u32 = 1 << 16;
const RCTL_BSIZE2: u32 = 1 << 17;
const RCTL_BSEX: u32 = 1 << 25;
const RCTL_SECRC: u32 = 1 << 26;

const RDBAL: u32 = 0x2800;
const RDBAH: u32 = 0x2804;
const RDLEN: u32 = 0x2808;
const RDH: u32 = 0x2810;
const RDT: u32 = 0x2818;

const RAL0: u32 = 0x5400;
const RAH0: u32 = 0x5404;

#[repr(packed)]
struct Rd {
    buffer: u64,
    length: u16,
    checksum: u16,
    status: u8,
    error: u8,
    special: u16,
}

const RD_DD: u8 = 1;
const RD_EOP: u8 = 1 << 1;

const TCTL: u32 = 0x400;
const TCTL_EN: u32 = 1 << 1;
const TCTL_PSP: u32 = 1 << 3;

const TDBAL: u32 = 0x3800;
const TDBAH: u32 = 0x3804;
const TDLEN: u32 = 0x3808;
const TDH: u32 = 0x3810;
const TDT: u32 = 0x3818;

#[repr(packed)]
struct Td {
    buffer: u64,
    length: u16,
    cso: u8,
    command: u8,
    status: u8,
    css: u8,
    special: u16,
}

const TD_CMD_EOP: u8 = 1;
const TD_CMD_IFCS: u8 = 1 << 1;
const TD_CMD_RS: u8 = 1 << 3;
const TD_DD: u8 = 1;

#[repr(packed)]
struct Data {
    receive_buffer: [[u8; 16384]; 16],
    transmit_buffer: [[u8; 16384]; 16],
    receive_ring: [Rd; 16],
    transmit_ring: [Td; 16],
}

impl Data {
    #[inline]
    fn receive_buffer_offsets() -> [u64; 16] {
        let mut buf = [0; 16];

        for i in 0..buf.len() {
            buf[i] = i as u64 * 16384
        }

        buf
    }

    #[inline]
    fn transmit_buffer_offsets() -> [u64; 16] {
        let mut buf = [mem::size_of::<[[u8; 16384]; 16]>() as u64; 16];
        
        for i in 0..buf.len() {
            buf[i] += i as u64 * 16384
        }

        buf
    }

    #[inline]
    fn receive_ring_offset() -> u64 {
        (mem::size_of::<[[u8; 16384]; 16]>() * 2) as u64
    }

    #[inline]
    fn transmit_ring_offset() -> u64 {
        ((mem::size_of::<[[u8; 16384]; 16]>() * 2) + mem::size_of::<[Rd; 16]>()) as u64
    }
}

pub struct Intel8254x {
    base: *mut u32,
    dma: Dma<Data>,
}

impl Intel8254x {
    pub unsafe fn new(physical_base: u64) -> Result<Intel8254x> {
        let base = physical_map::<[u8; 1024 * 256]>(physical_base)? as *mut u32;

        let mut card = Intel8254x {
            base,
            dma: Dma::zeroed()?,
        };

        card.init();

        Ok(card)
    }

    unsafe fn read_reg(&self, register: u32) -> u32 {
        self.base.add(register as usize).read_volatile()
    }

    unsafe fn write_reg(&self, register: u32, data: u32) -> u32 {
        self.base.add(register as usize).write_volatile(data);
        self.base.add(register as usize).read_volatile()
    }

    unsafe fn flag(&self, register: u32, flag: u32, value: bool) {
        if value {
            self.write_reg(register, self.read_reg(register) | flag);
        } else {
            self.write_reg(register, self.read_reg(register) & !flag);
        }
    }

    unsafe fn init(&mut self) {
        self.flag(CTRL, CTRL_RST, true);

        while self.read_reg(CTRL) & CTRL_RST == CTRL_RST {
            println!("  - waiting for reset: {:#x}", self.read_reg(CTRL));
        }

        // enable auto-negotiate, link, clear reset, do not invert loss-of-signal
        self.flag(CTRL, CTRL_ASDE | CTRL_SLU, true);
        self.flag(CTRL, CTRL_LRST | CTRL_PHY_RST | CTRL_ILOS, false);

        // no flow control
        self.write_reg(FCAH, 0);
        self.write_reg(FCAL, 0);
        self.write_reg(FCT, 0);
        self.write_reg(FCTTV, 0);

        // do not use VLANs
        self.flag(CTRL, CTRL_VME, false);

        let mac_low = self.read_reg(RAL0);
        let mac_high = self.read_reg(RAH0);
        let mac = [
            mac_low as u8,
            (mac_low >> 8) as u8,
            (mac_low >> 16) as u8,
            (mac_low >> 24) as u8,
            mac_high as u8,
            (mac_high >> 8) as u8,
        ];

        println!("  - mac address: {:>02x}:{:>02x}:{:>02x}:{:>02x}:{:>02x}:{:>02x}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

        // receive buffer
        let receive_buffer_offsets = Data::receive_buffer_offsets();

        for i in 0..self.dma.receive_ring.len() {
            self.dma.receive_ring[i].buffer = self.dma.physical() + receive_buffer_offsets[i];
        }

        let receive_ring_physical = self.dma.physical() + Data::receive_ring_offset();

        self.write_reg(RDBAH, (receive_ring_physical >> 32) as u32);
        self.write_reg(RDBAL, receive_ring_physical as u32);
        self.write_reg(RDLEN, (self.dma.receive_ring.len() * mem::size_of::<Rd>()) as u32);
        self.write_reg(RDH, 0);
        self.write_reg(RDT, self.dma.receive_ring.len() as u32 - 1);

        // transmit buffer
        let transmit_buffer_offsets = Data::transmit_buffer_offsets();

        for i in 0..self.dma.transmit_ring.len() {
            self.dma.transmit_ring[i].buffer = self.dma.physical() + transmit_buffer_offsets[i];
        }

        let transmit_ring_physical = self.dma.physical() + Data::transmit_ring_offset();

        self.write_reg(TDBAH, (transmit_ring_physical >> 32) as u32);
        self.write_reg(TDBAL, transmit_ring_physical as u32);
        self.write_reg(TDLEN, (self.dma.transmit_ring.len() * mem::size_of::<Td>()) as u32);
        self.write_reg(TDH, 0);
        self.write_reg(TDT, 0);

        self.write_reg(IMS, IMS_RXT | IMS_RX | IMS_RXDMT | IMS_RXSEQ);

        self.flag(RCTL, RCTL_EN, true);
        self.flag(RCTL, RCTL_UPE, true);
        // self.flag(RCTL, RCTL_MPE, true);
        self.flag(RCTL, RCTL_LPE, true);
        self.flag(RCTL, RCTL_LBM, false);
        // RCTL.RDMTS = Minimum threshold size ???
        // RCTL.MO = Multicast offset
        self.flag(RCTL, RCTL_BAM, true);
        self.flag(RCTL, RCTL_BSIZE1, true);
        self.flag(RCTL, RCTL_BSIZE2, false);
        self.flag(RCTL, RCTL_BSEX, true);
        self.flag(RCTL, RCTL_SECRC, true);

        self.flag(TCTL, TCTL_EN, true);
        self.flag(TCTL, TCTL_PSP, true);
        // TCTL.CT = Collision threshold
        // TCTL.COLD = Collision distance
        // TIPG Packet Gap
        // TODO ...

        while self.read_reg(STATUS) & 2 != 2 {
            println!("  - waiting for link up: {:#x}", self.read_reg(STATUS));
        }

        println!("  - link up with speed {}", match (self.read_reg(STATUS) >> 6) & 0b11 {
            0b00 => "10 Mb/s",
            0b01 => "100 Mb/s",
            _ => "1000 Mb/s",
        });
    }
}

