use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::env::temp_dir;
use std::path::PathBuf;

pub(crate) fn random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

pub(crate) fn temp_file() -> PathBuf {
    temp_dir().join(format!("{}.nut.db", random_string(32)))
}
