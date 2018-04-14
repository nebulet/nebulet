use core::ops::{Deref, DerefMut};
use x86_64::structures::paging::{PageTable, Page, PageTableFlags, Level4, PhysFrame};
use x86_64::instructions::tlb;

use self::mapper::Mapper;

mod mapper;
mod temporary_page;

pub const P4: *mut PageTable<Level4> = 0xffffffff_fffff000 as *mut _;

pub unsafe fn init() -> ActivePageTable {
    let active_table = ActivePageTable::new();
    
    // let mut new_table = {
    //     let frame = memory::allocate_frames(1).expect("Couldn't allocate frame for paging::init");
    //     InactivePageTable::new(frame)
    // };

    active_table
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temp_page: &mut temporary_page::TemporaryPage, f: F)
        where F: FnOnce(&mut Mapper)
    {
        use x86_64::registers::control::Cr3;
        {
            let backup = Cr3::read().0;

            // map temporary page to current p4 table
            let p4_table = temp_page.map_table_frame(backup.clone(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
            tlb::flush_all(); // not happy about this

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set(backup, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
            tlb::flush_all();
        }

        temp_page.unmap(self);
    }

    pub fn flush(&mut self, page: Page) {
        tlb::flush(page.start_address());
    }
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

pub struct InactivePageTable {
    p4_frame: PhysFrame,
}

impl InactivePageTable {
    pub fn new(frame: PhysFrame) -> InactivePageTable {
        // TODO: zero and recursively map the frame
        InactivePageTable {
            p4_frame: frame,
        }
    }
}
