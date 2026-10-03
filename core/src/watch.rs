use crate::{scan, Root, SkillState};
use notify_debouncer_mini::{
    new_debouncer,
    notify::{self, RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer,
};
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

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
    on_change: impl Fn(Vec<SkillState>) + Send + 'static,
) -> notify::Result<WatchGuard> {
    let roots = roots.to_vec();
    let (sender, receiver) = mpsc::channel();
    let notify_sender = sender.clone();
    let mut debouncer: Debouncer<RecommendedWatcher> = new_debouncer(
        Duration::from_millis(300),
        move |result| { let _ = notify_sender.send(Message::Changed(result)); },
    )?;
    let mut watched = Vec::<PathBuf>::new();
    for root in &roots {
        // macOS FSEvents can miss a watched directory's own deletion. Its parent catches
        // deletion/recreation; only its immediate entries are watched, not sibling trees.
        if let Some(parent) = root.path.parent().filter(|parent| parent.exists()) {
            if let Err(error) = debouncer.watcher().watch(parent, RecursiveMode::NonRecursive) {
                eprintln!("skill watcher parent {}: {error}", parent.display());
            }
        }
        if root.path.exists() {
            match debouncer.watcher().watch(&root.path, RecursiveMode::Recursive) {
                Ok(()) => watched.push(root.path.clone()),
                Err(error) => eprintln!("skill watcher root {}: {error}", root.path.display()),
            }
        }
    }
    let worker = thread::spawn(move || {
        let mut guard = debouncer;
        while let Ok(message) = receiver.recv() {
            match message {
                Message::Stop => break,
                Message::Changed(result) => {
                    if let Err(error) = result {
                        eprintln!("skill watcher: {error}");
                    }
                    // ponytail: full scans are cheap at this scale; event interpretation belongs
                    // here only if measured scan latency exceeds the debounce window.
                    match scan(&roots) {
                        Ok(state) => on_change(state),
                        Err(error) => eprintln!("skill watcher rescan: {error}"),
                    }
                    // FSEvents drops a watched root when it disappears. Reattach if its parent
                    // later reports recreation; don't crash when permissions deny access.
                    for root in &roots {
                        if root.path.exists() && !watched.contains(&root.path) {
                            match guard.watcher().watch(&root.path, RecursiveMode::Recursive) {
                                Ok(()) => watched.push(root.path.clone()),
                                Err(error) => eprintln!("skill watcher reattach: {error}"),
                            }
                        } else if !root.path.exists() {
                            watched.retain(|path| path != &root.path);
                        }
                    }
                }
            }
        }
    });
    Ok(WatchGuard { sender, worker: Some(worker) })
}
