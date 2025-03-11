use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub start_time: DateTime<Local>,
    pub project_directory: PathBuf,
    pub initial_line_count: i32,
    pub initial_char_count: i64,
    pub final_line_count: Option<i32>,
    pub final_char_count: Option<i64>,
    pub lines_written: Option<i32>,
    pub chars_written: Option<i64>,
    pub files_created: Option<Vec<PathBuf>>,
    pub files_modified: Option<Vec<PathBuf>>,
    pub files_deleted: Option<Vec<PathBuf>>,
    pub active_time_seconds: i64,
    #[serde(skip)]
    pub initial_files: Option<HashSet<PathBuf>>,
    #[serde(skip)]
    pub last_activity: Option<Instant>,
    #[serde(skip)]
    pub initial_file_hashes: Option<HashMap<PathBuf, String>>,
}

impl Session {
    pub fn new(project_directory: &str) -> Result<Self> {
        let project_path = PathBuf::from(project_directory);
        let initial_files = Self::scan_files(&project_path)?;
        let initial_file_hashes = Self::compute_file_hashes(&initial_files)?;
        let (initial_lines, initial_chars) = Self::count_all_content(&project_path)?;

        Ok(Session {
            start_time: Local::now(),
            project_directory: project_path,
            initial_line_count: initial_lines,
            initial_char_count: initial_chars,
            final_line_count: None,
            final_char_count: None,
            lines_written: None,
            chars_written: None,
            files_created: None,
            files_modified: None,
            files_deleted: None,
            active_time_seconds: 0,
            initial_files: Some(initial_files),
            initial_file_hashes: Some(initial_file_hashes),
            last_activity: Some(Instant::now()),
        })
    }

    pub fn save(&self) -> Result<()> {
        let session_file = PathBuf::from(".ego_session.json");
        let session_json = serde_json::to_string(self)?;
        fs::write(session_file, session_json)?;
        Ok(())
    }

    pub fn load() -> Result<Option<Self>> {
        let session_file = PathBuf::from(".ego_session.json");
        if session_file.exists() {
            let session_json = fs::read_to_string(session_file)?;
            let mut session: Session = serde_json::from_str(&session_json)?;

            let current_files = Self::scan_files(&session.project_directory)?;
            session.initial_files = Some(current_files.clone());
            session.initial_file_hashes = Some(Self::compute_file_hashes(&current_files)?);
            session.last_activity = Some(Instant::now());

            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub fn end(&mut self) -> Result<()> {
        let (final_line_count, final_char_count) =
            Self::count_all_content(&self.project_directory)?;
        self.final_line_count = Some(final_line_count);
        self.final_char_count = Some(final_char_count);

        self.lines_written = Some(final_line_count - self.initial_line_count);
        self.chars_written = Some(final_char_count - self.initial_char_count);

        let current_files = Self::scan_files(&self.project_directory)?;
        let current_file_hashes = Self::compute_file_hashes(&current_files)?;

        if let (Some(initial_files), Some(initial_hashes)) =
            (&self.initial_files, &self.initial_file_hashes)
        {
            let created: Vec<PathBuf> = current_files.difference(initial_files).cloned().collect();
            let deleted: Vec<PathBuf> = initial_files.difference(&current_files).cloned().collect();

            let common_files: HashSet<PathBuf> = initial_files
                .intersection(&current_files)
                .cloned()
                .collect();

            let modified: Vec<PathBuf> = common_files
                .into_iter()
                .filter(|path| {
                    if let (Some(initial_hash), Some(current_hash)) =
                        (initial_hashes.get(path), current_file_hashes.get(path))
                    {
                        initial_hash != current_hash
                    } else {
                        false
                    }
                })
                .collect();

            self.files_created = Some(created);
            self.files_deleted = Some(deleted);
            self.files_modified = Some(modified);
        }

        fs::remove_file(".ego_session.json")?;
        Ok(())
    }

    pub fn record_activity(&mut self) {
        if let Some(last) = self.last_activity {
            let elapsed = last.elapsed();
            // Only count time if it's less than 5 minutes since last activity
            // (to exclude long breaks)
            if elapsed.as_secs() < 300 {
                self.active_time_seconds += elapsed.as_secs() as i64;
            }
        }
        self.last_activity = Some(Instant::now());
    }

    fn compute_file_hashes(files: &HashSet<PathBuf>) -> Result<HashMap<PathBuf, String>> {
        let mut file_hashes = HashMap::new();

        for file_path in files {
            if let Ok(content) = fs::read_to_string(file_path) {
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                let hash = format!("{:x}", hasher.finish());
                file_hashes.insert(file_path.clone(), hash);
            }
        }

        Ok(file_hashes)
    }

    fn scan_files(dir: &Path) -> Result<HashSet<PathBuf>> {
        let mut file_set = HashSet::new();

        fn visit_dirs(dir: &Path, files: &mut HashSet<PathBuf>) -> io::Result<()> {
            if dir.file_name().map_or(false, |name| {
                let name_str = name.to_string_lossy();
                name_str.starts_with(".")
            }) {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    visit_dirs(&path, files)?;
                } else if path.is_file() {
                    let extension = path.extension().and_then(|e| e.to_str());
                    if let Some(ext) = extension {
                        if [
                            "rs", "txt", "md", "py", "js", "html", "css", "c", "cpp", "h", "hpp",
                            "java", "json", "yaml", "yml", "toml",
                        ]
                        .contains(&ext.to_lowercase().as_str())
                        {
                            files.insert(path.clone());
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dirs(dir, &mut file_set)?;
        Ok(file_set)
    }

    fn count_all_content(dir: &Path) -> Result<(i32, i64)> {
        let mut total_lines: i32 = 0;
        let mut total_chars: i64 = 0;

        fn visit_dirs(dir: &Path, lines: &mut i32, chars: &mut i64) -> io::Result<()> {
            if dir.file_name().map_or(false, |name| {
                let name_str = name.to_string_lossy();
                name_str.starts_with(".")
            }) {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    visit_dirs(&path, lines, chars)?;
                } else if path.is_file() {
                    let extension = path.extension().and_then(|e| e.to_str());
                    if let Some(ext) = extension {
                        if [
                            "rs", "txt", "md", "py", "js", "html", "css", "c", "cpp", "h", "hpp",
                            "java", "json", "yaml", "yml", "toml",
                        ]
                        .contains(&ext.to_lowercase().as_str())
                        {
                            if let Ok(content) = fs::read_to_string(&path) {
                                *lines += content.lines().count() as i32;
                                *chars += content.chars().count() as i64;
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dirs(dir, &mut total_lines, &mut total_chars)?;
        Ok((total_lines, total_chars))
    }
}
