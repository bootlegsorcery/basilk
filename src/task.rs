use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::ListItem,
};
use serde::{Deserialize, Serialize};

use crate::{storage::Storage, util::Util, App};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Task {
    pub title: String,
    pub status: String,
    pub priority: u8,
    pub markdown: Option<String>,
}

pub const TASK_STATUSES: [&str; 3] = ["UpNext", "OnGoing", "Done"];
const TASK_STATUSES_SORT_ORDER: [&str; 3] = ["OnGoing", "UpNext", "Done"];
pub const TASK_PRIORITIES: [u8; 4] = [1, 2, 3, 0];

impl Task {
    pub fn load_statues_items(app: &App, items: &mut Vec<ListItem>) {
        items.clear();
        for status in &app.config.statuses {
            let span = Span::styled(status.label.clone(), Style::new().fg(status.to_color()));
            items.push(ListItem::from(span));
        }
    }

    pub fn load_priority_items(items: &mut Vec<ListItem>) {
        items.clear();
        for priority_value in TASK_PRIORITIES {
            let span = Span::styled(
                Util::get_priority_indicator(priority_value),
                Style::new().fg(Color::Red),
            );
            items.push(ListItem::from(span));
        }
    }

    pub fn load_items(app: &mut App, items: &mut Vec<ListItem>) {
        let config = &app.config;
        let tasks = &mut app.projects[app.selected_project_index.selected().unwrap()].tasks;

        // Get the currently selected task title before reordering
        let last_task_title_selected = tasks
            .clone()
            .get(app.selected_task_index.selected().unwrap_or(0))
            .map(|t| t.title.clone())
            .unwrap_or_default();

        // Sort tasks by status order in config, then by priority (high to low)
        // Priority 1 = !!! (highest), 2 = !! (medium), 3 = ! (lowest), 0 = none
        tasks.sort_by(|a, b| {
            let status_a_idx = config
                .statuses
                .iter()
                .position(|s| s.label == a.status)
                .unwrap_or(usize::MAX);
            let status_b_idx = config
                .statuses
                .iter()
                .position(|s| s.label == b.status)
                .unwrap_or(usize::MAX);

            match status_a_idx.cmp(&status_b_idx) {
                // Sort priority: 1 (!!!), 2 (!!), 3 (!), then 0 (none) at the end
                std::cmp::Ordering::Equal => {
                    match (a.priority, b.priority) {
                        (0, 0) => std::cmp::Ordering::Equal,
                        (0, _) => std::cmp::Ordering::Greater, // 0 goes last
                        (_, 0) => std::cmp::Ordering::Less,    // 0 goes last
                        _ => a.priority.cmp(&b.priority),      // 1, 2, 3 in order
                    }
                }
                other => other,
            }
        });

        let new_index = tasks
            .iter()
            .position(|t| t.title == last_task_title_selected)
            .unwrap_or(0);

        items.clear();

        for task in tasks.iter() {
            let status_config = config.statuses.iter().find(|s| s.label == task.status);
            let is_terminal = status_config.map(|s| s.terminal).unwrap_or(false);
            let status_color = status_config.map(|s| s.to_color()).unwrap_or(Color::Gray);
            let modifier = if is_terminal {
                Modifier::CROSSED_OUT
            } else {
                Modifier::empty()
            };

            let mut repr = vec![
                Span::styled(
                    format!("[{}] ", task.status),
                    Style::default().fg(status_color).add_modifier(modifier),
                ),
                Span::styled(task.title.clone(), Style::default().add_modifier(modifier)),
            ];

            if task.priority != 0 {
                let priority_repr = vec![Span::styled(
                    format!("[{}] ", Util::get_priority_indicator(task.priority)),
                    Style::new().fg(Color::Red),
                )];
                repr = [priority_repr, repr].concat();
            }

            items.push(ListItem::from(Line::from(repr)));
        }

        app.selected_task_index.select(Some(new_index));
    }

    pub fn reload(app: &mut App, items: &mut Vec<ListItem>) {
        app.projects = Storage::read();
        Task::load_items(app, items);
    }

    pub fn _get_all(app: &App) -> &Vec<Task> {
        &app.projects[app.selected_project_index.selected().unwrap()].tasks
    }

    pub fn get_current(app: &mut App) -> &Task {
        &app.projects[app.selected_project_index.selected().unwrap()].tasks
            [app.selected_task_index.selected().unwrap()]
    }

    pub fn create(app: &mut App, items: &mut Vec<ListItem>, value: &str) {
        use crate::config::Config;
        if value.is_empty() {
            return;
        }
        let project = &app.projects[app.selected_project_index.selected().unwrap()];
        let default_status = Config::get_origin_status(&app.config);
        let task_path = Storage::create_task(&project.title, value, &default_status);
        let new_task = Task {
            title: value.to_string(),
            status: default_status,
            priority: 0,
            markdown: Some(task_path.file_name().unwrap().to_string_lossy().to_string()),
        };
        let mut internal_projects = app.projects.clone();
        internal_projects[app.selected_project_index.selected().unwrap()]
            .tasks
            .push(new_task);
        Task::reload(app, items);
    }

    pub fn rename(app: &mut App, items: &mut Vec<ListItem>, value: &str) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();
        let mut internal_projects = app.projects.clone();
        let old_title = internal_projects[project_idx].tasks[task_idx].title.clone();
        internal_projects[project_idx].tasks[task_idx].title = value.to_string();
        let project = &internal_projects[project_idx];
        let old_filename = format!("{}.md", old_title.replace(' ', "_"));
        let new_filename = format!("{}.md", value.replace(' ', "_"));
        let data_dir = Storage::get_data_dir();
        let old_path = data_dir.join(&project.title).join(&old_filename);
        let new_path = data_dir.join(&project.title).join(&new_filename);
        std::fs::rename(&old_path, &new_path).ok();
        if let Some(task) = internal_projects[project_idx].tasks.get_mut(task_idx) {
            task.markdown = Some(new_filename);
        }
        Storage::write_task(app, project_idx, task_idx);
        Task::reload(app, items);
    }

    pub fn change_status(app: &mut App, items: &mut Vec<ListItem>, value: &str) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();
        app.projects[project_idx].tasks[task_idx].status = value.to_string();
        Storage::write_task(app, project_idx, task_idx);
        Task::reload(app, items);
    }

    pub fn change_priority(app: &mut App, items: &mut Vec<ListItem>, value: u8) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();
        app.projects[project_idx].tasks[task_idx].priority = value;
        Storage::write_task(app, project_idx, task_idx);
        Task::reload(app, items);
    }

    pub fn delete(app: &mut App, items: &mut Vec<ListItem>) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();
        let project_name = app.projects[project_idx].title.clone();
        let task_filename = app.projects[project_idx].tasks[task_idx]
            .markdown
            .clone()
            .unwrap_or_else(|| {
                format!(
                    "{}.md",
                    app.projects[project_idx].tasks[task_idx]
                        .title
                        .replace(' ', "_")
                )
            });
        Storage::delete_task(&project_name, &task_filename);
        let mut internal_projects = app.projects.clone();
        internal_projects[project_idx].tasks.remove(task_idx);
        Task::reload(app, items);
    }

    pub fn get_markdown_path(app: &mut App) -> std::path::PathBuf {
        Storage::get_markdown_path(app)
    }
}
