fn main() {
    if let Err(error) = harnesslab_cli::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
