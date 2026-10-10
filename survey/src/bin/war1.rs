#[path = "../war1/mod.rs"]
mod war1;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Err(failure) = war1::cli::main_args(&args) {
        eprintln!("{}", failure.message);
        std::process::exit(failure.exit_code);
    }
}
