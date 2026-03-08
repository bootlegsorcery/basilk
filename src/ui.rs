use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_input::Input;

pub struct Ui {}

impl Ui {
    pub fn create_rect_area(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        // Calculate vertical centering with fixed pixel heights for precision
        let popup_height = ((r.height as f32 * percent_y as f32) / 100.0) as u16;
        let top_margin = (r.height.saturating_sub(popup_height)) / 2;

        let popup_layout = Layout::vertical([
            Constraint::Length(top_margin),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(r);

        // Calculate horizontal centering with fixed pixel widths for precision
        let popup_width = ((r.width as f32 * percent_x as f32) / 100.0) as u16;
        let left_margin = (r.width.saturating_sub(popup_width)) / 2;

        Layout::horizontal([
            Constraint::Length(left_margin),
            Constraint::Length(popup_width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
    }

    pub fn create_centered_modal_area(percent_x: u16, height: u16, r: Rect) -> Rect {
        // Calculate vertical position to center the modal
        let top_margin = (r.height.saturating_sub(height)) / 2;

        let popup_layout = Layout::vertical([
            Constraint::Length(top_margin),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

        // Calculate horizontal centering with fixed pixel widths for precision
        let popup_width = ((r.width as f32 * percent_x as f32) / 100.0) as u16;
        let left_margin = (r.width.saturating_sub(popup_width)) / 2;

        Layout::horizontal([
            Constraint::Length(left_margin),
            Constraint::Length(popup_width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
    }

    pub fn create_question_modal(
        text_first_line: &str,
        text_second_line: &str,
        title: &str,
        tip: Option<&str>,
        f: &mut Frame,
        area: Rect,
    ) {
        // Calculate width based on content length
        let content_width = text_first_line.len().max(text_second_line.len());
        let title_width = title.len();
        let tip_width = tip.map(|t| t.len()).unwrap_or(0);
        let max_content_width = content_width.max(title_width).max(tip_width);

        // Add padding for borders and centering
        let popup_width = ((max_content_width + 6) as u16).clamp(30, area.width.min(80));
        let percent_x = ((popup_width as f32 / area.width as f32) * 100.0) as u16;

        // Calculate height based on content + optional tip
        let base_height = 4u16; // 2 lines + 2 borders
        let tip_height = if tip.is_some() { 2 } else { 0 }; // tip line + spacing
        let height = base_height + tip_height;

        let area = Ui::create_centered_modal_area(percent_x, height, area);

        f.render_widget(Clear, area);

        let mut lines = vec![Line::raw(text_first_line), Line::raw(text_second_line)];

        if let Some(tip_text) = tip {
            lines.push(Line::raw(""));
            lines.push(Line::styled(
                tip_text,
                Style::default().add_modifier(Modifier::DIM),
            ));
        }

        f.render_widget(
            Paragraph::new(Text::from(lines))
                .alignment(Alignment::Center)
                .block(Block::bordered().title(title)),
            area,
        );
    }

    pub fn create_input_modal_with_tip(
        title: &str,
        tip: &str,
        f: &mut Frame,
        area: Rect,
        input: &Input,
    ) {
        // Calculate width based on input value length, title length, and tip length
        let input_len = input.value().len();
        let title_len = title.len();
        let tip_len = tip.len();
        let max_content_width = input_len.max(title_len).max(tip_len);

        // Add padding for borders and some extra space for typing
        let popup_width = ((max_content_width + 10) as u16).clamp(35, area.width.min(70));
        let percent_x = ((popup_width as f32 / area.width as f32) * 100.0) as u16;

        // Height: input box (3 lines) + gap + tip (1 line) = 5
        let modal_height = 5u16;
        let modal_area = Ui::create_centered_modal_area(percent_x, modal_height, area);

        // Clear the modal area first to make it solid
        f.render_widget(Clear, modal_area);

        // Split modal area: input widget takes top 3 lines, tip takes bottom 1 line
        let input_area = Rect {
            x: modal_area.x,
            y: modal_area.y,
            width: modal_area.width,
            height: 3,
        };

        let tip_area = Rect {
            x: modal_area.x,
            y: modal_area.y + 3,
            width: modal_area.width,
            height: 1,
        };

        let width = input_area.width.max(3) - 3;
        let scroll = input.visual_scroll(width as usize);

        let input_widget = Paragraph::new(input.value())
            .block(Block::default().borders(Borders::ALL).title(title))
            .scroll((0, scroll as u16));

        f.render_widget(input_widget, input_area);

        // Render tip below the input widget
        f.render_widget(
            Paragraph::new(tip)
                .alignment(Alignment::Center)
                .style(Style::default().add_modifier(Modifier::DIM)),
            tip_area,
        );

        f.set_cursor(
            input_area.x + ((input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            input_area.y + 1,
        )
    }
}
