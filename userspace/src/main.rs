#![no_main]

#[macro_use]
extern crate sip;

use sip::thread;
use sip::Mutex;

struct Philosopher {
    name: String,
    left: usize,
    right: usize,
}

impl Philosopher {
    fn new(name: &str, left: usize, right: usize) -> Philosopher {
        Philosopher {
            name: name.to_string(),
            left,
            right,
        }
    }

    fn eat(&self, table: &Table) {
        let _left = table.forks[self.left].lock();
        let _right = table.forks[self.right].lock();

        println!("{} is eating.", self.name);

        thread::yield_now();

        println!("{} is done eating.", self.name);
    }
}

struct Table {
    forks: [Mutex<()>; 5],
}

#[no_mangle]
pub fn main() {
    static TABLE: Table = Table {
        forks: [
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
            Mutex::new(()),
        ]
    };

    let philosophers = vec![
        Philosopher::new("Judith Butler", 0, 1),
        Philosopher::new("Gilles Deleuze", 1, 2),
        Philosopher::new("Karl Marx", 2, 3),
        Philosopher::new("Emma Goldman", 3, 4),
        Philosopher::new("Michel Foucault", 0, 4),
    ];

    for p in philosophers {
        thread::spawn(move || {
            p.eat(&TABLE);
        }).unwrap();
    }

    loop {}
}
