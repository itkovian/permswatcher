#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate notify;

extern crate rustc_serialize;  // for writing JSON in the log messages

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvError};

use clap::{Arg, App, ArgMatches};
use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use notify::op::{Op};

pub mod task;
pub mod pattern;
pub mod watcher;



/// The options for the permission watcher
///
///     - path: can take multiple value,
///     metadata: s
///     - log: boolean indicating if events should be logged
fn get_opts<'a>() -> ArgMatches<'a> {
    App::new("Permission watcher")
        .version(crate_version!())
        .author("Andy Georges <itkovian@gmail.com>")
        .about("Watch for permission changes in the directory tree structure")
        .arg(Arg::with_name("path")
             .short("p")
             .long("path")
             .number_of_values(1)
             .multiple(true)
             .required(true)
             .help("The paths to monitor for permission changes"))
        .arg(Arg::with_name("loglevel")
             .short("l")
             .long("loglevel")
             .required(false)
             .default_value("warning")
             .help("Set the log level (accepted values: info, debug)"))
        .get_matches()
}


/// Initialise the logger and set the log level
fn init_logger(loglevel: &str) {
    let mut log_builder = env_logger::LogBuilder::new();
    let level = match loglevel {
        "info" => log::LogLevelFilter::Info,
        "debug" => log::LogLevelFilter::Debug,
        "error" => log::LogLevelFilter::Error,
        _ => log::LogLevelFilter::Warn
    };

    log_builder
        .format(|r| format!("*** {}", r.args()))
        .filter(None, level);
    log_builder.init().expect("unable to initialize logger");
}


fn main() {
    let opts = get_opts();
    init_logger(opts.value_of("loglevel").unwrap());

    let patterns = pattern::predefined_patterns();

    let (tx, rx) = channel();
    // TODO: Scan this set of directories based on the given set of patterns.
    // It sbould thus be limited to say, the _gent_ home, data and scratch filesets
    // This base dir could be in a config file
    let initial_paths: Vec<_> = opts.values_of("path").unwrap().collect();
    let paths = scan_watch_paths(&initial_paths, &patterns);

    let mut watcher: RecommendedWatcher = Watcher::new_raw(tx).expect("Failed creating a watcher");

    for path in paths {
        watcher::add_watch(&mut watcher, &path);
    }

    let e = watch(&rx, &mut watcher, &patterns);
}


/// Scan the given set of paths for paths that should be watche
///
/// Whenever a path under the subtree matches a pattern, it should be 
/// watched.
///
/// TODO: Limit the scan to a certain depth?
fn scan_watch_paths(paths: &Vec<&str>, patterns: &Vec<pattern::Pattern>) -> Vec<PathBuf> {

    paths.iter().clone().map(|p| PathBuf::from(p)).collect()

}


/// Listen for notifications on the given paths
fn watch(rx: &Receiver<notify::RawEvent>, 
         watcher: &mut RecommendedWatcher,
         patterns: &Vec<pattern::Pattern>) -> notify::Result<()> {
    loop {
        // TODO: Add error handling for recv
        match rx.recv() {
            Ok(notify::RawEvent{path: Some(path), op: Ok(op), cookie: None}) => 
                process(rx, &path, &op, watcher, patterns),
            Ok(notify::RawEvent{path: Some(path), op: Ok(op), cookie: Some(_)}) => 
                process(rx, &path, &op, watcher, patterns),
            Ok(event) => warn!("Broken event {:?}", event),
            Err(e) => warn!("Error {}", e)
        };
    }
}


/// Process the received event
fn process(rx: &Receiver<notify::RawEvent>, 
           path: &PathBuf, 
           op: &Op, 
           watcher: &mut RecommendedWatcher,
           patterns: &Vec<pattern::Pattern>) -> () {

    match fs::metadata(path) {
        Ok(metadata) => {
            let ps: Vec<pattern::Pattern> = patterns.into_iter()
                                                    .filter(|p| p.is_match(path, op))
                                                    .cloned()
                                                    .collect(); // There should be a single match.
            match ps.len() {
                0 => warn!("No matching pattern found for operation {:?} on path {:?}", op, path), 
                1 => task::conduct_tasks(&ps[0], path, watcher, &metadata),
                _ => warn!("Multiple matching patterns found for operation {:?} on path {:?}", op, path),
            };

        },
        Err(_) => warn!("Cannot get metadata for {:?}", path)
    };
}


