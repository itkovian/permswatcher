/// MIT License (C) 2016 Andy Georges
///
/// Tasks to conduct when a monitoring event is matching a watched directory or file
///
/// A Task defines which actions should be taken, e.g., 
/// - log an event
/// - notify an admin
/// - rescan a directory
/// - add a watch for the given directory

extern crate notify;

use notify::{RecommendedWatcher};

use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use watcher;
use pattern::{Pattern};

/// Task type
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Task {
    PermissionCheck, // check if the permissions are as expected
    AddWatcher,      // Add the changed file to the watchlist for future monitoring
    Rescan,          // Check all the files in this directory, if any
}


/// Perform the tasks
pub fn conduct_tasks(pattern: &Pattern,
                 path: &PathBuf, 
                 watcher: &mut RecommendedWatcher,
                 metadata: &Metadata,
                 patterns: &Vec<Pattern>) -> () {

    for task in pattern.tasks.iter().clone() {
    
        match *task {
           Task::PermissionCheck => permission_check(path, metadata, pattern.permission_mask),
           Task::AddWatcher => watcher::add_watch(watcher, path),
           Task::Rescan => {
               let new_paths = watcher::rescan(path, watcher, patterns);
           }
        }

    }
}


fn permission_check(path: &PathBuf, metadata: &Metadata, permission_mask: u32) -> () {

    if permission_mask | metadata.permissions().mode() != permission_mask {
        println!("Permissions on path {:?} are {:?} instead of expected {:?}",
                 path, metadata.permissions().mode(), permission_mask);
    }

}


