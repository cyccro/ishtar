use std::{env, ffi::OsString};

use ishtar::Ishtar;
mod helpers;
mod ishtar;
fn main() {
    let args = env::args().collect::<Vec<String>>();
    let flag = if args.len() > 1 {
        args[1] == ".keeplog"
    } else {
        false
    };
    let mut ishtar = Ishtar::new(flag).unwrap();
    ishtar.run().unwrap();
}
