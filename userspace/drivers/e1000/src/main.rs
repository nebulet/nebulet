
extern crate sip;

mod device;

use self::device::Intel8254x;

fn main() {
    let device = unsafe { Intel8254x::new(0xcafebabe) };

    
}
