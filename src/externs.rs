// #![no_builtins]

// use core::mem;

// const WORD_SIZE: usize = mem::size_of::<usize>();

// /// Memmove
// ///
// /// Copy N bytes of memory from src to dest. The memory areas may overlap.
// ///
// /// This faster implementation works by copying bytes not one-by-one, but in
// /// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
// #[no_mangle]
// pub unsafe extern fn memmove(dest: *mut u8, src: *const u8,
//                              n: usize) -> *mut u8 {
//     if src < dest as *const u8 {
//         let n_usize: usize = n/WORD_SIZE; // Number of word sized groups
//         let mut i: usize = n_usize*WORD_SIZE;

//         // Copy `WORD_SIZE` bytes at a time
//         while i != 0 {
//             i -= WORD_SIZE;
//             *((dest as usize + i) as *mut usize) =
//                 *((src as usize + i) as *const usize);
//         }

//         let mut i: usize = n;

//         // Copy 1 byte at a time
//         while i != n_usize*WORD_SIZE {
//             i -= 1;
//             *((dest as usize + i) as *mut u8) =
//                 *((src as usize + i) as *const u8);
//         }
//     } else {
//         let n_usize: usize = n/WORD_SIZE; // Number of word sized groups
//         let mut i: usize = 0;

//         // Copy `WORD_SIZE` bytes at a time
//         let n_fast = n_usize*WORD_SIZE;
//         while i < n_fast {
//             *((dest as usize + i) as *mut usize) =
//                 *((src as usize + i) as *const usize);
//             i += WORD_SIZE;
//         }

//         // Copy 1 byte at a time
//         while i < n {
//             *((dest as usize + i) as *mut u8) =
//                 *((src as usize + i) as *const u8);
//             i += 1;
//         }
//     }

//     dest
// }

// /// Memcmp
// ///
// /// Compare two blocks of memory.
// ///
// /// This faster implementation works by comparing bytes not one-by-one, but in
// /// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
// #[no_mangle]
// pub unsafe extern fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
//     let n_usize: usize = n/WORD_SIZE;
//     let mut i: usize = 0;

//     let n_fast = n_usize*WORD_SIZE;
//     while i < n_fast {
//         let a = *((s1 as usize + i) as *const usize);
//         let b = *((s2 as usize + i) as *const usize);
//         if a != b {
//             let n: usize = i + WORD_SIZE;
//             // Find the one byte that is not equal
//             while i < n {
//                 let a = *((s1 as usize + i) as *const u8);
//                 let b = *((s2 as usize + i) as *const u8);
//                 if a != b {
//                     return a as i32 - b as i32;
//                 }
//                 i += 1;
//             }
//         }
//         i += WORD_SIZE;
//     }

//     while i < n {
//         let a = *((s1 as usize + i) as *const u8);
//         let b = *((s2 as usize + i) as *const u8);
//         if a != b {
//             return a as i32 - b as i32;
//         }
//         i += 1;
//     }

//     0
// }
