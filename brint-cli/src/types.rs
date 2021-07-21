use std::path::PathBuf;

use semver::Version;

#[derive(Debug)]
pub struct FeatureDecorator {
    pub path: PathBuf,
    pub line: usize,
    pub feature_name: String,
    pub version: Version,
    pub new: String,
    pub old: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}
