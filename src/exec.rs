use std::env;
use std::process::{Command, Stdio};

use indexmap::IndexMap;
use regex::Regex;

use crate::entry::{Entry, OPTIONS};
use crate::history::save_history;

lazy_static! {
    static ref RE_EXEC_OPT: Regex = Regex::new(r"\s*%\w").unwrap();
}

pub fn exec_pretrimmed(cmd: &str) {
    exec_raw(cmd.trim());
}

pub fn execute(pathstr: &str, entries: &mut IndexMap<String, Entry>) {
    let entry = entries.get_mut(pathstr).unwrap();
    entry.count += 1;

    if !entry.desktop {
        exec_command(entry);
    } else if !entry.terminal {
        exec_app(entry);
    } else {
        exec_term(entry);
    }

    save_history(entries);
}

// Execute command from bin entry
fn exec_command(entry: &Entry) {
    let cmd = entry.exec.trim();
    exec_raw(cmd);
}

// Run app from desktop entry, not terminal app
fn exec_app(entry: &Entry) {
    let cmd = entry.exec.trim();
    exec_raw(&RE_EXEC_OPT.replace_all(cmd, ""));
}

// Run terminal app from desktop entry
fn exec_term(entry: &Entry) {
    let cmd = RE_EXEC_OPT.replace_all(entry.exec.trim(), "").into_owned();

    let mut term_cmd: Vec<String> = Vec::new();
    match &OPTIONS.terminal_command {
        Some(val) => {
            term_cmd.extend(shlex::split(val).expect("Failed to parse --terminal-command option"))
        }
        None => match env::var_os("TERM") {
            Some(val) => term_cmd = vec![val.to_str().unwrap().to_string(), "-e".to_string()],
            None => term_cmd = vec!["alacritty".to_string(), "-e".to_string()],
        },
    }
    term_cmd.push(cmd);

    // convert Vec<String> to Iter<&str> and join to a single String
    let command = shlex::try_join(term_cmd.iter().map(String::as_str)).unwrap();
    exec_raw(&command);
}

fn exec_raw(cmd: &str) {
    // we don't want to wait on it, we want it to keep running while we exit
    #[expect(clippy::zombie_processes)]
    Command::new("setsid")
        .arg("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start command");
}
