//! Exécution de `git` en lecture seule, avec délai maximal.

use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(4);
const FLAGS: [&str; 6] = ["-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null", "-c", "core.untrackedCache=false"];

/// Démarre `git` en lecture seule dans `folder`, sortie standard capturée.
fn spawn_git(folder: &Path, args: &[&str]) -> Option<Child> {
    Command::new("git")
        .args(FLAGS)
        .arg("-C")
        .arg(folder)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .ok()
}

/// Attend la fin du processus ; le tue et retourne `None` après le délai maximal.
fn wait_in_time(child: &mut Child) -> Option<ExitStatus> {
    let started = Instant::now();
    loop {
        match child.try_wait().ok()? {
            Some(status) => return Some(status),
            None if started.elapsed() > TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => sleep(Duration::from_millis(5)),
        }
    }
}

/// Lance `git` dans `folder` ; retourne la sortie standard si la commande réussit à temps.
pub fn run_git(folder: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let mut child = spawn_git(folder, args)?;
    let mut pipe = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        let _ = pipe.read_to_end(&mut out);
        out
    });
    let status = wait_in_time(&mut child)?;
    let out = reader.join().ok()?;
    status.success().then_some(out)
}
