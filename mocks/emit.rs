// Print to STDOUT and STDERR.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("{}", &args[0]);
    eprintln!("{}", &args[1]);
}
