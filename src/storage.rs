use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::project::Project;

static DIR_CONFIG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TaskMetadata {
    pub status: String,
    pub priority: u8,
}

pub struct Storage;

impl Storage {
    pub fn get_data_dir() -> PathBuf {
        let mut path = dirs::config_dir().unwrap();
        path.push(DIR_CONFIG_NAME);
        path.push("data");
        path
    }

    pub fn check() -> Result<bool, Box<dyn Error>> {
        let data_dir = Storage::get_data_dir();
        fs::create_dir_all(&data_dir)?;
        Ok(false)
    }

    pub fn read() -> Vec<Project> {
        let data_dir = Storage::get_data_dir();
        let mut projects = Vec::new();

        if !data_dir.exists() {
            return projects;
        }

        for entry in fs::read_dir(&data_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let folder_name = path.file_name().unwrap().to_string_lossy().to_string();

                let mut tasks = Vec::new();

                if let Ok(entries) = fs::read_dir(&path) {
                    for task_entry in entries.flatten() {
                        let task_path = task_entry.path();
                        if task_path.is_file() && task_path.extension().map_or(false, |e| e == "md")
                        {
                            if let Some(task) = Storage::read_task(&task_path) {
                                tasks.push(task);
                            }
                        }
                    }
                }

                projects.push(Project {
                    title: folder_name,
                    tasks,
                });
            }
        }

        projects
    }

    fn read_task(path: &Path) -> Option<crate::task::Task> {
        let content = fs::read_to_string(path).ok()?;
        let filename = path.file_stem()?.to_string_lossy().to_string();

        let mut status = "UpNext".to_string();
        let mut priority = 0u8;

        if let Some(frontmatter) = content
            .strip_prefix("---")
            .and_then(|c| c.split_once("---"))
        {
            if let Ok(metadata) = serde_yaml::from_str::<TaskMetadata>(frontmatter.0.trim()) {
                status = metadata.status;
                priority = metadata.priority;
            }
        }

        Some(crate::task::Task {
            title: filename,
            status,
            priority,
            markdown: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        })
    }

    pub fn write_task(app: &mut crate::App, project_idx: usize, task_idx: usize) {
        let project = &app.projects[project_idx];
        let task = &project.tasks[task_idx];

        let data_dir = Storage::get_data_dir();
        let project_dir = data_dir.join(&project.title);

        if !project_dir.exists() {
            fs::create_dir_all(&project_dir).ok();
        }

        let task_filename = task
            .markdown
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format!("{}.md", task.title.replace(' ', "_")));

        let task_path = project_dir.join(&task_filename);

        let front_matter = format!(
            "---\nstatus: {}\npriority: {}\n---\n\n",
            task.status, task.priority
        );

        let existing_content = fs::read_to_string(&task_path)
            .ok()
            .and_then(|c| {
                c.strip_prefix("---")
                    .and_then(|c| c.split_once("---"))
                    .map(|(_, content)| content.trim_start().to_string())
            })
            .unwrap_or_default();

        let full_content = format!("{}{}", front_matter, existing_content);
        fs::write(&task_path, full_content).ok();
    }

    pub fn create_project(name: &str) {
        let data_dir = Storage::get_data_dir();
        let project_dir = data_dir.join(name);
        fs::create_dir_all(project_dir).ok();
    }

    pub fn rename_project(old_name: &str, new_name: &str) {
        let data_dir = Storage::get_data_dir();
        let old_path = data_dir.join(old_name);
        let new_path = data_dir.join(new_name);
        fs::rename(old_path, new_path).ok();
    }

    pub fn delete_project(name: &str) {
        let data_dir = Storage::get_data_dir();
        let project_dir = data_dir.join(name);
        fs::remove_dir_all(project_dir).ok();
    }

    pub fn create_task(project_name: &str, task_title: &str) -> PathBuf {
        let data_dir = Storage::get_data_dir();
        let project_dir = data_dir.join(project_name);

        if !project_dir.exists() {
            fs::create_dir_all(&project_dir).ok();
        }

        let filename = format!("{}.md", task_title.replace(' ', "_"));
        let task_path = project_dir.join(&filename);

        let content = format!(
            "---\nstatus: UpNext\npriority: 0\n---\n\n# {}\n",
            task_title
        );

        fs::write(&task_path, content).ok();

        task_path
    }

    pub fn delete_task(project_name: &str, task_filename: &str) {
        let data_dir = Storage::get_data_dir();
        let task_path = data_dir.join(project_name).join(task_filename);
        fs::remove_file(task_path).ok();
    }

    pub fn get_markdown_path(app: &mut crate::App) -> PathBuf {
        let project = &mut app.projects[app.selected_project_index.selected().unwrap()];
        let task = &project.tasks[app.selected_task_index.selected().unwrap()];

        let data_dir = Storage::get_data_dir();
        let project_dir = data_dir.join(&project.title);

        if !project_dir.exists() {
            fs::create_dir_all(&project_dir).ok();
        }

        let filename = task
            .markdown
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format!("{}.md", task.title.replace(' ', "_")));

        project_dir.join(filename)
    }

    pub fn save_markdown_path(app: &mut crate::App, path: &Path) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();

        if let Some(project) = app.projects.get_mut(project_idx) {
            if let Some(task) = project.tasks.get_mut(task_idx) {
                task.markdown = Some(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }

        Storage::write_task(app, project_idx, task_idx);
    }
}
