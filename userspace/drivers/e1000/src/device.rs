use sip::driver::{Dma, physical_map};
use sip::Result;
use std::{mem, slice};

const REG_CTRL: u32         = 0x0000;
const REG_STATUS: u32       = 0x0008;
const REG_EEPROM: u32       = 0x0014;
const REG_CTRL_EXT: u32     = 0x0018;

const REG_RCTRL: u32        = 0x0100;
const REG_RXDESCLO: u32     = 0x2800;
const REG_RXDESCHI: u32     = 0x2804;
const REG_RXDESCLEN: u32    = 0x2808;
const REG_RXDESCHEAD: u32   = 0x2810;
const REG_RXDESCTAIL: u32   = 0x2818;

const REG_TCTRL: u32        = 0x0400;
const REG_TXDESCLO: u32     = 0x3800;
const REG_TXDESCHI: u32     = 0x3804;
const REG_TXDESCLEN: u32    = 0x3808;
const REG_TXDESCHEAD: u32   = 0x3810;
const REG_TXDESCTAIL: u32   = 0x3818;

const REG_RXADDR: u32       = 0x5400;

const RCTL_EN: u32          = 1 << 1;   // Receiver Enable
const RCTL_SBP: u32         = 1 << 2;   // Store Bad Packets 
const RCTL_UPE: u32         = 1 << 3;   // Unicast Promiscuous Enabled
const RCTL_MPE: u32         = 1 << 4;   // Multicast Promiscuous Enabled
const RCTL_LPE: u32         = 1 << 5;   // Long Packet Reception Enable
const RCTL_LBM_NONE: u32    = 0 << 6;   // No Loopback
const RCTL_LBM_PHY: u32     = 3 << 6;   // PHY or external SerDesc loopback
const RTCL_RDMTS_HALF: u32  = 0 << 8;   // Free Buffer Threshold is 1/2 of RDLEN

const RTCL_RDMTS_QUARTER: u32   = 1 << 8;   // Free Buffer Threshold is 1/4 of RDLEN
const RTCL_RDMTS_EIGHTH: u32    = 2 << 8;   // Free Buffer Threshold is 1/8 of RDLEN

const RCTL_MO_36: u32       = 0 << 12;  // Multicast Offset - bits 47:36
const RCTL_MO_35: u32       = 1 << 12;  // Multicast Offset - bits 46:35
const RCTL_MO_34: u32       = 2 << 12;  // Multicast Offset - bits 45:34
const RCTL_MO_32: u32       = 3 << 12;  // Multicast Offset - bits 43:32
const RCTL_BAM: u32         = 1 << 15;  // Broadcast Accept Mode
const RCTL_VFE: u32         = 1 << 18;  // VLAN Filter Enable
const RCTL_CFIEN: u32       = 1 << 19;  // Canonical Form Indicator Enable
const RCTL_CFI: u32         = 1 << 20;  // Canonical Form Indicator Bit Value
const RCTL_DPF: u32         = 1 << 22;  // Discard Pause Frames
const RCTL_PMCF: u32        = 1 << 23;  // Pass MAC Control Frames
const RCTL_SECRC: u32       = 1 << 26;  // Strip Ethernet CRC

const RCTL_BSIZE_256: u32   = 3 << 16;
const RCTL_BSIZE_512: u32   = 2 << 16;
const RCTL_BSIZE_1024: u32  = 1 << 16;
const RCTL_BSIZE_2048: u32  = 0 << 16;
const RCTL_BSIZE_4096: u32  = (3 << 16) | (1 << 25);
const RCTL_BSIZE_8192: u32  = (2 << 16) | (1 << 25);
const RCTL_BSIZE_16384: u32 = (1 << 16) | (1 << 25);

const TCTL_EN: u32          = 1 << 1;   // Transmit Enable
const TCTL_PSP: u32         = 1 << 3;   // Pad Short Packets
const TCTL_CT_SHIFT: u32    = 4;        // Collision Threshold
const TCTL_COLD_SHIFT: u32  = 12;       // Collision Distance
const TCTL_SWXOFF: u32      = 1 << 22;  // Software XOFF Transmission
const TCTL_RTLC: u32        = 1 << 24;  // Re-transmit on Late Collision

const CMD_EOP: u32          = 1 << 0;   // End of Packet
const CMD_IFCS: u32         = 1 << 1;   // Insert FCS
const CMD_IC: u32           = 1 << 2;   // Insert Checksum
const CMD_RS: u32           = 1 << 3;   // Report Status
const CMD_RPS: u32          = 1 << 4;   // Report Packet Sent
const CMD_VLE: u32          = 1 << 6;   // VLAN Packet Enable
const CMD_IDE: u32          = 1 << 7;   // Interrupt Delay Enable

#[repr(C, packed)]
struct Rd {
    buffer: u64,
    length: u16,
    checksum: u16,
    status: u8,
    error: u8,
    special: u16,
}

#[repr(C, packed)]
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

pub struct Intel8254x {
    base: *mut u32,
    receive_buffer: [Dma<[u8; 16384]>; 16],
    transmit_buffer: [Dma<[u8; 16384]>; 16],
    receive_ring: Dma<[Rd; 16]>,
    transmit_ring: Dma<[Td; 16]>,
    has_eeprom: bool,
}

impl Intel8254x {
    pub unsafe fn new(physical_base: u64) -> Result<Intel8254x> {
        let base = physical_map::<[u8; 1024 * 256]>(physical_base)? as *mut u32;
        println!("base: {:p}", base);

        println!("finished physical map");

        let mut card = Intel8254x {
            base,
            receive_buffer: [Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?],
            receive_ring: Dma::zeroed()?,
            transmit_buffer: [Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?],
            transmit_ring: Dma::zeroed()?,
            has_eeprom: false,
        };

        println!("Initializing card");

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

    unsafe fn eeprom_detect(&self) -> bool {
        self.write_reg(REG_EEPROM, 1);

        let mut has_eeprom = false;

        for _ in 0..100000 {
            let val = self.read_reg(REG_EEPROM);
            if val & 0x10 != 0 {
                has_eeprom = true;
            }
        }
        
        has_eeprom
    }

    unsafe fn eeprom_read(&self, addr: u8) -> u16 {
        let mut temp = 0;

        self.write_reg(REG_EEPROM, 1 | ((addr as u32) << 8));
        while {
            temp = self.read_reg(REG_EEPROM);
            temp & (1 << 4) == 0
        } {}
        ((temp >> 16) as u16) & 0xffff
    }

    unsafe fn read_mac(&self) -> [u8; 6] {
        let mut mac = [0; 6];

        if self.has_eeprom {
            let t = self.eeprom_read(0);
            mac[0] = t as u8;
            mac[1] = (t >> 8) as u8;
            let t = self.eeprom_read(1);
            mac[2] = t as u8;
            mac[3] = (t >> 8) as u8;
            let t = self.eeprom_read(2);
            mac[4] = t as u8;
            mac[5] = (t >> 8) as u8;
        } else {
            let mac_base = slice::from_raw_parts(self.base.add(REG_RXADDR as usize) as *const u8, 6);
            mac.copy_from_slice(mac_base);
        }

        mac
    }

    unsafe fn flag(&self, register: u32, flag: u32, value: bool) {
        if value {
            self.write_reg(register, self.read_reg(register) | flag);
        } else {
            self.write_reg(register, self.read_reg(register) & !flag);
        }
    }

    unsafe fn init(&mut self) {
        self.has_eeprom = self.eeprom_detect();
        println!("has_eeprom: {}", self.has_eeprom);

        let mac = self.read_mac();

        println!("  - mac address: {:>02x}:{:>02x}:{:>02x}:{:>02x}:{:>02x}:{:>02x}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

        println!("status: {:#x}", self.read_reg(REG_STATUS));

        // self.flag(CTRL, CTRL_RST, true);

        // while self.read_reg(CTRL) & CTRL_RST == CTRL_RST {
        //     println!("  - waiting for reset: {:#x}", self.read_reg(CTRL));
        // }

        // // enable auto-negotiate, link, clear reset, do not invert loss-of-signal
        // self.flag(CTRL, CTRL_ASDE | CTRL_SLU, true);
        // self.flag(CTRL, CTRL_LRST | CTRL_PHY_RST | CTRL_ILOS, false);

        // // no flow control
        // self.write_reg(FCAH, 0);
        // self.write_reg(FCAL, 0);
        // self.write_reg(FCT, 0);
        // self.write_reg(FCTTV, 0);

        // // do not use VLANs
        // self.flag(CTRL, CTRL_VME, false);

        // let mac_low = self.read_reg(RAL0);
        // let mac_high = self.read_reg(RAH0);
        // let mac = [
        //     mac_low as u8,
        //     (mac_low >> 8) as u8,
        //     (mac_low >> 16) as u8,
        //     (mac_low >> 24) as u8,
        //     mac_high as u8,
        //     (mac_high >> 8) as u8,
        // ];

        // println!("  - mac address: {:>02x}:{:>02x}:{:>02x}:{:>02x}:{:>02x}:{:>02x}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

        // // receive buffer
        // for i in 0..self.receive_ring.len() {
        //     self.receive_ring[i].buffer = self.receive_buffer[i].physical();
        // }

        // let receive_ring_physical = self.receive_ring.physical();

        // self.write_reg(RDBAH, (receive_ring_physical >> 32) as u32);
        // self.write_reg(RDBAL, receive_ring_physical as u32);
        // self.write_reg(RDLEN, (self.receive_ring.len() * mem::size_of::<Rd>()) as u32);
        // self.write_reg(RDH, 0);
        // self.write_reg(RDT, self.receive_ring.len() as u32 - 1);

        // // transmit buffer
        // for i in 0..self.transmit_ring.len() {
        //     self.transmit_ring[i].buffer = self.transmit_buffer[i].physical();
        // }

        // let transmit_ring_physical = self.transmit_ring.physical();

        // self.write_reg(TDBAH, (transmit_ring_physical >> 32) as u32);
        // self.write_reg(TDBAL, transmit_ring_physical as u32);
        // self.write_reg(TDLEN, (self.transmit_ring.len() * mem::size_of::<Td>()) as u32);
        // self.write_reg(TDH, 0);
        // self.write_reg(TDT, 0);

        // self.write_reg(IMS, IMS_RXT | IMS_RX | IMS_RXDMT | IMS_RXSEQ);

        // self.flag(RCTL, RCTL_EN, true);
        // self.flag(RCTL, RCTL_UPE, true);
        // // self.flag(RCTL, RCTL_MPE, true);
        // self.flag(RCTL, RCTL_LPE, true);
        // self.flag(RCTL, RCTL_LBM, false);
        // // RCTL.RDMTS = Minimum threshold size ???
        // // RCTL.MO = Multicast offset
        // self.flag(RCTL, RCTL_BAM, true);
        // self.flag(RCTL, RCTL_BSIZE1, true);
        // self.flag(RCTL, RCTL_BSIZE2, false);
        // self.flag(RCTL, RCTL_BSEX, true);
        // self.flag(RCTL, RCTL_SECRC, true);

        // self.flag(TCTL, TCTL_EN, true);
        // self.flag(TCTL, TCTL_PSP, true);
        // // TCTL.CT = Collision threshold
        // // TCTL.COLD = Collision distance
        // // TIPG Packet Gap
        // // TODO ...

        // while self.read_reg(STATUS) & 2 != 2 {
        //     // println!("  - waiting for link up: {:#x}", self.read_reg(STATUS));
        // }

        // println!("  - link up with speed {}", match (self.read_reg(STATUS) >> 6) & 0b11 {
        //     0b00 => "10 Mb/s",
        //     0b01 => "100 Mb/s",
        //     _ => "1000 Mb/s",
        // });
    }
}

