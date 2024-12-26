use ishtar::Ishtar;
mod helpers;
mod ishtar;
fn main() {
    let mut ishtar = Ishtar::new();
    ishtar.run().unwrap();
}
