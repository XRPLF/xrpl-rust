#[cfg(feature = "cli")]
fn main() -> Result<(), xrpl::cli::CliError> {
    xrpl::cli::run()
}

#[cfg(not(feature = "cli"))]
fn main() {
    #[cfg(feature = "std")]
    {
        eprintln!("CLI feature is not enabled. Recompile with --features cli");
        std::process::exit(1);
    }

    #[cfg(not(feature = "std"))]
    {
        panic!("CLI feature is not enabled. Recompile with --features cli");
    }
}
