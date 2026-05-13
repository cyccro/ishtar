use ishtar::Ishtar;
mod plugins;
mod helpers;
mod ishtar;
mod widgets;
fn main() {
    let mut ishtar = Ishtar::new();
    ishtar.run().unwrap();
}
