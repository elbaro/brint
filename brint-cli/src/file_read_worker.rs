use slog_scope::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

type TokioTx<T> = tokio::sync::mpsc::UnboundedSender<T>;
type CrossbeamRx<T> = crossbeam::channel::Receiver<T>;

pub struct FileReadWorker {
    path_tx: TokioTx<Option<PathBuf>>,
    result_rx: CrossbeamRx<(PathBuf, String)>,
}

impl FileReadWorker {
    pub fn run_in_background_thread() -> Self {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_current_thread()
                // tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap(),
        );
        let (path_tx, mut path_rx) = tokio::sync::mpsc::unbounded_channel::<Option<PathBuf>>();
        let (result_tx, result_rx) = crossbeam::channel::unbounded();

        {
            let runtime = runtime.clone();
            let _ = std::thread::spawn(move || {
                runtime.block_on(async move {
                    let mut wg = awaitgroup::WaitGroup::new();
                    while let Some(Some(path)) = path_rx.recv().await {
                        let result_tx = result_tx.clone();
                        let worker = wg.worker();
                        tokio::spawn(async move {
                            match tokio::fs::read_to_string(&path).await {
                                Ok(code) => {
                                    result_tx.send((path, code)).unwrap();
                                }
                                Err(e) => {
                                    warn!("Failed to read file"; "path"=>path.display(), "err"=>e);
                                }
                            }
                            worker.done();
                        });
                    }

                    wg.wait().await;
                });
            });
        }
        Self { path_tx, result_rx }
    }

    pub fn result(&self) -> &CrossbeamRx<(PathBuf, String)> {
        &self.result_rx
    }
    pub fn push(&self, path: PathBuf) {
        self.path_tx.send(Some(path)).unwrap();
    }
    pub fn push_dir(&self, dir: &Path) {
        for entry in jwalk::WalkDir::new(dir) {
            // jwalk::WalkDir::new(dir)
            // .into_iter()
            // .par_bridge()
            // .for_each(|entry|
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        match ext.to_str() {
                            Some(ext) => match ext {
                                "py" => {
                                    self.push(path);
                                }
                                _ => {}
                            },
                            None => {
                                warn!("non utf-8 file extension"; "path" => path.display());
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Error while walking dir"; "err" => format_args!("{}", e));
                }
            }
        }
    }
    pub fn no_more_input(&mut self) {
        self.path_tx.send(None).unwrap();
    }
}
