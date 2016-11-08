/// MIT License (C) 2016 Andy Georges
///
/// Watchers for the filesystem

use std::path::Path;
use std::path::PathBuf;

extern crate notify;

use notify::{RecommendedWatcher, Watcher, RecursiveMode};


/// Add monitoring for the given path, no recursion
pub fn add_watch(watcher: &mut RecommendedWatcher, path: &PathBuf) -> () {
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


pub fn rescan(path: &PathBuf, watcher: &RecommendedWatcher) -> () {}

