/// MIT License (C) 2016 Andy Georges
///
/// Pattern to compare directories with an allow taking the correct action.
///
/// A pattern is defined as a regular expression, annotated with
/// a modification operation. When both match, a set of tasks should be 
/// carried out.

use std::path::PathBuf;

extern crate notify;
extern crate regex;

use self::notify::op::Op;
use self::regex::Regex;

use task::Task;

/// Pattern type
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    name: String,
    op: Op,
    pattern: Regex,
    pub permission_mask: u32,
    pub tasks: Vec<Task>,
}


impl Pattern {

    pub fn new(name: String, op: Op, pattern: Regex, permission_mask: u32, tasks: Vec<Task>) -> Pattern {
        Pattern {
            name: name,
            op: op,
            pattern: pattern,
            permission_mask: permission_mask,
            tasks: tasks,
        }
    }

    // Matching a given path?
    pub fn is_match(&self, path: &PathBuf, op: &notify::op::Op) -> bool {
        self.pattern.is_match(&path.to_str().unwrap()) && self.op.eq(op)
    }
}


/// Predefined patterns and associated tasks
pub fn predefined_patterns() -> Vec<Pattern> {
    let mut m: Vec<Pattern> = Vec::new();

    // user grouping directories
    m.push(Pattern::new(
        String::from("USR grouping dir"),
        notify::op::CREATE,
        Regex::new("^/.*/vsc[0-9]{3}$").unwrap(),
        0o750,
        vec![Task::AddWatcher, Task::Rescan]));

    // user personal directories
    m.push(Pattern::new(
        String::from("USR personal dir"),
        notify::op::CHMOD,
        Regex::new("^/.*/vsc[0-9]{3}/vsc[0-9]{5}$").unwrap(),
        0o750,
        vec![Task::PermissionCheck]));

    m
}


