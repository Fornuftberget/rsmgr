use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "rsmgr", about = "Förnuftberget Resource Manager")]
pub struct Config {
    /// Path to ssh-key, if dir then we will look for a key, if omitted then we will try to use ssh-agent
    #[structopt(name = "ssh_key", short = "k", long, parse(from_os_str))]
    pub ssh_key: Option<PathBuf>,

    /// How many workers(threads) to use for resource updates
    #[structopt(name = "num_workers", short = "n", long)]
    pub num_workers: Option<usize>,

    /// Git base url
    #[structopt(
        name = "base_url",
        short = "u",
        long,
        default_value = "git@github.com:Fornuftberget/"
    )]
    pub base_url: String,

    /// Base path to resources
    #[structopt(
        name = "base_path",
        short = "b",
        long,
        default_value = "resources/",
        parse(from_os_str)
    )]
    pub base_path: PathBuf,

    /// Path to server.cfg
    #[structopt(
        name = "config",
        short = "c",
        long,
        default_value = "server.cfg",
        parse(from_os_str)
    )]
    pub config: PathBuf,

    /// Verbose output
    #[structopt(name = "verbose", short = "v", long)]
    pub verbose: bool,

    /// Discard any local changes
    #[structopt(name = "discard", long)]
    pub discard: bool,
}
