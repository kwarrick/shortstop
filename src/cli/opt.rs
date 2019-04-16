use std::path::PathBuf;

use super::StructOpt;

/// Command line options
#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(name = "PROG", parse(from_os_str))]
    pub prog: PathBuf,
    #[structopt(name = "ARGS")]
    pub args: Vec<String>,
}
