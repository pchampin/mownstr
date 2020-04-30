//! This example prograp allocates several MownStr in sequence,
//! freeing each one before allocating the next once.
//! Then it checks that the memory consumption of the program
//! well under the total size of all MownStr allocated.
//! This is a way to check that Drop is working properly.
//!
//! NB: this program relies on Linux's /proc filesystem.

use mownstr::MownStr;
use std::fs;
use std::str::FromStr;

const CAP: usize = 100_000_000;

fn main() {
    let m0 = get_vmsize();
    println!("{} MB", m0 / 1000);
    let mut v = vec![MownStr::from("hello")];
    for _ in 1..5 {
        v.pop(); //comment this line to simulate a memory leak
        let mut s = String::with_capacity(CAP);
        for _ in 0..CAP / 10 {
            s.push_str("xxxxxxxxxx");
        }
        v.push(MownStr::from(s));
        println!(
            "{} {} {}",
            v.len(),
            v[v.len() - 1].len(),
            &v[v.len() - 1][..2]
        );
    }
    let m1 = get_vmsize();
    println!("{} MB", m1 / 1000);
    let increase = (m1 - m0) as f64 / (CAP / 1000) as f64;
    if increase < 2.0 {
        println!("No memory leak, everything is ok")
    } else {
        println!("MEMORY LEAK; DROP IMPLEMENTATION OF MOWN_STR IS FAULTY");
    }
}

fn get_vmsize() -> usize {
    let txt = fs::read_to_string("/proc/self/status").expect("read proc status");
    let txt = txt.split("VmSize:").skip(1).next().unwrap();
    let txt = txt.split(" kB").next().unwrap();
    let txt = txt.trim();
    usize::from_str(txt).unwrap()
}
