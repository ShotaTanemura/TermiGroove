use ratatui::layout::Alignment;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
};
use std::time::{SystemTime, UNIX_EPOCH};
use tui_big_text::{BigText, PixelSize};

use crate::app_state::{AppState, FocusPane, Mode};

const HEADER_TITLE: &str = "WELCOME TO TERMIGROOVE";
const HEADER_SUBTITLE: &str = "Load your samples...";
const RIGHT_TITLE: &str = "Selected (Enter = To Pads)";

pub fn draw_ui(frame: &mut Frame, state: &AppState) {
    let (header_area, body_area, footer_area) = vertical_layout(frame);
    match state.mode {
        Mode::Browse => {
            render_header(frame, header_area);
            let (left_area, right_area) = body_layout(body_area);
            // Render explorer in left area (help is provided via explorer theme title bottom)
            frame.render_widget(&state.file_explorer.widget(), left_area);
            render_right(frame, right_area, state);
        }
        Mode::Pads => {
            // No header in Pads mode (leave header area empty)
            render_pads(frame, body_area, state);
        }
    }
    render_footer(frame, footer_area, state);
}

fn vertical_layout(
    frame: &mut Frame,
) -> (
    ratatui::prelude::Rect,
    ratatui::prelude::Rect,
    ratatui::prelude::Rect,
) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // header
            Constraint::Min(1),    // body
            Constraint::Length(1), // footer
        ])
        .split(size);
    (chunks[0], chunks[1], chunks[2])
}

fn body_layout(area: ratatui::prelude::Rect) -> (ratatui::prelude::Rect, ratatui::prelude::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);
    (chunks[0], chunks[1])
}

fn render_header(frame: &mut Frame, area: ratatui::prelude::Rect) {
    // Split header area: big text + subtitle line
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Length(1)])
        .split(area);

    let big = BigText::builder()
        .pixel_size(PixelSize::Full)
        .style(Style::default().fg(Color::Green))
        .lines(vec![HEADER_TITLE.into()])
        .alignment(Alignment::Center)
        .build();
    frame.render_widget(big, chunks[0]);

    let subtitle =
        Paragraph::new(Line::from(Span::raw(HEADER_SUBTITLE))).alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[1]);
}

fn render_right(frame: &mut Frame, area: ratatui::prelude::Rect, state: &AppState) {
    let mut right_block = Block::default()
        .title(RIGHT_TITLE)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .padding(Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        });

    if matches!(state.focus, FocusPane::RightSelected) {
        right_block = right_block.border_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        );
    }

    // Render names (not paths) using a stateful List with a visible cursor highlight
    let items: Vec<ListItem> = state
        .selection
        .items
        .iter()
        .map(|p| {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            ListItem::new(Line::from(Span::styled(
                name,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )))
        })
        .collect();

    let list = List::new(items)
        .block(right_block)
        .highlight_style(
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !state.selection.items.is_empty() {
        list_state.select(Some(state.selection.right_idx));
    } else {
        list_state.select(None);
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_footer(frame: &mut Frame, area: ratatui::prelude::Rect, state: &AppState) {
    let footer = Paragraph::new(Line::from(vec![Span::raw(state.status_message.clone())]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(footer, area);
}

fn render_pads(frame: &mut Frame, area: ratatui::prelude::Rect, state: &AppState) {
    // Determine grid based on number of pads
    let total = state.pads.key_to_slot.len().max(1);
    let cols = total.clamp(1, 10) as u16; // cap columns for readability
    let rows = ((total as f32) / (cols as f32)).ceil() as u16;

    // Build column constraints
    let mut col_constraints = Vec::with_capacity(cols as usize);
    for _ in 0..cols {
        col_constraints.push(Constraint::Percentage(100 / cols));
    }

    // Split into rows first
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100 / rows); rows as usize])
        .split(area);

    // Flatten key/slot items in a stable order
    let items: Vec<(char, String)> = state
        .pads
        .key_to_slot
        .iter()
        .map(|(k, slot)| (*k, slot.file_name.clone()))
        .collect();

    let mut idx: usize = 0;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    for row_area in row_chunks.iter().copied() {
        let cols_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints.clone())
            .split(row_area);
        for cell in cols_areas.iter().copied() {
            if idx >= items.len() {
                break;
            }
            let (key, file_name) = &items[idx];
            idx += 1;

            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green));
            // Active highlight (revert after 150ms from last press)
            let is_active = state
                .pads
                .last_press_ms
                .get(key)
                .map(|t| now_ms.saturating_sub(*t) <= 150)
                .unwrap_or(false);
            if is_active {
                block = block.border_style(
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                );
            }

            // Compose key + filename lines
            let key_line = Line::from(Span::styled(
                key.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
            let name_line = Line::from(Span::styled(
                truncate_middle(file_name, 18),
                Style::default().fg(Color::Green),
            ));
            let para = Paragraph::new(vec![key_line, name_line])
                .alignment(Alignment::Center)
                .block(block);
            frame.render_widget(para, cell);
        }
    }
}

fn truncate_middle(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    if max <= 3 {
        return s[..max].to_string();
    }
    let half = (max - 3) / 2;
    format!("{}...{}", &s[..half], &s[s.len() - half..])
}
