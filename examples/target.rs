use std::io::{self, BufRead};

fn main() {
    let mut health: f32 = 75.0;
    let max_health: f32 = 100.0;
    let mut position: [f32; 3] = [1.5, 2.5, 3.5];
    let mut score: i32 = 42;
    let alive: u8 = 1;

    println!("pid: {}", std::process::id());
    println!();
    print_state(&health, &max_health, &position, &score, &alive);
    println!();
    println!("commands: damage, heal, move, score, quit");
    println!();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        match line.trim() {
            "damage" => {
                health = (health - 25.0).max(0.0);
                println!("  health -> {health}");
            }
            "heal" => {
                health = (health + 10.0).min(max_health);
                println!("  health -> {health}");
            }
            "move" => {
                position[0] += 1.0;
                position[1] += 0.5;
                position[2] -= 0.3;
                println!("  position -> {:?}", position);
            }
            "score" => {
                score += 100;
                println!("  score -> {score}");
            }
            "quit" | "q" => break,
            "status" => print_state(&health, &max_health, &position, &score, &alive),
            _ => println!("  unknown: {}", line.trim()),
        }
    }
}

fn print_state(health: &f32, max_health: &f32, position: &[f32; 3], score: &i32, alive: &u8) {
    println!("addresses:");
    println!("  health:     {:p}  (f32 = {health})", health);
    println!("  max_health: {:p}  (f32 = {max_health})", max_health);
    println!("  position:   {:p}  (vec3 = {position:?})", position);
    println!("  score:      {:p}  (i32 = {score})", score);
    println!("  alive:      {:p}  (u8  = {alive})", alive);
}
