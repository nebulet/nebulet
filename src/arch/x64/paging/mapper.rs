use x86_64::{VirtAddr, PhysAddr};
use x86_64::structures::paging::{Page, PageTable, Level4, PhysFrame, PageTableFlags, PAGE_SIZE};
use core::ptr::NonNull;

use memory;

pub struct Mapper {
    p4: NonNull<PageTable<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: NonNull::new_unchecked(super::P4),
        }
    }

    pub fn p4(&self) -> &PageTable<Level4> {
        unsafe {
            self.p4.as_ref()
        }
    }

    pub fn p4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe {
            self.p4.as_mut()
        }
    }

    pub fn translate(&self, virtual_addr: VirtAddr) -> Option<PhysAddr> {
        let offset = virtual_addr.as_u64() % PAGE_SIZE as u64;
        self.translate_page(Page::containing_address(virtual_addr))
            .map(|frame| frame.start_address() + offset)
    }

    pub fn translate_page(&self, page: Page) -> Option<PhysFrame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 16GiB page?
                if let Some(start_addr) = p3_entry.points_to() {
                    let start_frame = PhysFrame::containing_address(start_addr);
                    if p3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                        let p2_index = u16::from(page.p2_index()) as u64;
                        let p1_index = u16::from(page.p1_index()) as u64;
                        // address must be 1GiB aligned
                        return Some(PhysFrame {
                            number: start_frame.number + p2_index * 512 + p1_index,
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_addr) = p2_entry.points_to() {
                        let start_frame = PhysFrame::containing_address(start_addr);
                        if p2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                            let p1_index = u16::from(page.p1_index()) as u64;
                            return Some(PhysFrame {
                                number: start_frame.number + p1_index,
                            });
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].points_to())
            .map(|addr| PhysFrame::containing_address(addr))
        .or_else(huge_page)
    }

    pub fn map_to(&mut self, page: Page, frame: PhysFrame, flags: PageTableFlags) {
        let p3 = self.p4_mut()
                   .next_table_create(page.p4_index(), || memory::allocate_frame().unwrap());
        let p2 = p3.next_table_create(page.p3_index(), || memory::allocate_frame().unwrap());
        let p1 = p2.next_table_create(page.p2_index(), || memory::allocate_frame().unwrap());

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PageTableFlags::PRESENT);
    }

    pub fn map(&mut self, page: Page, flags: PageTableFlags) {
        let frame = memory::allocate_frame().expect("could not allocate frame");
        self.map_to(page, frame, flags);
    }

    pub fn remap(&mut self, page: Page, flags: PageTableFlags) {
        let p3 = self.p4_mut()
                   .next_table_create(page.p4_index(), || memory::allocate_frame().unwrap());
        let p2 = p3.next_table_create(page.p3_index(), || memory::allocate_frame().unwrap());
        let p1 = p2.next_table_create(page.p2_index(), || memory::allocate_frame().unwrap());
        let mut entry = p1[page.p1_index()];

        let frame = PhysFrame::containing_address(entry.points_to().unwrap());
        entry.set(frame, flags | PageTableFlags::PRESENT);
    }

    pub fn identity_map(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        self.map_to(page, frame, flags);
    }

    pub fn unmap(&mut self, page: Page) {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("Mapping code does not support huge pages yet");
        let frame = PhysFrame::containing_address(p1[page.p1_index()].points_to().unwrap());
        p1[page.p1_index()].set_unused();

        use x86_64::instructions::tlb;
        tlb::flush(page.start_address());

        memory::deallocate_frame(frame);
    }
}