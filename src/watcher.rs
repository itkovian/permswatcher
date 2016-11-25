/// MIT License (C) 2016 Andy Georges
///
/// Watchers for the filesystem

use std::path::Path;
use std::path::PathBuf;

extern crate notify;
extern crate slog_scope;

use notify::{RecommendedWatcher, Watcher, RecursiveMode};


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


pub fn rescan(path: &PathBuf, watcher: &mut RecommendedWatcher) -> () {}

