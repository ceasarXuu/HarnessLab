fn main() {
    match harnesslab_cli::run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            std::process::exit(3);
        }
    }
}
