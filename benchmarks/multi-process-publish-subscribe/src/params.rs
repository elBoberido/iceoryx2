use clap::Parser;

/// Multi-process benchmark for the publish-subscribe messaging pattern
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Params {
    /// For internal use only
    #[arg(long)]
    pub leader: bool,
    /// For internal use only
    #[arg(long)]
    pub follower: bool,
    /// The number of iterations to run
    #[arg(long, default_value_t = 10_000_000)]
    pub iterations: usize,
    // TODO add mode with poll and event
}
