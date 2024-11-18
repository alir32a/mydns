use std::fs;
use std::path::{Path, PathBuf};
use log::{error};

pub fn get_home_dir() -> Option<PathBuf> {
    if let Some(path) = home::home_dir() {
        return Some(path.join(".mydns"));
    }
    
    None
}

pub fn check_home_dir() {
    if let Some(path) = get_home_dir() {
        if !Path::exists(&path) {
            fs::create_dir(&path).unwrap_or_else(|e| {
                error!("failed to create the home directory, {}", e.to_string())
            });
        }
    } else {
        error!("cannot find the home directory")
    }
}