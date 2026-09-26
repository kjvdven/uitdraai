#[cfg_attr(not(test), expect(dead_code, reason = "wired up once the CLI lands"))]
mod render;
#[cfg_attr(not(test), expect(dead_code, reason = "wired up once the CLI lands"))]
mod theme;

fn main() {
    println!("Hello, world!");
}
