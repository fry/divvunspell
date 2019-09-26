use std::error::Error;
use std::time::{Instant, SystemTime};

use clap::{App, AppSettings, Arg};
use divvunspell::archive::{BoxSpellerArchive, ZipSpellerArchive};
use divvunspell::report::*;
use divvunspell::speller::SpellerConfig;
use divvunspell::transducer::thfst::ThfstTransducer;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

static CFG: SpellerConfig = SpellerConfig {
    max_weight: Some(50000.0),
    n_best: Some(10),
    beam: None,
    pool_max: 128,
    pool_start: 128,
    seen_node_sample_rate: 15,
    with_caps: true,
};

fn load_words(
    path: &str,
    max_words: Option<usize>,
) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .delimiter(b'\t')
        .has_headers(false)
        .flexible(true)
        .from_path(path)?;

    Ok(rdr
        .records()
        .filter_map(Result::ok)
        .filter_map(|r| {
            r.get(0)
                .and_then(|x| r.get(1).map(|y| (x.to_string(), y.to_string())))
        })
        .take(max_words.unwrap_or(std::usize::MAX))
        .collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("divvunspell-accuracy")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Accuracy testing for DivvunSpell.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .takes_value(true)
                .help("Provide JSON config file to override test defaults"),
        )
        .arg(
            Arg::with_name("words")
                .value_name("WORDS")
                .help("The 'input -> expected' list in tab-delimited value file (TSV)"),
        )
        // .arg(
        //     Arg::with_name("bhfst")
        //         .value_name("BHFST")
        //         .help("Use the given BHFST file"),
        // )
        .arg(
            Arg::with_name("zhfst")
                .value_name("ZHFST")
                .help("Use the given ZHFST file"),
        )
        .arg(
            Arg::with_name("json-output")
                .short("o")
                .value_name("JSON-OUTPUT")
                .help("The file path for the JSON report output"),
        )
        .arg(
            Arg::with_name("max-words")
                .short("w")
                .takes_value(true)
                .help("Truncate typos list to max number of words specified"),
        )
        .get_matches();

    let cfg: SpellerConfig = match matches.value_of("config") {
        Some(path) => {
            let file = std::fs::File::open(path)?;
            serde_json::from_reader(file)?
        }
        None => CFG.clone(),
    };

    // let archive: BoxSpellerArchive<ThfstTransducer, ThfstTransducer> =
    //     match matches.value_of("bhfst") {
    //         Some(path) => BoxSpellerArchive::open(path)?,
    //         None => {
    //             eprintln!("No BHFST found for given path; aborting.");
    //             std::process::exit(1);
    //         }
    //     };

    let archive = match matches.value_of("zhfst") {
        Some(path) => ZipSpellerArchive::open(path)?,
        None => {
            eprintln!("No ZHFST found for given path; aborting.");
            std::process::exit(1);
        }
    };

    let words = match matches.value_of("words") {
        Some(path) => load_words(
            path,
            matches
                .value_of("max-words")
                .and_then(|x| x.parse::<usize>().ok()),
        )?,
        None => {
            eprintln!("No word list for given path; aborting.");
            std::process::exit(1);
        }
    };

    let pb = ProgressBar::new(words.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{pos}/{len} [{percent}%] {wide_bar} {elapsed_precise}"),
    );

    let start_time = Instant::now();
    let results = words
        .par_iter()
        .progress_with(pb)
        .map(|(input, expected)| {
            let now = Instant::now();
            let suggestions = archive.speller().suggest_with_config(&input, &cfg);
            let now = now.elapsed();

            let time = Time {
                secs: now.as_secs(),
                subsec_nanos: now.subsec_nanos(),
            };

            let position = suggestions.iter().position(|x| x.value == expected);

            AccuracyResult {
                input: input.clone(),
                expected: expected.clone(),
                time,
                suggestions: suggestions.into_iter().map(|x| Suggestion {
                    value: x.value.to_string(),
                    weight: x.weight
                }).collect(),
                position,
            }
        })
        .collect::<Vec<_>>();

    let now = start_time.elapsed();
    let total_time = Time {
        secs: now.as_secs(),
        subsec_nanos: now.subsec_nanos(),
    };
    let now_date = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let start_timestamp = Time {
        secs: now_date.as_secs(),
        subsec_nanos: now_date.subsec_nanos(),
    };

    let summary = Summary::new(&results);
    println!("{}", summary);

    if let Some(path) = matches.value_of("json-output") {
        let output = std::fs::File::create(path)?;
        let report = Report {
            metadata: archive.metadata().map(|x| x.to_owned()),
            config: cfg,
            summary,
            results,
            start_timestamp,
            total_time,
        };
        println!("Writing JSON report…");
        serde_json::to_writer_pretty(output, &report)?;
    };

    println!("Done!");
    Ok(())
}
