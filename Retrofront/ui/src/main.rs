fn main() {
    if let Err(error) = retrofront_ui::run() {
        eprintln!("Retrofront Slint UI failed: {error}");
        std::process::exit(1);
    }
}
