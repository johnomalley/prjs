#[macro_use]
extern crate lazy_static;
extern crate chrono;

use std::collections::HashSet;
use std::fs::read_dir;
use std::env;
use std::fs::ReadDir;
use std::fs::DirEntry;
use std::time::UNIX_EPOCH;
use std::time::SystemTime;
use std::io::Result;
use std::cmp;
use chrono::{DateTime, Utc, Local};
use chrono::naive::NaiveDateTime;

const MAX_DEPTH: u32 = 4;
const MAX_PROJECTS: usize = 20;

struct Project {
    path: String,
    timestamp: i64
}

lazy_static! {
    static ref EXCLUDED_DIRS: HashSet<&'static str> = {
        let mut dirs = HashSet::new();
        dirs.insert("node_modules");
        dirs.insert("target");
        dirs
    };
}

fn should_exclude(dir: &DirEntry) -> bool {
    EXCLUDED_DIRS.contains(dir.file_name().to_str().unwrap_or(""))
}

fn to_timestamp(modified: SystemTime) -> i64 {
    modified.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64    
}

fn max_timestamp(path: &str) -> Result<i64> {
    let entries = read_dir(path)?;
    let mut timestamp: i64 = 0;
    for entry_result in entries {
        let entry = entry_result?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            if !should_exclude(&entry) {
                for child_path in entry.path().to_str() {
                    timestamp = cmp::max(timestamp, max_timestamp(&child_path)?);
                }
            }
        } else {
            timestamp = cmp::max(timestamp, to_timestamp(metadata.modified()?))
        }
    }
    Ok(timestamp)
}

fn create_project(path: &str) -> Result<Project> {
    Ok(
        Project {
            path: String::from(path),
            timestamp: max_timestamp(path)?
        }
    )
}

fn to_project(entry: &DirEntry) -> Result<Option<Project>> {
    match entry.path().to_str() {
        Some(s) => Ok(Some(create_project(&s)?)),
        None => Ok(None)
    }
}

fn is_project_dir(entry: &DirEntry) -> Result<bool> {
    for path_string in entry.path().to_str() {
        for child_entry in read_dir(path_string)? {
            if child_entry?.file_name().to_str() == Some(".git") {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn add_projects(projects: &mut Vec<Project>, entries: ReadDir, depth: u32) -> Result<()> {
    for entry_result in entries {
        let entry = entry_result?;
        if entry.metadata()?.is_dir() {
            if is_project_dir(&entry)? {
                for project in to_project(&entry)? {
                    projects.push(project);
                }
            } else if depth < MAX_DEPTH {
                let path = entry.path();
                for path_string in path.to_str() {
                    add_projects(projects, read_dir(path_string)?, depth + 1)?;
                }
            }
        }
    }
    Ok(())
}

fn all_projects(projects_dir: &str) -> Result<Vec<Project>> {
    let mut projects: Vec<Project> = Vec::new();
    add_projects(&mut projects, read_dir(projects_dir)?, 0)?;
    projects.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    projects.truncate(MAX_PROJECTS);
    Ok(projects)
}

fn format_time(modified: i64) -> String {
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(modified, 0), Utc).with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
}

fn main() {
    let home = env::var("HOME").expect("HOME environment variable is required");
    let projects_dir = format!("{}/projects", home);
    let projects = all_projects(&projects_dir).expect("unable to get all projects");
    for project in projects {
        println!("{} {}", format_time(project.timestamp), &project.path[(projects_dir.len() + 1)..]);
    }
}
