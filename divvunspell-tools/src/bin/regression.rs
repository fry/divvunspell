use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use divvunspell::report::*;
use structdiff::{Apply, Diff, Field, types::VecAction};

#[derive(Debug, StructOpt)]
struct Opts {
    #[structopt(parse(from_os_str))]
    diff_a: PathBuf,

    #[structopt(parse(from_os_str))]
    diff_b: PathBuf,
}

#[derive(Debug)]
enum Difference {
    Position { previous: u8, current: u8 }
}

fn process_actions(results: &Vec<AccuracyResult>, actions: Vec<VecAction<AccuracyResult>>) -> Result<(), Box<dyn Error>> {
    // let mut x = 0;
    
    let actions = actions.into_iter().filter_map(|action| {
        use VecAction::*;

        match action {
            Set(index, field) => {
                Some((index, field))
            },
            _ => None
            // Push
            // Truncate
            // Append
        }
    });

    actions.for_each(|(index, field)| {
        use Field::*;

        let result = &results[index];

        match field {
            None => {},
            Set(value) => {

            },
            Changes(changes) => {
                match changes.position {
                    None => {},
                    Changes(changes) => {
                        println!("{}: {:?} -> {:?}", result.input, result.position, changes);
                    }
                    Set(value) => {
                        println!("{}: {:?} -> {:?}", result.input, result.position, value);
                    },
                    Actions(actions) => {
                        println!("{}: {:?} -> {:?}", result.input, result.position, actions);
                    }
                }
            }
            Actions(actions) => {

            },
        }
    });

    // println!("\n{}/{}", x, results.len());
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();

    let file_a = File::open(&opts.diff_a)?;
    let file_b = File::open(&opts.diff_b)?;

    let report_a: Report = serde_json::from_reader(file_a)?;
    let report_b: Report = serde_json::from_reader(file_b)?;
    let changeset = report_a.results.changeset(&report_b.results);

    match changeset {
        Field::None => {
            println!("No changes.");
            Ok(())    
        },
        Field::Actions(actions) => {
            process_actions(&report_a.results, actions)
        },
        _ => Ok(())
    }

    // println!("{:#?}", &changeset);
    // changeset.apply(&mut results);
    // assert_eq!(&report_a.results, &results);
    // Ok(())
}
