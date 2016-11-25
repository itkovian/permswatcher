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
    name: String,               // for log purposes
    op: Op,                     // the change operation that is listening for on this pattern
    base: String,               // the base directory from where we start looking for the pattern
    pattern: Regex,             // regex identifying the directory or file
    pub permission_mask: u32,   // detected permissions should be at most this
    pub tasks: Vec<Task>,       // what to do when the op and pattern match
}


impl Pattern {

    pub fn new(name: String, 
               op: Op, 
               base: String,
               pattern: String,                 // the Regex will be base + pattern
               permission_mask: u32, 
               tasks: Vec<Task>) -> Pattern {
        Pattern {
            name: name,
            op: op,
            base: base.clone(),
            pattern: Regex::new(&(String::from("^") + &base + &pattern)).unwrap(),
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

    // TODO: the base should be picked up from the config file (on a per filesystem basis)

    // user grouping directories
    m.push(Pattern::new(
        String::from("USR grouping dir"),
        notify::op::CREATE,
        String::from("/vulpix/home/gent"),
        String::from("/vsc[0-9]{3}$"),
        0o750,
        vec![Task::AddWatcher, Task::Rescan]));

    // user personal directories chmod
    m.push(Pattern::new(
        String::from("USR personal dir"),
        notify::op::CHMOD,
        String::from("/vulpix/home/gent"),
        String::from("/vsc[0-9]{3}/vsc[0-9]{5}$"),
        0o750,
        vec![Task::PermissionCheck]));

    // user personal directories creation
    m.push(Pattern::new(
        String::from("USR personal dir"),
        notify::op::CREATE,
        String::from("/vulpix/home/gent"),
        String::from("/vsc[0-9]{3}/vsc[0-9]{5}$"),
        0o750,
        vec![Task::AddWatcher]));

    // user .ssh authorized_keys file
    m.push(Pattern::new(
        String::from("USR authorized_keys file"),
        notify::op::CHMOD,
        String::from("/vulpix/home/gent"),
        String::from("/vsc[0-9]{3}/vsc[0-9]{5}/.ssh/authorized_keys$"),
        0o600,
        vec![Task::PermissionCheck]));

    m
}


