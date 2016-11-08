/// MIT License (C) 2016 Andy Georges
///
/// Tasks to conduct when a monitoring event is matching a watched directory or file
///
/// A Task defines which actions should be taken, e.g., 
/// - log an event
/// - notify an admin
/// - rescan a directory
/// - add a watch for the given directory


/// Task type
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Task {
    PermissionCheck, // check if the permissions are as expected
    AddWatcher,      // Add the changed file to the watchlist for future monitoring
    Rescan,          // Check all the files in this directory, if any
}



