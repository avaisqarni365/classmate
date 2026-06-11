fn main() {
    if let Err(err) = classmate_lib::headless::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
