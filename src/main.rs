use std::{error::Error, fmt::Debug, fs, io::stdout, process::Command};

use cli::Cli;
use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};
use tui_input::{backend::crossterm::EventHandler, Input};

mod cli;
mod config;
mod git;
mod project;
mod storage;
mod task;
mod ui;
mod util;
mod view;

use config::{Config, ConfigToml};
use git::Git;
use project::Project;
use storage::Storage;
use task::{Task, TASK_COSTS, TASK_PRIORITIES, TASK_TIMES};
use view::View;

#[derive(Default, PartialEq, Debug)]
pub enum ViewMode {
    #[default]
    ViewProjects,
    RenameProject,
    AddProject,
    DeleteProject,
    GitCommit,

    ViewTasks,
    RenameTask,
    ChangeStatusTask,
    ChangePriorityTask,
    ChangeCostTask,
    ChangeTimeTask,
    AddTask,
    DeleteTask,
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum TaskViewMode {
    #[default]
    List,
    Kanban,
}

pub struct App {
    // TODO: Better list state mgmt
    selected_project_index: ListState,
    selected_task_index: ListState,
    selected_status_task_index: ListState,
    selected_priority_task_index: ListState,
    selected_cost_task_index: ListState,
    selected_time_task_index: ListState,
    view_mode: ViewMode,
    task_view_mode: TaskViewMode,
    projects: Vec<Project>,
    config: ConfigToml,
    is_git_repo: bool,
    git_has_changes: bool,
    git_commit_message: String,
    show_help: bool,
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    Cli::read();

    // setup terminal
    let terminal = init_terminal()?;

    // Check the storage
    let _were_applied_migrations = Storage::check()?;

    // create app and run it
    App::setup().run(terminal)?;

    restore_terminal()?;

    Ok(())
}

impl App {
    fn setup() -> Self {
        let is_git_repo = Git::is_git_repo();
        let git_has_changes = if is_git_repo {
            Git::has_changes()
        } else {
            false
        };

        Self {
            selected_project_index: ListState::default().with_selected(Some(0)),
            selected_task_index: ListState::default().with_selected(Some(0)),
            selected_status_task_index: ListState::default().with_selected(Some(0)),
            selected_priority_task_index: ListState::default().with_selected(Some(0)),
            selected_cost_task_index: ListState::default().with_selected(Some(0)),
            selected_time_task_index: ListState::default().with_selected(Some(0)),
            view_mode: ViewMode::default(),
            task_view_mode: TaskViewMode::default(),
            projects: Storage::read(),
            config: Config::read(),
            is_git_repo,
            git_has_changes,
            git_commit_message: String::new(),
            show_help: false,
        }
    }

    fn run(
        &mut self,
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut input = Input::default();

        let mut items: Vec<ListItem> = vec![];
        Project::load_items(self, &mut items);

        let mut status_items: Vec<ListItem> = vec![];
        Task::load_statues_items(self, &mut status_items);

        let mut priority_items: Vec<ListItem> = vec![];
        Task::load_priority_items(&mut priority_items);

        let mut cost_items: Vec<ListItem> = vec![];
        Task::load_cost_items(&mut cost_items);

        let mut time_items: Vec<ListItem> = vec![];
        Task::load_time_items(&mut time_items);

        loop {
            terminal.draw(|f| {
                self.render(
                    f,
                    f.size(),
                    &input,
                    &items,
                    &status_items,
                    &priority_items,
                    &cost_items,
                    &time_items,
                )
            })?;

            if let Event::Key(key) = event::read()? {
                // Capture only the "Press" event to prevent double input on Windows
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;

                    // Handle help modal first - it should be modal above any view
                    if self.show_help {
                        match key.code {
                            Esc | Char('q') | Char('?') => {
                                self.show_help = false;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match self.view_mode {
                        ViewMode::ViewProjects => match key.code {
                            Enter | Right | Char('l') => {
                                if items.is_empty() {
                                    continue;
                                }

                                Task::load_items(self, &mut items);
                                self.selected_task_index.select(Some(0));

                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            Char('r') => {
                                if items.is_empty() {
                                    continue;
                                }

                                input = input
                                    .clone()
                                    .with_value(Project::get_current(self).title.clone());

                                App::change_view(self, ViewMode::RenameProject);
                            }
                            Char('n') => {
                                input.reset();

                                App::change_view(self, ViewMode::AddProject);
                            }
                            Char('d') => {
                                if items.is_empty() {
                                    continue;
                                }

                                App::change_view(self, ViewMode::DeleteProject);
                            }
                            Char('c') => {
                                if self.is_git_repo && self.git_has_changes {
                                    input.reset();
                                    App::change_view(self, ViewMode::GitCommit);
                                }
                            }
                            Down | Tab | Char('j') => {
                                self.next(&items);
                            }
                            Up | BackTab | Char('k') => {
                                self.previous(&items);
                            }
                            Char('q') => {
                                return Ok(());
                            }
                            Char('?') => {
                                self.show_help = true;
                            }
                            _ => {}
                        },
                        ViewMode::RenameProject => match key.code {
                            Enter => {
                                Project::rename(self, &mut items, input.value());
                                input.reset();

                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            Esc => {
                                input.reset();

                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            _ => {
                                input.handle_event(&Event::Key(key));
                            }
                        },
                        ViewMode::AddProject => match key.code {
                            Esc => {
                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            Enter => {
                                Project::create(self, &mut items, input.value());
                                self.selected_project_index
                                    .select(Some(self.projects.len()));

                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            _ => {
                                input.handle_event(&Event::Key(key));
                            }
                        },
                        ViewMode::DeleteProject => match key.code {
                            Char('y') => {
                                Project::delete(self, &mut items);
                                self.selected_project_index.select_previous();

                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            Char('n') => {
                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            _ => {}
                        },
                        ViewMode::GitCommit => match key.code {
                            Enter => {
                                if !input.value().is_empty() {
                                    match Git::commit_all(input.value()) {
                                        Ok(()) => {
                                            self.git_has_changes = Git::has_changes();
                                            self.git_commit_message.clear();
                                        }
                                        Err(e) => {
                                            self.git_commit_message = e;
                                        }
                                    }
                                }
                                input.reset();
                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            Esc => {
                                input.reset();
                                self.git_commit_message.clear();
                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            _ => {
                                input.handle_event(&Event::Key(key));
                            }
                        },

                        ViewMode::ViewTasks => match key.code {
                            Esc | Left | Char('h') => {
                                Project::load_items(self, &mut items);

                                App::change_view(self, ViewMode::ViewProjects);
                            }
                            Enter => {
                                if items.is_empty() {
                                    continue;
                                }

                                let current_status = Task::get_current(self).status.clone();
                                let index = self
                                    .config
                                    .statuses
                                    .iter()
                                    .position(|s| s.label == current_status)
                                    .unwrap_or(0);

                                self.selected_status_task_index.select(Some(index));

                                App::change_view(self, ViewMode::ChangeStatusTask);
                            }
                            Char('p') => {
                                if items.is_empty() {
                                    continue;
                                }

                                let index = TASK_PRIORITIES
                                    .into_iter()
                                    .position(|t| t == Task::get_current(self).priority)
                                    .unwrap();

                                self.selected_priority_task_index.select(Some(index));

                                App::change_view(self, ViewMode::ChangePriorityTask);
                            }
                            Char('c') => {
                                if items.is_empty() {
                                    continue;
                                }

                                let index = TASK_COSTS
                                    .into_iter()
                                    .position(|t| t == Task::get_current(self).cost)
                                    .unwrap();

                                self.selected_cost_task_index.select(Some(index));

                                App::change_view(self, ViewMode::ChangeCostTask);
                            }
                            Char('t') => {
                                if items.is_empty() {
                                    continue;
                                }

                                let index = TASK_TIMES
                                    .into_iter()
                                    .position(|t| t == Task::get_current(self).time)
                                    .unwrap();

                                self.selected_time_task_index.select(Some(index));

                                App::change_view(self, ViewMode::ChangeTimeTask);
                            }
                            Char('r') => {
                                if items.is_empty() {
                                    continue;
                                }

                                input = input
                                    .clone()
                                    .with_value(Task::get_current(self).title.clone());

                                App::change_view(self, ViewMode::RenameTask);
                            }
                            Char('n') => {
                                input.reset();

                                App::change_view(self, ViewMode::AddTask);
                            }
                            Char('d') => {
                                if items.is_empty() {
                                    continue;
                                }

                                App::change_view(self, ViewMode::DeleteTask);
                            }
                            Char('e') => {
                                if items.is_empty() {
                                    continue;
                                }

                                let input_clone = input.clone();
                                drop(terminal);
                                drop(input);

                                restore_terminal().unwrap();

                                let content = Storage::get_task_content(self);
                                let temp_path = Storage::get_temp_edit_path();
                                fs::write(&temp_path, &content).ok();

                                Command::new("nvim")
                                    .arg(temp_path.as_os_str())
                                    .status()
                                    .expect("Failed to open nvim");

                                let edited_content =
                                    fs::read_to_string(&temp_path).ok().unwrap_or_default();
                                fs::remove_file(&temp_path).ok();

                                enable_raw_mode().unwrap();
                                stdout().execute(EnterAlternateScreen).unwrap();

                                Storage::save_task_content(self, &edited_content);

                                terminal = init_terminal().unwrap();

                                Task::reload(self, &mut items);
                                input = input_clone;
                            }
                            Char('v') => {
                                // Toggle between List and Kanban view
                                self.task_view_mode = match self.task_view_mode {
                                    TaskViewMode::List => TaskViewMode::Kanban,
                                    TaskViewMode::Kanban => TaskViewMode::List,
                                };
                            }
                            Char('i') => {
                                // Toggle cost/time indicators visibility
                                self.config.ui.show_cost_time = !self.config.ui.show_cost_time;
                                // Reload items to update the display immediately without changing order
                                Task::load_items(self, &mut items);
                            }
                            Down | Tab | Char('j') => {
                                self.next(&items);
                            }
                            Up | BackTab | Char('k') => {
                                self.previous(&items);
                            }
                            Char('q') => {
                                return Ok(());
                            }
                            Char('?') => {
                                self.show_help = true;
                            }
                            _ => {}
                        },
                        ViewMode::RenameTask => match key.code {
                            Enter => {
                                Task::rename(self, &mut items, input.value());
                                input.reset();

                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            Esc => {
                                input.reset();

                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            _ => {
                                input.handle_event(&Event::Key(key));
                            }
                        },
                        ViewMode::ChangeStatusTask => match key.code {
                            Enter => {
                                let status_label = self.config.statuses
                                    [self.selected_status_task_index.selected().unwrap()]
                                .label
                                .clone();
                                Task::change_status(self, &mut items, &status_label);

                                self.selected_status_task_index.select(Some(0));
                                App::change_view(self, ViewMode::ViewTasks);
                            }

                            Down | BackTab | Char('j') => {
                                self.next(&status_items);
                            }
                            Up | Tab | Char('k') => {
                                self.previous(&status_items);
                            }
                            Esc => {
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            _ => {}
                        },
                        ViewMode::ChangePriorityTask => match key.code {
                            Enter => {
                                Task::change_priority(
                                    self,
                                    &mut items,
                                    TASK_PRIORITIES
                                        [self.selected_priority_task_index.selected().unwrap()],
                                );

                                self.selected_priority_task_index.select(Some(0));
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            Down | BackTab | Char('j') => {
                                self.next(&priority_items);
                            }
                            Up | Tab | Char('k') => {
                                self.previous(&priority_items);
                            }
                            Esc => {
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            _ => {}
                        },
                        ViewMode::ChangeCostTask => match key.code {
                            Enter => {
                                Task::change_cost(
                                    self,
                                    &mut items,
                                    TASK_COSTS[self.selected_cost_task_index.selected().unwrap()],
                                );

                                self.selected_cost_task_index.select(Some(0));
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            Down | BackTab | Char('j') => {
                                self.next(&priority_items);
                            }
                            Up | Tab | Char('k') => {
                                self.previous(&priority_items);
                            }
                            Esc => {
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            _ => {}
                        },
                        ViewMode::ChangeTimeTask => match key.code {
                            Enter => {
                                Task::change_time(
                                    self,
                                    &mut items,
                                    TASK_TIMES[self.selected_time_task_index.selected().unwrap()],
                                );

                                self.selected_time_task_index.select(Some(0));
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            Down | BackTab | Char('j') => {
                                self.next(&priority_items);
                            }
                            Up | Tab | Char('k') => {
                                self.previous(&priority_items);
                            }
                            Esc => {
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            _ => {}
                        },
                        ViewMode::AddTask => match key.code {
                            Enter => {
                                Task::create(self, &mut items, input.value());

                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            Esc => {
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            _ => {
                                input.handle_event(&Event::Key(key));
                            }
                        },
                        ViewMode::DeleteTask => match key.code {
                            Char('y') => {
                                Task::delete(self, &mut items);
                                self.selected_task_index.select_previous();

                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            Char('n') => {
                                App::change_view(self, ViewMode::ViewTasks);
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    fn render(
        &mut self,
        f: &mut Frame,
        area: Rect,
        input: &Input,
        items: &Vec<ListItem>,
        status_items: &Vec<ListItem>,
        priority_items: &Vec<ListItem>,
        cost_items: &Vec<ListItem>,
        time_items: &Vec<ListItem>,
    ) {
        let layout = Layout::vertical(if self.config.ui.show_help {
            [
                Constraint::Percentage(2),
                Constraint::Percentage(93),
                // Space for the footer helper
                Constraint::Percentage(5),
            ]
        } else {
            [
                Constraint::Percentage(2),
                // Expand the main area
                Constraint::Percentage(98),
                // Remove the footer help area
                Constraint::Percentage(0),
            ]
        });

        let [header_area, main_area, footer_area] = layout.areas(area);

        // Header
        f.render_widget(
            Paragraph::new(format!("::{}::", env!("CARGO_PKG_NAME"))).centered(),
            header_area,
        );

        // Main view
        View::show_items(self, items, f, main_area);

        // Other views
        if self.view_mode == ViewMode::AddTask || self.view_mode == ViewMode::AddProject {
            View::show_new_item_modal(f, area, input)
        }

        if self.view_mode == ViewMode::RenameTask || self.view_mode == ViewMode::RenameProject {
            View::show_rename_item_modal(f, area, input)
        }

        if self.view_mode == ViewMode::DeleteTask || self.view_mode == ViewMode::DeleteProject {
            View::show_delete_item_modal(self, f, area)
        }

        if self.view_mode == ViewMode::ChangeStatusTask {
            View::show_select_task_status_modal(self, status_items, f, area)
        }

        if self.view_mode == ViewMode::ChangePriorityTask {
            View::show_select_task_priority_modal(self, priority_items, f, area)
        }

        if self.view_mode == ViewMode::ChangeCostTask {
            View::show_select_task_cost_modal(self, cost_items, f, area)
        }

        if self.view_mode == ViewMode::ChangeTimeTask {
            View::show_select_task_time_modal(self, time_items, f, area)
        }

        if self.view_mode == ViewMode::GitCommit {
            let modified_files = if self.is_git_repo && self.git_has_changes {
                Git::get_modified_files()
            } else {
                Vec::new()
            };
            View::show_git_commit_modal(self, f, area, input, &modified_files)
        }

        if self.show_help {
            View::show_help_modal(self, f, area)
        }

        // Always show minimal footer
        View::show_footer_helper(self, f, footer_area)
    }

    fn next(&mut self, items: &Vec<ListItem>) -> () {
        let i = match self.use_state().selected() {
            Some(i) => {
                if i >= items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.use_state().select(Some(i))
    }

    fn previous(&mut self, items: &Vec<ListItem>) {
        let i = match self.use_state().selected() {
            Some(i) => {
                if i == 0 {
                    items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.use_state().select(Some(i))
    }

    fn use_state(&mut self) -> &mut ListState {
        match self.view_mode {
            ViewMode::ViewProjects => return &mut self.selected_project_index,
            ViewMode::RenameProject => return &mut self.selected_project_index,
            ViewMode::AddProject => return &mut self.selected_project_index,
            ViewMode::DeleteProject => return &mut self.selected_project_index,
            ViewMode::GitCommit => return &mut self.selected_project_index,

            ViewMode::ViewTasks => return &mut self.selected_task_index,
            ViewMode::RenameTask => return &mut self.selected_task_index,
            ViewMode::ChangeStatusTask => return &mut self.selected_status_task_index,
            ViewMode::ChangePriorityTask => return &mut self.selected_priority_task_index,
            ViewMode::ChangeCostTask => return &mut self.selected_cost_task_index,
            ViewMode::ChangeTimeTask => return &mut self.selected_time_task_index,
            ViewMode::AddTask => return &mut self.selected_task_index,
            ViewMode::DeleteTask => return &mut self.selected_task_index,
        };
    }

    fn change_view(&mut self, mode: ViewMode) {
        self.view_mode = mode
    }
}
