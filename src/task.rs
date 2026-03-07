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

pub const TASK_STATUS_DONE: &str = "Done";
pub const TASK_STATUS_ON_GOING: &str = "OnGoing";
pub const TASK_STATUS_UP_NEXT: &str = "UpNext";

pub const TASK_STATUSES: [&'static str; 3] =
    [TASK_STATUS_UP_NEXT, TASK_STATUS_ON_GOING, TASK_STATUS_DONE];

const TASK_STATUSES_SORT_ORDER: [&'static str; 3] =
    [TASK_STATUS_ON_GOING, TASK_STATUS_UP_NEXT, TASK_STATUS_DONE];

// Ascending order: 1 highest priority; 2 medium; 3 lowest
pub const TASK_PRIORITIES: [u8; 4] = [1, 2, 3, 0];

impl Task {
    fn get_status_color(status: &String) -> ratatui::prelude::Color {
        match status.as_str() {
            TASK_STATUS_DONE => return Color::LightGreen,
            TASK_STATUS_ON_GOING => return Color::Yellow,
            TASK_STATUS_UP_NEXT => return Color::LightMagenta,
            _ => return Color::Gray,
        }
    }

    pub fn load_statues_items(items: &mut Vec<ListItem>) {
        items.clear();

        for status in TASK_STATUSES {
            let span = Span::styled(
                status,
                Style::new().fg(Task::get_status_color(&status.to_string())),
            );

            items.push(ListItem::from(span))
        }
    }

    pub fn load_priority_items(items: &mut Vec<ListItem>) {
        items.clear();

        for priority_value in TASK_PRIORITIES {
            let span = Span::styled(
                Util::get_priority_indicator(priority_value),
                Style::new().fg(Color::Red),
            );

            items.push(ListItem::from(span))
        }
    }

    pub fn load_items(app: &mut App, items: &mut Vec<ListItem>) {
        let tasks = &mut app.projects[app.selected_project_index.selected().unwrap()].tasks;

        let last_task_title_selected = tasks
            .clone()
            .get(app.selected_task_index.selected().unwrap_or(0))
            .unwrap_or(&Task {
                title: "".to_string(),
                status: "".to_string(),
                priority: 0,
                markdown: None,
            })
            .clone()
            .title;

        // Sort by status
        tasks.sort_by_key(|t| {
            TASK_STATUSES_SORT_ORDER
                .into_iter()
                .position(|o| o == t.status)
        });

        // Sort by priority
        tasks.sort_by_key(|t| TASK_PRIORITIES.into_iter().position(|o| o == t.priority));

        let new_index = tasks
            .into_iter()
            .position(|t| t.title == last_task_title_selected)
            .unwrap_or(0);

        items.clear();

        for task in tasks.iter() {
            let modifier = if task.status == TASK_STATUS_DONE {
                Modifier::CROSSED_OUT
            } else {
                Modifier::empty()
            };

            let mut repr = vec![
                Span::styled(
                    format!("[{}] ", task.status),
                    Style::default()
                        .fg(Task::get_status_color(&task.status))
                        .add_modifier(modifier),
                ),
                Span::styled(task.title.clone(), Style::default().add_modifier(modifier)),
            ];

            if task.priority != 0 {
                let priority_repr = vec![Span::styled(
                    format!("[{}] ", Util::get_priority_indicator(task.priority)),
                    Style::new().fg(Color::Red),
                )];
                repr = [priority_repr, repr].concat()
            }

            let line = Line::from(repr);

            items.push(ListItem::from(line))
        }

        app.selected_task_index.select(Some(new_index))
    }

    pub fn reload(app: &mut App, items: &mut Vec<ListItem>) {
        app.projects = Storage::read();
        Task::load_items(app, items)
    }

    pub fn _get_all(app: &App) -> &Vec<Task> {
        return &app.projects[app.selected_project_index.selected().unwrap()].tasks;
    }

    pub fn get_current(app: &mut App) -> &Task {
        return &app.projects[app.selected_project_index.selected().unwrap()].tasks
            [app.selected_task_index.selected().unwrap()];
    }

    pub fn create(app: &mut App, items: &mut Vec<ListItem>, value: &str) {
        if value.is_empty() {
            return;
        }

        let project = &app.projects[app.selected_project_index.selected().unwrap()];
        let task_path = Storage::create_task(&project.title, value);

        let new_task = Task {
            title: value.to_string(),
            status: TASK_STATUS_UP_NEXT.to_string(),
            priority: 0,
            markdown: Some(task_path.file_name().unwrap().to_string_lossy().to_string()),
        };

        let mut internal_projects = app.projects.clone();
        internal_projects[app.selected_project_index.selected().unwrap()]
            .tasks
            .push(new_task);

        Task::reload(app, items)
    }

    pub fn rename(app: &mut App, items: &mut Vec<ListItem>, value: &str) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();

        let mut internal_projects = app.projects.clone();
        let old_title = internal_projects[project_idx].tasks[task_idx].title.clone();
        internal_projects[project_idx].tasks[task_idx].title = value.to_string();

        let project = &internal_projects[project_idx];
        let task = &project.tasks[task_idx];

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
        Task::reload(app, items)
    }

    pub fn change_status(app: &mut App, items: &mut Vec<ListItem>, value: &str) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();

        let status = value.to_string();

        app.projects[project_idx].tasks[task_idx].status = status.clone();

        if status == TASK_STATUS_DONE {
            app.projects[project_idx].tasks[task_idx].priority = 0
        }

        Storage::write_task(app, project_idx, task_idx);
        Task::reload(app, items)
    }

    pub fn change_priority(app: &mut App, items: &mut Vec<ListItem>, value: u8) {
        let project_idx = app.selected_project_index.selected().unwrap();
        let task_idx = app.selected_task_index.selected().unwrap();

        app.projects[project_idx].tasks[task_idx].priority = value;

        Storage::write_task(app, project_idx, task_idx);
        Task::reload(app, items)
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

        Task::reload(app, items)
    }

    pub fn get_markdown_path(app: &mut App) -> std::path::PathBuf {
        Storage::get_markdown_path(app)
    }
}
