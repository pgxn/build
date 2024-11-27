// Simple app that returns an error.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("DED: {}", &args[1..].join(" "));
    std::process::exit(2)
}
