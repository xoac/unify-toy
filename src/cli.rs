use clap::Parser;
use std::path::PathBuf;

/// Unify information about hotels data
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Input csv files that need to be unified
    #[clap(long)]
    pub input: PathBuf,
    /// The json like file with hotels details
    #[clap(long)]
    pub hotels: PathBuf,
    /// csv file with room details
    #[clap(long)]
    pub room_names: PathBuf,
    /// Output file where data will be saved
    #[clap(long)]
    pub output: PathBuf, // TODO this could be changed into Option and use io::stdout
}
