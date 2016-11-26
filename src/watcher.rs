/// MIT License (C) 2016 Andy Georges
///
/// Watchers for the filesystem

extern crate glob;
extern crate notify;
extern crate slog_scope;

use self::glob::glob;
use std::path::Path;
use std::path::PathBuf;

use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use pattern::Pattern;


/// Add monitoring for the given path, no recursion
pub fn add_watch(watcher: &mut RecommendedWatcher, path: &PathBuf) -> () {
        match Path::new(path).canonicalize() {
            Ok(canonical_path) => {
                watcher.watch(&canonical_path, RecursiveMode::NonRecursive).expect("Cannot watch path");
                info!(slog_scope::logger(), "Watching path {:?}", canonical_path);
            },
            Err(_)             => {
                info!(slog_scope::logger(), "Invalid path");
                return;
            }
        }
}

/// Rescan the directory structure, starting at the given path.
///
/// We add a watcher for every file that matches one of the patterns.
/// Since duplicate watchers are only considered once, this is ok.
pub fn rescan(path: &PathBuf, 
              watcher: &mut RecommendedWatcher,
              patterns: &Vec<Pattern>) -> () {

    let mut globPath = path.clone();
    globPath.push("/*");
    info!(slog_scope::logger(), "Scanning {:?} for entries", path);

    for entry in glob(globPath.to_str().unwrap()).unwrap().filter_map(Result::ok) {
        let path = PathBuf::from(entry);
        for pattern in patterns {
            if pattern.is_path_match(&path) {
                info!(slog_scope::logger(), "Found a matching pattern {:?} for path {:?}", pattern, path);
                add_watch(watcher, &path);
            }
        }
    }
}

