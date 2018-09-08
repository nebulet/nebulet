use core::marker::PhantomData;
use core::{cmp, mem, slice, str};

pub struct Tar<'a> {
    data: &'a [u8],
}

impl<'a> Tar<'a> {
    pub fn load(data: &'a [u8]) -> Tar<'a> {
        Tar { data }
    }

    pub fn iter(&self) -> Iter<'a> {
        Iter {
            ptr: self.data.as_ptr(),
            remaining: self.data.len(),
            _phantom: PhantomData,
        }
    }
}

pub struct File<'a> {
    pub path: &'a str,
    pub data: &'a [u8],
}

pub struct Iter<'a> {
    ptr: *const u8,
    remaining: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = File<'a>;
    fn next(&mut self) -> Option<File<'a>> {
        let header_size = mem::size_of::<Header>();
        assert!(header_size == 512);

        // println!("debug: {}:{}", file!(), line!());

        if self.remaining <= header_size * 2 {
            return None;
        }

        let header = unsafe { &*(self.ptr as *const Header) };

        // println!("debug: {}:{}", file!(), line!());

        if header == unsafe { &mem::zeroed() } {
            return None;
        }

        // println!("debug: {}:{}", file!(), line!());

        self.remaining -= header_size;

        let first_null = header
            .size
            .iter()
            .enumerate()
            .find_map(|(i, &byte)| if byte == 0 { Some(i) } else { None })
            .unwrap_or(header.size.len());

        let size_str = str::from_utf8(&header.size[..first_null]).ok()?.trim();
        // println!("debug: {}:{}", file!(), line!());
        let size = usize::from_str_radix(size_str, 8).ok()?;
        // println!("debug: {}:{}", file!(), line!());
        let file_size = cmp::min(size, self.remaining);
        let rounded_file_size = {
            let rem = file_size % 512;
            file_size + 512 - rem
        };
        self.remaining -= rounded_file_size;

        let data =
            unsafe { slice::from_raw_parts(self.ptr.add(header_size) as *const u8, file_size) };
        self.ptr = unsafe { self.ptr.add(header_size + rounded_file_size) };

        let first_null = header
            .name
            .iter()
            .enumerate()
            .find_map(|(i, &byte)| if byte == 0 { Some(i) } else { None })
            .unwrap_or(header.name.len());
        // println!("debug: {}:{}", file!(), line!());
        let path = str::from_utf8(&header.name[..first_null]).ok()?;
        // println!("debug: {}:{}", file!(), line!());

        let file = File { path, data };

        Some(file)
    }
}

#[repr(C, align(512))]
struct Header {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    checksum: [u8; 8],
    typeflag: u8,
    linkname: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    uname: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    prefix: [u8; 155],
}

impl PartialEq for Header {
    fn eq(&self, other: &Header) -> bool {
        let self_slice = unsafe {
            slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<Header>())
        };
        let other_slice = unsafe {
            slice::from_raw_parts(other as *const _ as *const u8, mem::size_of::<Header>())
        };
        self_slice == other_slice
    }
}
