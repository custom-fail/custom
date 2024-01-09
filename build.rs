
fn main() {
    #[cfg(not(feature = "http-interactions"))]
    #[cfg(not(feature = "custom-clients"))]
    #[cfg(not(feature = "gateway"))]
    #[cfg(not(feature = "tasks"))]
    {
        eprintln!("You need to specify features before compiling code\n\tTo compile everything try using `cargo build --features all`");
        std::process::exit(1);
    }
}