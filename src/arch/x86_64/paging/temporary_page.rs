use super::ActivePageTable;
use x86_64::structures::paging::{PhysFrame, Page, PageTableFlags, PageTable, Level1};
use x86_64::VirtAddr;

pub struct TemporaryPage {
    page: Page,
}

impl TemporaryPage {
    pub fn new(page: Page) -> TemporaryPage {
        TemporaryPage {
            page: page,
        }
    }

    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page
    pub fn map(&mut self, frame: PhysFrame, active_table: &mut ActivePageTable) -> VirtAddr {
        assert!(active_table.translate_page(self.page.clone()).is_none(),
            "temporary page already mapped");
        
        active_table.map_to(self.page.clone(), frame, PageTableFlags::WRITABLE);
        self.page.start_address()
    }

    /// Unmaps the temporary page in the active table
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page.clone());
    }

    /// Maps the temporary page to the given page table frame in the active
    /// table. Returns a reference to the now mapped table
    pub fn map_table_frame(&mut self, frame: PhysFrame, active_table: &mut ActivePageTable) -> &mut PageTable<Level1> {
        unsafe {
            &mut *(self.map(frame, active_table).as_u64() as *mut PageTable<Level1>)
        }
    }
}