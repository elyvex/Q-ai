fn main() {
    use clap::Parser;
    let cli = cli::Cli::parse();
    let code = cli::dispatch(cli);
    std::process::exit(code);
}
