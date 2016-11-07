#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate notify;
extern crate regex;

extern crate rustc_serialize;  // for writing JSON in the log messages

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvError};

use clap::{Arg, App, ArgMatches};
use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use notify::op::{Op};
use regex::{Regex};


/// Tasks that need to be carried out depending on the type of change we detect and the
/// type of file that is changed.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Task {
    PermissionCheck, // check if the permissions are as expected
    AddWatcher,      // Add the changed file to the watchlist for future monitoring
    Rescan,          // Check all the files in this directory, if any
}


/// Patterns and associated tasks
#[derive(Clone, Debug, Eq, PartialEq)]
struct Pattern {
    name: String,
    op: notify::op::Op,
    pattern: Regex,
    tasks: Vec<Task>,
}

/// Pattern implementation
impl Pattern {

    fn new(name: String, op: Op, pattern: Regex, tasks: Vec<Task>) -> Pattern {
        Pattern {
            name: name,
            op: op,
            pattern: pattern,
            tasks: tasks,
        }
    }

    /// Mathing a given path?
    fn is_match(&self, path: &PathBuf) -> bool {
        self.pattern.is_match(&path.to_str().unwrap())
    }
}


/// Predefined patterns and associated tasks
fn predefined_patterns() -> Vec<Pattern> {
    let mut m: Vec<Pattern> = Vec::new();

    // USR grouping directories
    m.push(Pattern::new(
        String::from("USR grouping dir"),
        notify::op::CREATE,
        Regex::new("^/.*/vsc[0-9]{3}$").unwrap(),
        vec![Task::AddWatcher, Task::Rescan]));

    // USR personal directories
    m.push(Pattern::new(
        String::from("USR personal dir"),
        notify::op::CHMOD,
        Regex::new("^/.*/vsc[0-9]{3}/vsc[0-9]{5}$").unwrap(),
        vec![Task::PermissionCheck]));

    m
}


/// The options for the permission watcher
///
///     - path: can take multiple values
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

/// Add monitoring for the given path, no recursion
fn add_watch(watcher: &mut RecommendedWatcher, path: &PathBuf) -> () {
        match Path::new(path).canonicalize() {
            Ok(canonical_path) => {
                watcher.watch(&canonical_path, RecursiveMode::NonRecursive).expect("Cannot watch path");
                info!("Watching path {:?}", canonical_path);
            },
            Err(_)             => {
                info!("Invalid path");
                return;
            }
        }
}


fn main() {
    let opts = get_opts();
    init_logger(opts.value_of("loglevel").unwrap());

    let patterns = predefined_patterns();

    let (tx, rx) = channel();
    let paths = opts.values_of("path").unwrap();
    let mut watcher: RecommendedWatcher = Watcher::new_raw(tx).expect("Failed creating a watcher");

    for path in paths {
        add_watch(&mut watcher, & PathBuf::from(path));
    }

    let e = watch(&rx, &mut watcher, &patterns);
}


/// Listen for notifications on the given paths
fn watch(rx: &Receiver<notify::RawEvent>, 
         watcher: &mut RecommendedWatcher,
         patterns: &Vec<Pattern>) -> notify::Result<()> {
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
           patterns: &Vec<Pattern>) -> () {

    match fs::metadata(path) {
        Ok(metadata) => {
            match *op {
                notify::op::CHMOD => {
                    println!("Current permissions for {:?}: {:o}", path, metadata.permissions().mode());
                },
                notify::op::CREATE => process_create(path, &metadata, watcher, patterns),
                _ => println!("Not watching op {:?}", op)
            }
        }
       Err(_)       => warn!("Cannot get metadata for {:?}", path)
    };
}


/// Process newly created directories or files
fn process_create(path: &PathBuf, 
                  metadata: &fs::Metadata, 
                  watcher: &mut RecommendedWatcher,
                  patterns: &Vec<Pattern>) -> () {

    // Are we dealing with a directory here?
    if metadata.file_type().is_dir() {
        
        let ps: Vec<Pattern> = patterns.into_iter().filter(|p| p.is_match(path)).cloned().collect(); // There should be a single match.

        if ps.len() > 1 {
            warn!("Multiple patterns match for path {:?}", path);
        }
        else {
            println!("Found the following match: {:?}", ps[0]);
        }

    }

}
