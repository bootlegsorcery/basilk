use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Clear, HighlightSpacing, List, ListItem, Paragraph},
    Frame,
};
use tui_input::Input;

use crate::{project::Project, task::Task, ui::Ui, util::Util, App, ViewMode};

pub struct View {}

impl View {
    pub fn show_new_item_modal(f: &mut Frame, area: Rect, input: &Input) {
        Ui::create_input_modal_with_tip("New", "<Enter> confirm  <Esc> cancel", f, area, input)
    }

    pub fn show_rename_item_modal(f: &mut Frame, area: Rect, input: &Input) {
        Ui::create_input_modal_with_tip("Rename", "<Enter> confirm  <Esc> cancel", f, area, input)
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
            Some("<y> confirm  <n> cancel"),
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
        let selected_idx = app.selected_status_task_index.selected().unwrap_or(0);
        let total_items = status_items.len();

        // Cap height at 6 (4 items + 2 borders), or less if fewer items
        let modal_height = (total_items as u16 + 2).min(6);

        // Width based on content - find the longest status label
        let max_label_len = app
            .config
            .statuses
            .iter()
            .map(|s| s.label.len())
            .max()
            .unwrap_or(10);
        // Add space for highlight symbol ("> "), borders, and padding
        let content_width = (max_label_len + 6) as u16;
        let title_width = 10u16; // "X/Y" with indicators
        let max_content_width = content_width.max(title_width);

        let popup_width = (max_content_width + 4).clamp(20, area.width.min(50));
        let percent_x = ((popup_width as f32 / area.width as f32) * 100.0) as u16;

        // Use fixed height and center vertically
        let area = Ui::create_centered_modal_area(percent_x, modal_height, area);

        // Always clear to make modal solid
        f.render_widget(Clear, area);

        // Build title with scroll indicator on left side
        let scroll_indicator = if selected_idx > 0 { "▲ " } else { "  " };
        let more_indicator = if selected_idx < total_items - 1 {
            " ▼"
        } else {
            "  "
        };
        let title = format!(
            "{}{}/{}{}",
            scroll_indicator,
            selected_idx + 1,
            total_items,
            more_indicator
        );

        let task_status_list_widget = List::new(status_items.clone())
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Block::bordered().title(title));

        f.render_stateful_widget(task_status_list_widget, area, app.use_state())
    }

    pub fn show_select_task_priority_modal(
        app: &mut App,
        priority_items: &Vec<ListItem>,
        f: &mut Frame,
        area: Rect,
    ) {
        let selected_idx = app.selected_priority_task_index.selected().unwrap_or(0);
        let total_items = priority_items.len();

        // Cap height at 8 (6 items + 2 borders), or less if fewer items
        let modal_height = (total_items as u16 + 2).min(8);

        // Width based on content - find the longest priority label
        let max_label_len = 15; // Priority labels are short
        let content_width = (max_label_len + 6) as u16; // +6 for highlight symbol, borders, and padding
        let title_width = 10u16; // "X/Y" with indicators
        let max_content_width = content_width.max(title_width);

        let popup_width = (max_content_width + 4).clamp(20, area.width.min(50));
        let percent_x = ((popup_width as f32 / area.width as f32) * 100.0) as u16;

        // Use fixed height and center vertically
        let area = Ui::create_centered_modal_area(percent_x, modal_height, area);

        // Always clear to make modal solid
        f.render_widget(Clear, area);

        // Build title with scroll indicator on left side
        let scroll_indicator = if selected_idx > 0 { "▲ " } else { "  " };
        let more_indicator = if selected_idx < total_items - 1 {
            " ▼"
        } else {
            "  "
        };
        let title = format!(
            "{}{}/{}{}",
            scroll_indicator,
            selected_idx + 1,
            total_items,
            more_indicator
        );

        let task_priority_list_widget = List::new(priority_items.clone())
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Block::bordered().title(title));

        f.render_stateful_widget(task_priority_list_widget, area, app.use_state())
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
        } else if app.view_mode == ViewMode::ChangeStatusTask
            || app.view_mode == ViewMode::ChangePriorityTask
        {
            // Render the appropriate background based on task_view_mode before showing modal
            // Use non-stateful render for background since modal takes focus
            if app.task_view_mode == crate::TaskViewMode::Kanban {
                View::render_task_cards_non_interactive(app, f, area);
            } else {
                f.render_widget(list, area);
            }
        } else {
            f.render_stateful_widget(list, area, app.use_state());
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

                // Build task text spans - priority in red
                let mut task_spans = vec![Span::styled(
                    if is_currently_selected {
                        "> ".to_string()
                    } else {
                        "  ".to_string()
                    },
                    Style::default().fg(status_color),
                )];

                // Add priority indicator in red if present
                if task.priority != 0 {
                    task_spans.push(Span::styled(
                        format!("[{}] ", Util::get_priority_indicator(task.priority)),
                        Style::default()
                            .fg(ratatui::style::Color::Red)
                            .add_modifier(modifier),
                    ));
                }

                task_spans.push(Span::styled(task.title.clone(), card_style));

                let task_text = Paragraph::new(Line::from(task_spans));
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

    fn render_task_cards_non_interactive(app: &mut App, f: &mut Frame, area: Rect) {
        // Clone necessary data to avoid borrow issues
        let config = app.config.clone();
        let project_idx = app.selected_project_index.selected().unwrap_or(0);

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

            // Render tasks as cards in this column - NO SELECTION HIGHLIGHTING
            let mut current_y = inner_area.y;
            let card_height = 3u16; // Title + spacing

            for task in status_tasks.iter() {
                if current_y + card_height > inner_area.bottom() {
                    continue; // Skip rendering if out of bounds
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

                let card_style = Style::default().add_modifier(modifier);

                // Render card with border (no selection highlighting)
                let card_block = Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(Style::default().fg(status_color));

                let card_inner = card_block.inner(card_area);
                f.render_widget(card_block, card_area);

                // Render task text (no selection indicator)
                let task_text = Paragraph::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(content, card_style),
                ]));
                f.render_widget(task_text, card_inner);

                current_y += card_height + 1; // +1 for spacing between cards
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

    pub fn show_git_commit_modal(
        app: &mut App,
        f: &mut Frame,
        area: Rect,
        input: &Input,
        modified_files: &[String],
    ) {
        // Calculate width based on content
        let max_file_len = modified_files.iter().map(|f| f.len()).max().unwrap_or(0);
        let title_len = "Git Commit".len();
        let input_len = input.value().len();
        let max_content_width = max_file_len.max(title_len).max(input_len).max(25);

        // Add padding for borders and content
        let popup_width = ((max_content_width + 8) as u16).clamp(40, area.width.min(90));
        let percent_x = ((popup_width as f32 / area.width as f32) * 100.0) as u16;

        // Height based on files + header + input + tip
        let base_height = 5u16; // header + separator + input + tip + padding
        let file_height = modified_files.len() as u16 + 1; // files + potential "more" line
        let height = (base_height + file_height).min(16).max(8);

        let area = Ui::create_centered_modal_area(percent_x, height, area);

        // Clear to make modal solid
        f.render_widget(Clear, area);

        // Build the modal content
        let mut lines = vec![Line::from("Git Commit"), Line::from("")];

        // Show modified files (limit to available space)
        let max_files = (height as usize).saturating_sub(7);
        for (_i, file) in modified_files.iter().take(max_files).enumerate() {
            let display = if file.len() > area.width as usize - 6 {
                format!("{}...", &file[..(area.width as usize - 9).min(file.len())])
            } else {
                file.clone()
            };
            lines.push(Line::from(format!("  • {}", display)));
        }
        if modified_files.len() > max_files {
            lines.push(Line::from(format!(
                "  ... and {} more",
                modified_files.len() - max_files
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(
            format!("> {}", input.value())
                .chars()
                .take(area.width as usize - 4)
                .collect::<String>(),
        ));

        if !app.git_commit_message.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(format!("Error: {}", app.git_commit_message)));
        }

        let widget = Paragraph::new(Text::from(lines)).block(Block::bordered());

        f.render_widget(widget, area);

        // Render tip at the bottom
        let tip_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 2,
            width: area.width - 2,
            height: 1,
        };
        f.render_widget(
            Paragraph::new("<Enter> commit  <Esc> cancel")
                .alignment(Alignment::Center)
                .style(Style::default().add_modifier(Modifier::DIM)),
            tip_area,
        );
    }

    pub fn show_help_modal(app: &mut App, f: &mut Frame, area: Rect) {
        use crate::ViewMode;

        // Build lines first to calculate final dimensions
        let mut lines = vec![
            Line::from("Help").alignment(Alignment::Center),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  <Up/Down> or <k/j>    - Move up/down"),
            Line::from("  <Left/Right> or <h/l> - Move left/right"),
        ];

        // Show relevant shortcuts based on current view mode
        match app.view_mode {
            ViewMode::ViewProjects
            | ViewMode::AddProject
            | ViewMode::RenameProject
            | ViewMode::DeleteProject => {
                lines.push(Line::from(""));
                lines.push(Line::from("Projects:"));
                lines.push(Line::from("  <Enter>,>,l          - Open project tasks"));
                lines.push(Line::from("  <n>                   - New project"));
                lines.push(Line::from("  <r>                   - Rename project"));
                lines.push(Line::from("  <d>                   - Delete project"));
                if app.is_git_repo && app.git_has_changes {
                    lines.push(Line::from("  <c>                   - Git commit"));
                }
            }
            ViewMode::ViewTasks
            | ViewMode::AddTask
            | ViewMode::RenameTask
            | ViewMode::DeleteTask
            | ViewMode::ChangeStatusTask
            | ViewMode::ChangePriorityTask => {
                lines.push(Line::from(""));
                lines.push(Line::from("Tasks:"));
                lines.push(Line::from("  <Esc>,<h>             - Back to projects"));
                lines.push(Line::from("  <Enter>               - Change task status"));
                lines.push(Line::from("  <p>                   - Change priority"));
                lines.push(Line::from("  <n>                   - New task"));
                lines.push(Line::from("  <r>                   - Rename task"));
                lines.push(Line::from("  <d>                   - Delete task"));
                lines.push(Line::from("  <e>                   - Edit notes"));
                lines.push(Line::from("  <v>                   - Toggle view"));
            }
            ViewMode::GitCommit => {
                lines.push(Line::from(""));
                lines.push(Line::from("Git Commit:"));
                lines.push(Line::from("  <Enter>               - Commit changes"));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from("General:"));
        lines.push(Line::from("  <q>                   - Quit"));
        lines.push(Line::from("  <?>                   - Toggle this help"));

        // Calculate max width based on actual content
        let max_content_width = lines.iter().map(|line| line.width()).max().unwrap_or(30) as u16;

        // Add padding for borders and center alignment
        let popup_width = (max_content_width + 4).clamp(35, area.width.min(80));
        let percent_x = ((popup_width as f32 / area.width as f32) * 100.0) as u16;

        // Calculate height based on actual line count + borders
        let height = lines.len() as u16 + 2; // +2 for borders

        // Create the modal area with final dimensions
        let area = Ui::create_centered_modal_area(percent_x, height, area);
        f.render_widget(Clear, area);

        let widget = Paragraph::new(Text::from(lines)).block(Block::bordered());

        f.render_widget(widget, area);
    }

    pub fn show_footer_helper(_app: &mut App, f: &mut Frame, area: Rect) {
        let help_string = "<?> help | <q> quit";

        f.render_widget(
            Paragraph::new(help_string).alignment(Alignment::Center),
            area,
        );
    }
}
