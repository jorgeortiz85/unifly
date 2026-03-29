use super::{SettingsField, SettingsScreen, SettingsState};

use crate::tui::theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

pub(super) fn panel_rect(area: Rect) -> Rect {
    let width = 62u16.min(area.width.saturating_sub(4));
    let height = 32u16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    Rect::new(area.x + x, area.y + y, width, height)
}

fn centered_rect(area: Rect, cols: u16, rows: u16) -> Rect {
    let width = cols.min(area.width.saturating_sub(2));
    let height = rows.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

impl SettingsScreen {
    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        self.last_area.set(area);

        frame.render_widget(
            Block::default().style(Style::default().bg(theme::bg_base())),
            area,
        );

        let inner = self.render_centered_panel(frame, area);
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

        if let Some(ref error) = self.test_error {
            frame.render_widget(
                Paragraph::new(Span::styled(error, Style::default().fg(theme::error())))
                    .alignment(Alignment::Center),
                layout[2],
            );
        }

        self.render_key_hints(frame, layout[3]);

        match self.state {
            SettingsState::Editing => self.render_editing(frame, layout[1]),
            SettingsState::Testing => self.render_testing(frame, layout[1]),
        }

        let mut selector = self.theme_selector.borrow_mut();
        if let Some(ref mut theme_selector) = *selector {
            let overlay = centered_rect(area, 80, 28);
            frame.render_widget(Clear, overlay);
            frame.render_stateful_widget(opaline::ThemeSelector::new(), overlay, theme_selector);
        }
    }

    fn render_centered_panel(&self, frame: &mut Frame, area: Rect) -> Rect {
        let panel = panel_rect(area);

        frame.render_widget(
            Block::default().style(Style::default().bg(theme::bg_base())),
            panel,
        );

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "Settings",
                    Style::default()
                        .fg(theme::accent_secondary())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ]))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::accent_primary()));

        let inner = block.inner(panel);
        frame.render_widget(block, panel);
        inner
    }

    fn render_input_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        label: &str,
        value: &str,
        active: bool,
        masked: bool,
    ) {
        if area.height < 3 {
            return;
        }

        let label_area = Rect::new(area.x, area.y, area.width, 1);
        let label_style = if active {
            Style::default().fg(theme::accent_secondary())
        } else {
            Style::default().fg(theme::text_secondary())
        };
        frame.render_widget(Paragraph::new(Span::styled(label, label_style)), label_area);

        let display = if masked && !value.is_empty() {
            "\u{25CF}".repeat(value.len())
        } else {
            value.to_string()
        };

        let border_color = if active {
            theme::accent_primary()
        } else {
            theme::border_unfocused()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let block_area = Rect::new(area.x, area.y + 1, area.width, 3.min(area.height - 1));
        let inner = block.inner(block_area);
        frame.render_widget(block, block_area);

        let text = if active {
            format!("{display}\u{2588}")
        } else {
            display
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                text,
                Style::default().fg(theme::accent_secondary()),
            )),
            inner,
        );
    }

    fn render_auth_selector(&self, frame: &mut Frame, area: Rect) {
        if area.height < 3 {
            return;
        }

        let active = self.active_field == SettingsField::AuthMode;
        let label_style = if active {
            Style::default().fg(theme::accent_secondary())
        } else {
            Style::default().fg(theme::text_secondary())
        };
        frame.render_widget(
            Paragraph::new(Span::styled("  Auth Mode", label_style)),
            Rect::new(area.x, area.y, area.width, 1),
        );

        let border_color = if active {
            theme::accent_primary()
        } else {
            theme::border_unfocused()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let block_area = Rect::new(area.x, area.y + 1, area.width, 3.min(area.height - 1));
        let inner = block.inner(block_area);
        frame.render_widget(block, block_area);

        let arrow_style = if active {
            Style::default().fg(theme::accent_primary())
        } else {
            Style::default().fg(theme::border_unfocused())
        };
        let value_style = if active {
            Style::default()
                .fg(theme::accent_secondary())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::text_secondary())
        };

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" \u{25C2} ", arrow_style),
                Span::styled(self.auth_mode.label(), value_style),
                Span::styled(" \u{25B8}", arrow_style),
            ])),
            inner,
        );
    }

    fn render_toggle(&self, frame: &mut Frame, area: Rect, label: &str, value: bool, active: bool) {
        if area.height < 1 {
            return;
        }

        let marker = if value { "[\u{2713}]" } else { "[ ]" };
        let marker_style = if active {
            Style::default().fg(theme::accent_primary())
        } else if value {
            Style::default().fg(theme::success())
        } else {
            Style::default().fg(theme::border_unfocused())
        };
        let label_style = if active {
            Style::default().fg(theme::accent_secondary())
        } else {
            Style::default().fg(theme::text_secondary())
        };

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("  {marker} "), marker_style),
                Span::styled(label, label_style),
            ])),
            area,
        );
    }

    fn render_theme_field(&self, frame: &mut Frame, area: Rect) {
        if area.height < 1 {
            return;
        }

        let active = self.active_field == SettingsField::Theme;
        let label_style = if active {
            Style::default().fg(theme::accent_secondary())
        } else {
            Style::default().fg(theme::text_secondary())
        };
        let value_style = if active {
            Style::default()
                .fg(theme::accent_primary())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::text_primary())
        };
        let hint = if active { "  (Enter to change)" } else { "" };

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  Theme: ", label_style),
                Span::styled(opaline::current().meta.name.clone(), value_style),
                Span::styled(hint, Style::default().fg(theme::border_unfocused())),
            ])),
            area,
        );
    }

    fn render_editing(&self, frame: &mut Frame, area: Rect) {
        let field_layout = self.field_layout();
        let mut constraints: Vec<_> = field_layout
            .iter()
            .map(|(_, height)| Constraint::Length(*height))
            .collect();
        constraints.push(Constraint::Min(0));

        let fields_area = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        let chunks = Layout::vertical(constraints).split(fields_area);

        for ((field, _), chunk) in field_layout.iter().zip(chunks.iter().copied()) {
            match field {
                SettingsField::Url => self.render_input_field(
                    frame,
                    chunk,
                    "  Controller URL",
                    &self.url_input,
                    self.active_field == SettingsField::Url,
                    false,
                ),
                SettingsField::AuthMode => self.render_auth_selector(frame, chunk),
                SettingsField::ApiKey => self.render_input_field(
                    frame,
                    chunk,
                    "  API Key",
                    &self.api_key_input,
                    self.active_field == SettingsField::ApiKey,
                    true,
                ),
                SettingsField::Username => self.render_input_field(
                    frame,
                    chunk,
                    "  Username",
                    &self.username_input,
                    self.active_field == SettingsField::Username,
                    false,
                ),
                SettingsField::Password => self.render_input_field(
                    frame,
                    chunk,
                    "  Password",
                    &self.password_input,
                    self.active_field == SettingsField::Password,
                    !self.show_password,
                ),
                SettingsField::Site => self.render_input_field(
                    frame,
                    chunk,
                    "  Site",
                    &self.site_input,
                    self.active_field == SettingsField::Site,
                    false,
                ),
                SettingsField::Insecure => self.render_toggle(
                    frame,
                    chunk,
                    "Skip TLS verification (insecure)",
                    self.insecure,
                    self.active_field == SettingsField::Insecure,
                ),
                SettingsField::Theme => self.render_theme_field(frame, chunk),
            }
        }
    }

    fn render_testing(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

        let throbber = throbber_widgets_tui::Throbber::default()
            .label("  Testing connection...")
            .style(Style::default().fg(theme::accent_secondary()))
            .throbber_style(Style::default().fg(theme::accent_primary()));

        frame.render_stateful_widget(throbber, layout[1], &mut self.throbber_state.clone());
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("  Connecting to {}", self.url_input.trim()),
                Style::default().fg(theme::border_unfocused()),
            )),
            layout[2],
        );
    }

    fn render_key_hints(&self, frame: &mut Frame, area: Rect) {
        let hints = match self.state {
            SettingsState::Editing => {
                if self.active_field == SettingsField::AuthMode {
                    "\u{25C2}/\u{25B8} select  Tab next  Enter test & save  Esc cancel"
                } else if self.active_field == SettingsField::Insecure {
                    "Space toggle  Tab next  Enter test & save  Esc cancel"
                } else if self.active_field == SettingsField::Password {
                    "Ctrl+U reveal  Tab next  Enter test & save  Esc cancel"
                } else if self.active_field == SettingsField::Theme {
                    "Enter choose theme  Tab next  Esc cancel"
                } else {
                    "Tab next  Shift+Tab prev  Enter test & save  Esc cancel"
                }
            }
            SettingsState::Testing => "Esc cancel",
        };

        frame.render_widget(
            Paragraph::new(Span::styled(hints, theme::key_hint())).alignment(Alignment::Center),
            area,
        );
    }
}
