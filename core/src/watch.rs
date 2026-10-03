use crate::{configured_roots, default_managed_roots, scan_state, Root, ScanState};
use notify_debouncer_mini::{
    new_debouncer,
    notify::{self, RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer,
};
use std::{collections::BTreeMap, path::{Path, PathBuf}, sync::mpsc, thread, time::Duration};

enum Message {
    Changed(DebounceEventResult),
    Stop,
}

/// Dropping this guard stops the watcher and its scan worker.
pub struct WatchGuard {
    sender: mpsc::Sender<Message>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Drop for WatchGuard {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub fn watch(
    roots: &[Root],
    config: &Path,
    home: &Path,
    on_change: impl Fn(ScanState) + Send + 'static,
) -> notify::Result<WatchGuard> {
    let roots = roots.to_vec();
    let config = config.to_path_buf();
    let home = home.to_path_buf();
    let (sender, receiver) = mpsc::channel();
    let notify_sender = sender.clone();
    let mut debouncer: Debouncer<RecommendedWatcher> = new_debouncer(
        Duration::from_millis(300),
        move |result| { let _ = notify_sender.send(Message::Changed(result)); },
    )?;
    let mut watched = BTreeMap::new();
    rearm(&mut debouncer, &mut watched, &roots, &config, &home);
    let worker = thread::spawn(move || {
        let mut guard = debouncer;
        while let Ok(message) = receiver.recv() {
            match message {
                Message::Stop => break,
                Message::Changed(result) => {
                    if let Err(error) = result { eprintln!("skill watcher: {error}"); }
                    // ponytail: re-discover and re-arm every debounce; incremental event bookkeeping only if measured scans become costly.
                    rearm(&mut guard, &mut watched, &roots, &config, &home);
                    let managed = match default_managed_roots(&home) {
                        Ok(managed) => managed,
                        Err(error) => { eprintln!("skill watcher managed roots: {error}"); continue; }
                    };
                    match scan_state(&roots, &config, &managed) {
                        Ok(state) => on_change(state),
                        Err(error) => eprintln!("skill watcher rescan: {error}"),
                    }
                }
            }
        }
    });
    Ok(WatchGuard { sender, worker: Some(worker) })
}

fn rearm(debouncer: &mut Debouncer<RecommendedWatcher>, watched: &mut BTreeMap<PathBuf, RecursiveMode>, roots: &[Root], config: &Path, home: &Path) {
    let (configured, error, _) = configured_roots(roots, config);
    if let Some(error) = error { eprintln!("skill watcher config: {error}"); }
    let mut desired = BTreeMap::new();
    for path in configured.into_iter().map(|root| root.path)
        .chain([".claude/plugins/cache", ".codex/plugins/cache", ".cursor/plugins/cache", ".omp/plugins/cache/plugins", ".codex/skills/.system"].into_iter().map(|cache| home.join(cache)))
        .chain(std::iter::once(config.to_path_buf())) {
        if let Some(parent) = path.ancestors().skip(1).find(|parent| parent.is_dir()) {
            desired.entry(parent.to_path_buf()).or_insert(RecursiveMode::NonRecursive);
        }
        if path.is_dir() { desired.insert(path.to_path_buf(), RecursiveMode::Recursive); }
    }
    for (path, mode) in watched.iter() {
        if desired.get(path) != Some(mode) {
            if let Err(error) = debouncer.watcher().unwatch(path) { eprintln!("skill watcher unwatch {}: {error}", path.display()); }
        }
    }
    watched.retain(|path, mode| desired.get(path) == Some(mode));
    for (path, mode) in desired {
        if watched.contains_key(&path) { continue; }
        match debouncer.watcher().watch(&path, mode) {
            Ok(()) => { watched.insert(path, mode); }
            Err(error) => eprintln!("skill watcher watch {}: {error}", path.display()),
        }
    }
}
