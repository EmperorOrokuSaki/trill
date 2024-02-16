use crate::args::Args;
use crate::cli::Cli;


fn main() {
    let args = Args::parse();
    let cli = Cli::new(args);
}