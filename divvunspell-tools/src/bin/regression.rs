use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use divvunspell::report::*;
use structdiff::{Apply, Diff};

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(parse(from_os_str))]
    diff_a: PathBuf,

    #[structopt(parse(from_os_str))]
    diff_b: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();

    let file_a = File::open(&opts.diff_a)?;
    let file_b = File::open(&opts.diff_b)?;

    let mut report_a: Report = serde_json::from_reader(file_a)?;
    let report_b: Report = serde_json::from_reader(file_b)?;
    let changeset = report_a.changeset(&report_b);

    // println!("{:#?}", &changeset);
    changeset.apply(&mut report_a);
    assert_eq!(report_a, report_b);
    Ok(())
}
