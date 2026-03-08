use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Clear, HighlightSpacing, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_input::Input;

use crate::{project::Project, task::Task, ui::Ui, util::Util, App, ViewMode};

pub struct View {}

impl View {
    pub fn show_new_item_modal(f: &mut Frame, area: Rect, input: &Input) {
        Ui::create_input_modal("New", f, area, input)
    }

    pub fn show_migration_info_modal(f: &mut Frame, area: Rect) {
        let widget = Paragraph::new(Text::from(vec![
            Line::raw("New migrations were applied!"),
            Line::raw("Check the changelog"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::bordered());

        Ui::create_modal(f, 30, 4, area, widget)
    }

    pub fn show_rename_item_modal(f: &mut Frame, area: Rect, input: &Input) {
        Ui::create_input_modal("Rename", f, area, input)
    }

    pub fn show_delete_item_modal(app: &mut App, f: &mut Frame, area: Rect) {
        let title = match app.view_mode {
            ViewMode::DeleteTask => &Task::get_current(app).title,
            ViewMode::DeleteProject => &Project::get_current(app).title,
            _ => "",
        };

        Ui::create_question_modal(
            "Are you sure to delete?",
            format!("\"{}\"", title).as_str(),
            "Delete",
            f,
            area,
        )
    }

    pub fn show_select_task_status_modal(
        app: &mut App,
        status_items: &Vec<ListItem>,
        f: &mut Frame,
        area: Rect,
    ) {
        let area = Ui::create_rect_area(10, 5, area);

        let task_status_list_widget = List::new(status_items.clone())
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Block::bordered().title("Status"));

        f.render_widget(Clear, area);
        f.render_stateful_widget(task_status_list_widget, area, app.use_state())
    }

    pub fn show_select_task_priority_modal(
        app: &mut App,
        priority_items: &Vec<ListItem>,
        f: &mut Frame,
        area: Rect,
    ) {
        let area = Ui::create_rect_area(10, 6, area);

        let task_status_list_widget = List::new(priority_items.clone())
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Block::bordered().title("Priority"));

        f.render_widget(Clear, area);
        f.render_stateful_widget(task_status_list_widget, area, app.use_state())
    }

    pub fn show_items(app: &mut App, items: &Vec<ListItem>, f: &mut Frame, area: Rect) {
        let block: Block = match app.view_mode {
            ViewMode::ViewProjects
            | ViewMode::AddProject
            | ViewMode::RenameProject
            | ViewMode::DeleteProject => Block::bordered(),
            _ => Block::bordered().title(Util::get_spaced_title(&Project::get_current(app).title)),
        };

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items.clone())
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(block);

        // In ViewTasks mode, check task_view_mode to decide rendering
        if matches!(
            app.view_mode,
            ViewMode::ViewTasks | ViewMode::AddTask | ViewMode::RenameTask | ViewMode::DeleteTask
        ) {
            if app.task_view_mode == crate::TaskViewMode::Kanban {
                View::render_task_cards(app, f, area);
            } else {
                f.render_stateful_widget(list, area, app.use_state());
            }
        } else {
            if app.view_mode == ViewMode::ChangeStatusTask
                || app.view_mode == ViewMode::ChangePriorityTask
            {
                f.render_widget(list, area)
            } else {
                f.render_stateful_widget(list, area, app.use_state());
            }
        }
    }

    fn render_task_cards(app: &mut App, f: &mut Frame, area: Rect) {
        // Clone necessary data to avoid borrow issues
        let config = app.config.clone();
        let project_idx = app.selected_project_index.selected().unwrap_or(0);
        let selected_task_idx = app.selected_task_index.selected().unwrap_or(0);

        if project_idx >= app.projects.len() {
            return;
        }

        let mut tasks: Vec<Task> = app.projects[project_idx].tasks.clone();

        // Sort tasks by status order in config, then by priority (high to low) - same as load_items
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
                std::cmp::Ordering::Equal => b.priority.cmp(&a.priority), // Higher priority first
                other => other,
            }
        });

        // Get terminal statuses for strike-through styling
        let terminal_statuses: Vec<String> = config
            .statuses
            .iter()
            .filter(|s| s.terminal)
            .map(|s| s.label.clone())
            .collect();

        // Create horizontal layout with columns for each status
        let status_labels: Vec<String> = config.statuses.iter().map(|s| s.label.clone()).collect();
        let num_columns = status_labels.len().max(1);

        // Build constraints for columns (equal width)
        let constraints: Vec<Constraint> = (0..num_columns)
            .map(|_| Constraint::Percentage((100 / num_columns) as u16))
            .collect();

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Keep track of which task index we're rendering globally
        let mut global_task_idx: usize = 0;

        // Render each column
        for (_col_idx, (column_area, status_label)) in
            columns.iter().zip(status_labels.iter()).enumerate()
        {
            // Get tasks for this status - already sorted by priority from the main sort
            let status_tasks: Vec<&Task> =
                tasks.iter().filter(|t| t.status == *status_label).collect();

            // Find status config for color
            let status_config = config.statuses.iter().find(|s| s.label == *status_label);
            let status_color = status_config
                .map(|s| s.to_color())
                .unwrap_or(ratatui::style::Color::Gray);
            let is_terminal = terminal_statuses.contains(status_label);

            // Create column block with status title
            let column_block = Block::bordered()
                .title(format!(" {} ", status_label))
                .border_style(Style::default().fg(status_color));

            let inner_area = column_block.inner(*column_area);
            f.render_widget(column_block, *column_area);

            // Render tasks as cards in this column
            let mut current_y = inner_area.y;
            let card_height = 3u16; // Title + spacing

            for task in status_tasks.iter() {
                // Check if this is the selected task
                let is_currently_selected = selected_task_idx == global_task_idx;

                if current_y + card_height > inner_area.bottom() {
                    global_task_idx += 1;
                    continue; // Skip rendering if out of bounds, but still increment
                }

                let card_area = Rect {
                    x: inner_area.x,
                    y: current_y,
                    width: inner_area.width,
                    height: card_height,
                };

                // Build task content
                let mut content = task.title.clone();
                if task.priority != 0 {
                    content = format!(
                        "[{}] {}",
                        Util::get_priority_indicator(task.priority),
                        content
                    );
                }

                let modifier = if is_terminal {
                    Modifier::CROSSED_OUT
                } else {
                    Modifier::empty()
                };

                let card_style = if is_currently_selected {
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().add_modifier(modifier)
                };

                // Render card with border
                let card_block = Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(if is_currently_selected {
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(status_color)
                    });

                let card_inner = card_block.inner(card_area);
                f.render_widget(card_block, card_area);

                // Render task text
                let task_text = Paragraph::new(Line::from(vec![
                    Span::styled(
                        if is_currently_selected {
                            "> ".to_string()
                        } else {
                            "  ".to_string()
                        },
                        Style::default().fg(status_color),
                    ),
                    Span::styled(content, card_style),
                ]));
                f.render_widget(task_text, card_inner);

                current_y += card_height + 1; // +1 for spacing between cards
                global_task_idx += 1;
            }

            // Fill remaining space
            if current_y < inner_area.bottom() {
                let empty_area = Rect {
                    x: inner_area.x,
                    y: current_y,
                    width: inner_area.width,
                    height: inner_area.bottom() - current_y,
                };
                f.render_widget(Clear, empty_area);
            }
        }
    }

    pub fn show_footer_helper(app: &mut App, f: &mut Frame, area: Rect) {
        let help_string = match app.view_mode {
            ViewMode::ViewProjects => {
                "<Up/Down k/j> next/prev - <Enter/Right/l> go to tasks - <n> new - <r> rename - <d> delete - <q> quit"
            }
            ViewMode::RenameProject => "<Enter> confirm - <Esc> cancel",
            ViewMode::AddProject => "<Enter> confirm - <Esc> cancel",
            ViewMode::DeleteProject => "<y> confirm - <n> cancel",

            ViewMode::ViewTasks => {
                "<Up/Down k/j> next/prev - <Esc/Left/h> go to projects - <Enter> change status - <p> change priority - <n> new - <r> rename - <d> delete - <e> edit notes - <v> toggle view - <q> quit"
            }
            ViewMode::RenameTask => "<Enter> confirm - <Esc> cancel",
            ViewMode::ChangeStatusTask => "<Up/Down k/j> next/prev - <Enter> confirm - <Esc> cancel",
            ViewMode::ChangePriorityTask => "<Up/Down k/j> next/prev - <Enter> confirm - <Esc> cancel",
            ViewMode::AddTask => "<Enter> confirm - <Esc> cancel",
            ViewMode::DeleteTask => "<y> confirm - <n> cancel",
        };

        f.render_widget(
            Paragraph::new(help_string)
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center),
            area,
        );
    }
}
