#![no_main]
#![feature(const_fn)]

#[macro_use]
extern crate sip;

use sip::thread;
use sip::Mutex;

#[derive(Debug)]
struct Philosopher {
    name: &'static str,
    left: usize,
    right: usize,
}

impl Philosopher {
    const fn new(name: &'static str, left: usize, right: usize) -> Philosopher {
        Philosopher {
            name,
            left,
            right,
        }
    }

    fn eat(&self, table: &[Mutex<()>]) {
        let _left = table[self.left].lock();
        let _right = table[self.right].lock();

        println!("{} is eating.", self.name);

        thread::yield_now();

        println!("{} is done eating.", self.name);
    }
}

#[no_mangle]
pub fn main() {
    use std::panic;

    panic::set_hook(Box::new(|info| {
        println!("{}", info);
    }));

    static TABLE: [Mutex<()>; 5] = [
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
    ];

    let philosophers = vec![
        Philosopher::new("Judith Butler", 0, 1),
        Philosopher::new("Gilles Deleuze", 1, 2),
        Philosopher::new("Karl Marx", 2, 3),
        Philosopher::new("Emma Goldman", 3, 4),
        Philosopher::new("Michel Foucault", 4, 0),
    ];

    for p in philosophers {
        thread::spawn(move || {
            p.eat(&TABLE);
        }).unwrap();
    }

    loop {}
}
