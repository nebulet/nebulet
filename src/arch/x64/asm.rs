//! Architecture helpers


pub macro read_gs_offset64($offset:expr) {{
    let ret: u64;
    asm!("mov $0, gs:$1" : "=r"(ret) : "i"($offset) : "memory" : "intel", "volatile");
    ret
}}

pub macro write_gs_offset64($offset:expr, $val:expr) {{
    asm!("mov gs:$1, $0" : : "r"($val), "i"($offset) : "memory" : "intel" "volatile");
}}

pub macro read_gs_offset32($offset:expr) {{
    let ret: u32;
    asm!("movl $0, gs:$1" : "=r"(ret) : "i"($offset) : "memory" : "intel", "volatile");
    ret
}}

pub macro write_gs_offset32($offset:expr, $val:expr) {{
    asm!("movl gs:$1, $0" : : "r"($val), "i"($offset) : "memory" : "intel" "volatile");
}}