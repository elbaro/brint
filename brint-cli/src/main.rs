use colored::Colorize;
use dashmap::DashMap;
use rayon::prelude::*;
use semver::Version;
use std::path::Path;

use crate::types::FeatureDecorator;

pub mod file_read_worker;
pub mod python;
pub mod scan;
pub mod types;

/// jwalk => mpsc => FileReadWorker => spmc => par_iter => ast parsing

pub fn timestamp_fn(_io: &mut dyn std::io::Write) -> std::io::Result<()> {
    Ok(())
}

fn main() {
    let _guard = {
        use slog::Drain;

        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::CompactFormat::new(decorator)
            .use_custom_timestamp(timestamp_fn)
            .build()
            .fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = slog::Logger::root(drain, slog::o!());

        // let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
        // let log = slog::Logger::root(slog_term::FullFormat::new(drain).build().fuse(), slog::o!());
        let guard = slog_scope::set_global_logger(log);
        guard
    };

    let mut reader = file_read_worker::FileReadWorker::run_in_background_thread();
    reader.push_dir(Path::new("."));
    reader.no_more_input();

    let num_files = std::sync::atomic::AtomicUsize::new(0);
    let num_bytes = std::sync::atomic::AtomicUsize::new(0);

    let all_features = DashMap::<Version, Vec<FeatureDecorator>>::new();

    reader
        .result()
        .iter()
        .par_bridge()
        .for_each(|(path, code)| {
            let features = python::process_code(&path, &code);
            if let Some(features) = features {
                for f in features {
                    all_features.entry(f.version.clone()).or_default().push(f);
                }
            }

            num_files.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            num_bytes.fetch_add(code.len(), std::sync::atomic::Ordering::Relaxed);
        });
    println!("# of    files: {}", num_files.into_inner());
    println!("# of    bytes: {}", num_bytes.into_inner());
    println!("# of features: {}", all_features.len());
    println!("");

    let mut versions: Vec<Version> = all_features.iter().map(|kv| kv.key().clone()).collect();
    versions.sort_by(|a, b| b.cmp(a));

    for version in versions {
        let features = all_features.get(&version).unwrap();
        println!("[{}]", version.to_string().bold().underline());
        for f in features.iter() {
            println!(
                "    \"{}\" in {}:{}",
                f.feature_name,
                f.path.display(),
                f.line
            );
            if let Some(s) = &f.old {
                println!("        - old: {}", s);
            }
            println!("        - new: {}", f.new);
        }
        println!("");
    }
}
