use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub start_time: DateTime<Local>,
    pub project_directory: PathBuf,
    pub initial_line_count: i32,
    pub final_line_count: Option<i32>,
    pub lines_written: Option<i32>,
}

impl Session {
    pub fn new(project_directory: &str) -> Result<Self> {
        let project_path = PathBuf::from(project_directory);
        let initial_lines = Self::count_all_lines(&project_path)?;

        Ok(Session {
            start_time: Local::now(),
            project_directory: project_path,
            initial_line_count: initial_lines,
            final_line_count: None,
            lines_written: None,
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
            let session = serde_json::from_str(&session_json)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub fn end(&mut self) -> Result<()> {
        let final_count = Self::count_all_lines(&self.project_directory)?;
        self.final_line_count = Some(final_count);

        self.lines_written = Some(final_count - self.initial_line_count);

        fs::remove_file(".ego_session.json")?;
        Ok(())
    }

    fn count_all_lines(dir: &Path) -> Result<i32> {
        fn visit_dirs(dir: &Path, acc: &mut i32) -> io::Result<()> {
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
                    visit_dirs(&path, acc)?;
                } else if path.is_file() {
                    let extension = path.extension().and_then(|e| e.to_str());
                    if let Some(ext) = extension {
                        if [
                            "rs", "txt", "md", "py", "js", "html", "css", "c", "cpp", "h", "hpp",
                            "java", "json", "yaml", "yml", "toml",
                        ]
                        .contains(&ext.to_lowercase().as_str())
                        {
                            if let Ok(file) = fs::File::open(&path) {
                                let reader = io::BufReader::new(file);
                                *acc += reader.lines().count() as i32;
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        let mut total_lines: i32 = 0;
        visit_dirs(dir, &mut total_lines)?;
        Ok(total_lines)
    }
}
