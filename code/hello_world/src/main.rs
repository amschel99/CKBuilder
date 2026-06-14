#![no_std]
#![no_main]
#![allow(unexpected_cfgs)]

use ckb_std::debug;

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

pub fn program_entry() -> i8 {
    debug!("Hello, World!");
    0
}
