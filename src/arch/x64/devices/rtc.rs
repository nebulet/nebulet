use x86_64::instructions::port::Port;

use time;

// Pretty much copied from Redox OS

pub unsafe fn init() {
    let mut rtc = Rtc::new();
    time::START.0 = rtc.time();
}

fn cvt_bcd(value: usize) -> usize {
    (value & 0xF) + ((value / 16) * 10)
}

pub struct Rtc {
    addr: Port<u8>,
    data: Port<u8>,
}

impl Rtc {
    /// Create an empty Rtc
    pub fn new() -> Rtc {
        Rtc {
            addr: Port::new(0x70),
            data: Port::new(0x71),
        }
    }

    unsafe fn read(&mut self, reg: u8) -> u8 {
        self.addr.write(reg);
        self.data.read()
    }

    unsafe fn wait(&mut self) {
        // while self.read(0xA) & 0x80 != 0x80 {}
        // while self.read(0xA) & 0x80 == 0x80 {}
    }

    /// Get Time
    pub fn time(&mut self) -> u64 {
        let mut second;
        let mut minute;
        let mut hour;
        let mut day;
        let mut month;
        let mut year;
        let mut century;
        let register_b;

        /*let century_register = if let Some(ref fadt) = acpi::ACPI_TABLE.lock().fadt {
            Some(fadt.century)
        } else {
            None
        };*/

        unsafe {
            self.wait();
            second = self.read(0) as usize;
            minute = self.read(2) as usize;
            hour = self.read(4) as usize;
            day = self.read(7) as usize;
            month = self.read(8) as usize;
            year = self.read(9) as usize;
            century = /* TODO: Fix invalid value from VirtualBox
            if let Some(century_reg) = century_register {
                self.read(century_reg) as usize
            } else */ {
                20
            };
            register_b = self.read(0xB);
        }

        if register_b & 4 != 4 {
            second = cvt_bcd(second);
            minute = cvt_bcd(minute);
            hour = cvt_bcd(hour & 0x7F) | (hour & 0x80);
            day = cvt_bcd(day);
            month = cvt_bcd(month);
            year = cvt_bcd(year);
            century = /* TODO: Fix invalid value from VirtualBox
            if century_register.is_some() {
                cvt_bcd(century)
            } else */ {
                century
            };
        }

        if register_b & 2 != 2 || hour & 0x80 == 0x80 {
            hour = ((hour & 0x7F) + 12) % 24;
        }

        year += century * 100;

        // Unix time from clock
        let mut secs: u64 = (year as u64 - 1970) * 31_536_000;

        let mut leap_days = (year as u64 - 1972) / 4 + 1;
        if year % 4 == 0 && month <= 2 {
            leap_days -= 1;
        }
        secs += leap_days * 86_400;

        match month {
            2 => secs += 2_678_400,
            3 => secs += 5_097_600,
            4 => secs += 7_776_000,
            5 => secs += 10_368_000,
            6 => secs += 13_046_400,
            7 => secs += 15_638_400,
            8 => secs += 18_316_800,
            9 => secs += 20_995_200,
            10 => secs += 23_587_200,
            11 => secs += 26_265_600,
            12 => secs += 28_857_600,
            _ => (),
        }

        secs += (day as u64 - 1) * 86_400;
        secs += hour as u64 * 3600;
        secs += minute as u64 * 60;
        secs += second as u64;

        secs
    }
}
