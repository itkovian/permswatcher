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

    let (tx, rx) = channel();
    let paths = opts.values_of("path").unwrap();
    let mut watcher: RecommendedWatcher = Watcher::new_raw(tx).expect("Failed creating a watcher");

    for path in paths {
        add_watch(&mut watcher, & PathBuf::from(path));
    }

    let e = watch(&rx, &mut watcher);
}


/// Listen for notifications on the given paths
fn watch(rx: &Receiver<notify::RawEvent>, watcher: &mut RecommendedWatcher) -> notify::Result<()> {
    loop {
        // TODO: Add error handling for recv
        match rx.recv() {
            Ok(notify::RawEvent{path: Some(path), op: Ok(op), cookie: None}) => 
                process(rx, &path, &op, watcher),
            Ok(notify::RawEvent{path: Some(path), op: Ok(op), cookie: Some(_)}) => 
                process(rx, &path, &op, watcher),
            Ok(event) => warn!("Broken event {:?}", event),
            Err(e) => warn!("Error {}", e)
        };
    }
}


/// Process the received event
fn process(rx: &Receiver<notify::RawEvent>, path: &PathBuf, op: &Op, watcher: &mut RecommendedWatcher) -> () {

    match *op {
        notify::op::CHMOD => {
            match fs::metadata(path) {
                Ok(metadata) => 
                    println!("Current permissions for {:?}: {:o}", path, metadata.permissions().mode()),
                Err(_)       => warn!("Cannot get metadata for {:?}", path)
            };
        },
        notify::op::CREATE => {
            match fs::metadata(path) {
                Ok(metadata) => if metadata.file_type().is_dir() {
                                    add_watch(watcher, path);
                                },
                Err(_)       => warn!("Cannot get metadata for {:?}", path)
            };
        },
        _     => println!("Not watching op {:?}", op)
    };

}
