use ratatui::layout::Alignment;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    prelude::Buffer,
    style::{Color, Modifier, Style},
    symbols::border::DOUBLE,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph, Widget, WidgetRef},
};
use std::time::{SystemTime, UNIX_EPOCH};
use tui_big_text::{BigText, PixelSize};
use tui_popup::{Popup, SizedWidgetRef};

use crate::application::state::ApplicationState;
use crate::domain::r#loop::LoopState;
use crate::presentation::ViewModel;
use crate::presentation::{FocusPane, Mode, PopupFocus};

const HEADER_TITLE: &str = "WELCOME TO TERMIGROOVE";
const HEADER_SUBTITLE: &str = "Load your samples...";
const RIGHT_TITLE: &str = "Selected (Enter = To Pads)";

pub fn draw_ui(frame: &mut Frame, view_model: &ViewModel, app_state: &ApplicationState) {
    match view_model.mode {
        Mode::Browse => {
            let (header_area, body_area, footer_area) = vertical_layout(frame);
            render_header(frame, header_area);
            let (left_area, right_area) = body_layout(body_area);
            frame.render_widget(&view_model.file_explorer.widget(), left_area);
            render_right(frame, right_area, view_model, app_state);
            render_footer(frame, footer_area, view_model);
        }
        Mode::Pads => {
            let size = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(size);
            let summary_area = chunks[0];
            let body_area = chunks[1];
            let footer_area = chunks[2];
            render_summary_box(frame, summary_area, view_model, app_state);
            render_pads(frame, body_area, view_model, app_state);
            render_footer(frame, footer_area, view_model);
            if view_model.is_bpm_popup_open() {
                render_popup(frame, size, view_model, app_state);
            }
        }
    }
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

fn render_right(
    frame: &mut Frame,
    area: ratatui::prelude::Rect,
    view_model: &ViewModel,
    app_state: &ApplicationState,
) {
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

    if matches!(view_model.focus, FocusPane::RightSelected) {
        right_block = right_block.border_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        );
    }

    // Render names (not paths) using a stateful List with a visible cursor highlight
    let items: Vec<ListItem> = app_state
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
    if !app_state.selection.items.is_empty() {
        list_state.select(Some(app_state.selection.right_idx));
    } else {
        list_state.select(None);
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_footer(frame: &mut Frame, area: ratatui::prelude::Rect, view_model: &ViewModel) {
    let footer = Paragraph::new(Line::from(vec![Span::raw(
        view_model.status_message.clone(),
    )]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(footer, area);
}

fn render_pads(
    frame: &mut Frame,
    area: ratatui::prelude::Rect,
    _view_model: &ViewModel,
    app_state: &ApplicationState,
) {
    // Determine grid based on number of pads
    let total = app_state.pads.key_to_slot.len().max(1);
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
    let items: Vec<(char, String)> = app_state
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
            let is_active = app_state
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

fn render_summary_box(
    frame: &mut Frame,
    area: Rect,
    view_model: &ViewModel,
    app_state: &ApplicationState,
) {
    // Base green frame consistent with pads styling
    let border_style = Style::default().fg(Color::Green);

    // Outer block with some padding
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 1,
        });
    frame.render_widget(block, area);

    // Compute a ring rectangle inside the green frame, leaving margin for text
    let ring_rect = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    // Focus ring when SummaryBox is focused (drawn outside of the text area)
    let is_selected = matches!(
        view_model.popup_focus(),
        PopupFocus::PopupFieldBpm
            | PopupFocus::PopupFieldBars
            | PopupFocus::PopupOk
            | PopupFocus::PopupCancel
    );
    let selected_fill = Color::Rgb(40, 80, 40);

    let (focus_borders, focus_style, focus_bg_style) = if is_selected {
        (
            Borders::ALL,
            Style::default().fg(Color::Green),
            Some(Style::default().bg(selected_fill)),
        )
    } else if matches!(view_model.popup_focus(), PopupFocus::SummaryBox) {
        (
            Borders::ALL,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            None,
        )
    } else {
        (Borders::NONE, Style::default(), None)
    };

    let content_lines = 3;
    let minimal_height = content_lines + 2;
    let focus_rect = Rect {
        x: ring_rect.x,
        y: ring_rect.y,
        width: ring_rect.width,
        height: ring_rect.height.min(minimal_height),
    };
    let mut focus_block = Block::default()
        .borders(focus_borders)
        .border_style(focus_style);
    if let Some(style) = focus_bg_style {
        focus_block = focus_block.style(style);
    }
    let focus_inner = focus_block.inner(focus_rect);
    frame.render_widget(focus_block, focus_rect);

    // Content area inside the focus ring so the border doesn't overlap text
    let margin = if focus_borders == Borders::ALL { 0 } else { 1 };
    let content_rect = focus_inner.inner(Margin {
        horizontal: margin,
        vertical: margin,
    });

    // Labels left, values right, with wide spacing
    let col_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content_rect);

    let left = col_chunks[0];
    let right = col_chunks[1];

    let labels = Paragraph::new(vec![
        Line::from(Span::styled("bpm:", Style::default().fg(Color::Green))),
        Line::from(Span::styled("bars:", Style::default().fg(Color::Green))),
        Line::from(Span::styled("state:", Style::default().fg(Color::Green))),
    ])
    .alignment(Alignment::Left);

    let mut value_lines = vec![
        Line::from(Span::styled(
            app_state.get_bpm().to_string(),
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            app_state.get_bars().to_string(),
            Style::default().fg(Color::Green),
        )),
    ];
    let (label, style) = match app_state.loop_state() {
        LoopState::Paused { .. } => (
            "PAUSED",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        ),
        LoopState::Playing { .. } => (
            "playing",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        LoopState::Recording { .. } => (
            "recording",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        LoopState::Ready { .. } => ("ready", Style::default().fg(Color::Green)),
        LoopState::Idle => ("idle", Style::default().fg(Color::White)),
    };
    value_lines.push(Line::from(Span::styled(label, style)));
    let values = Paragraph::new(value_lines).alignment(Alignment::Right);

    // Render content
    frame.render_widget(labels, left);
    frame.render_widget(values, right);
}

fn render_popup(
    frame: &mut Frame,
    area: Rect,
    view_model: &ViewModel,
    _app_state: &ApplicationState,
) {
    let content = PopupContent { view_model };
    let popup = Popup::new(content)
        .title(Line::from("Configure tempo & loop").centered())
        .style(Style::default().bg(Color::Rgb(51, 114, 50)))
        .border_set(DOUBLE)
        .border_style(Style::default().fg(Color::White))
        .borders(Borders::ALL);

    frame.render_widget_ref(popup, area);
}

#[derive(Debug)]
struct PopupContent<'a> {
    view_model: &'a ViewModel,
}

impl<'a> SizedWidgetRef for PopupContent<'a> {
    fn width(&self) -> usize {
        44
    }

    fn height(&self) -> usize {
        10
    }
}

impl<'a> Widget for PopupContent<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl<'a> ratatui::widgets::WidgetRef for PopupContent<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // buf.set_style(area, Style::default().bg(Color::Rgb(8, 24, 8)));
        buf.set_style(area, Style::default().bg(Color::Rgb(51, 114, 50)));

        let padded = area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        if padded.width == 0 || padded.height == 0 {
            return;
        }

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(padded);

        let inputs_area = sections[0];
        let buttons_area = sections[1];

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(inputs_area);

        render_popup_input_row(
            buf,
            rows[0],
            "bpm",
            self.view_model.draft_bpm().value(),
            matches!(self.view_model.popup_focus(), PopupFocus::PopupFieldBpm,),
        );
        render_popup_input_row(
            buf,
            rows[1],
            "bars",
            self.view_model.draft_bars().value(),
            matches!(self.view_model.popup_focus(), PopupFocus::PopupFieldBars,),
        );

        let button_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Percentage(38),
            ])
            .split(buttons_area);

        render_popup_button(
            buf,
            button_row[1],
            "[ OK ]",
            matches!(self.view_model.popup_focus(), PopupFocus::PopupOk),
        );
        render_popup_button(
            buf,
            button_row[2],
            "[ Cancel ]",
            matches!(self.view_model.popup_focus(), PopupFocus::PopupCancel),
        );
    }
}

fn render_popup_input_row(buf: &mut Buffer, area: Rect, label: &str, value: &str, focused: bool) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    buf.set_style(area, Style::default().bg(Color::Rgb(51, 114, 50)));
    // buf.set_style(area, Style::default().bg(Color::Rgb(8, 24, 8)));

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(6), Constraint::Min(1)])
        .split(area);

    let label_col = columns[0];
    let value_col = columns[1];

    if label_col.width > 0 {
        let label_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(label_col);
        let center_area = label_rows[1];
        let line = Line::from(vec![Span::styled(
            format!("{}:", label),
            Style::default().fg(Color::White),
        )]);
        Paragraph::new(line)
            .alignment(Alignment::Left)
            .render(center_area, buf);
    }

    if value_col.width == 0 {
        return;
    }

    let mut border_style = Style::default().fg(Color::White);
    if focused {
        border_style = border_style.add_modifier(Modifier::BOLD);
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(value_col);
    block.render(value_col, buf);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut buf_style = Style::default().bg(Color::Rgb(40, 72, 40));
    if focused {
        buf_style = buf_style.bg(Color::Rgb(0, 32, 0));
    }
    buf.set_style(inner, buf_style);

    let mut value_style = Style::default().fg(Color::White).bg(Color::Rgb(40, 72, 40));
    if focused {
        value_style = value_style
            .add_modifier(Modifier::BOLD)
            .bg(Color::Rgb(0, 32, 0));
    }

    let text = Line::from(vec![Span::styled(value.to_string(), value_style)]);
    Paragraph::new(text)
        .alignment(Alignment::Right)
        .render(inner, buf);
}

fn render_popup_button(buf: &mut Buffer, area: Rect, label: &str, focused: bool) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    buf.set_style(area, Style::default().bg(Color::Rgb(51, 114, 50)));
    // buf.set_style(area, Style::default().bg(Color::Rgb(8, 24, 8)));

    let mut style = Style::default().fg(Color::White);
    if focused {
        style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
    }

    let line = Line::from(vec![Span::styled(label, style)]);
    Paragraph::new(line)
        .alignment(Alignment::Center)
        .render(area, buf);
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
