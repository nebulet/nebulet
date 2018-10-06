use acpi::{Acpi, search_for_rsdp_bios, AcpiHandler, PhysicalMapping};
use x86_64::{PhysAddr, VirtAddr, structures::paging::{Page, PhysFrame}};
use alloc::alloc::{Layout, Global, Alloc};
use super::paging::PageMapper;

pub struct NebuletAcpiHandler;

impl NebuletAcpiHandler {
    pub fn find_acpi() -> Acpi {
        let mut handler = NebuletAcpiHandler;

        if let Ok(acpi) = search_for_rsdp_bios(&mut handler) {
            acpi
        } else {
            panic!("Failed to find system acpi tables");
        }
    }
}

impl AcpiHandler for NebuletAcpiHandler {
    fn map_physical_region<T>(&mut self, physical_address: usize, size: usize) -> PhysicalMapping<T> {
        let phys_addr = PhysAddr::new(physical_address as u64);
        let mut page_mapper = unsafe { PageMapper::new() };
        let layout = Layout::from_size_align(size, 4096).unwrap();
        let virt_ptr = Global.alloc(layout).unwrap();
        let virt = VirtAddr::from_ptr(virt_ptr.as_ptr());
        let start_page = Page::containing_address(virt);
        
        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: virt_ptr.cast(),
            region_length
        }
    }

    fn unmap_physical_region<T>(&mut self, region: PhysicalMapping<T>) {

    }
}