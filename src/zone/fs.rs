use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

pub(crate) fn read_dir<P: AsRef<Path>>(p: P, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut res = Vec::new();
    
    for entry in fs::read_dir(p)? {
        match entry { 
            Ok(file) => {
                if file.path().is_dir() {
                    if recursive {
                        match read_dir(file.path(), true) {
                            Ok(mut dir) => {
                                res.append(&mut dir);
                            }
                            Err(_e) => {
                                continue;
                            }
                        }
                    }
                    
                    continue;
                }
                
                res.push(file.path());
            },
            Err(_e) => {
               continue 
            }
        }
    }
    
    Ok(res)
}