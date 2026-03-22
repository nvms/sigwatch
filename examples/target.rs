use std::io::{self, Read};

fn main() {
    let health: f32 = 75.0;
    let max_health: f32 = 100.0;
    let position: [f32; 3] = [1.5, 2.5, 3.5];
    let score: i32 = 42;
    let alive: u8 = 1;

    println!("pid: {}", std::process::id());
    println!();
    println!("addresses:");
    println!("  health:     {:p}  (f32 = {})", &health, health);
    println!("  max_health: {:p}  (f32 = {})", &max_health, max_health);
    println!("  position:   {:p}  (vec3 = {:?})", &position, position);
    println!("  score:      {:p}  (i32 = {})", &score, score);
    println!("  alive:      {:p}  (u8  = {})", &alive, alive);
    println!();
    println!("press enter to quit...");

    let _ = io::stdin().read(&mut [0u8]);
}
