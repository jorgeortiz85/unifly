use super::{AuthMode, CredentialField, OnboardingScreen, WizardStep};

use crate::tui::forms::widgets::render_input_field;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::tui::theme;

impl OnboardingScreen {
    pub(super) fn render_screen(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::default().style(Style::default().bg(theme::bg_base())),
            area,
        );

        let inner = self.render_centered_panel(frame, area);
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

        self.render_step_indicator(frame, layout[0]);
        self.render_key_hints(frame, layout[3]);

        if let Some(ref error) = self.error {
            frame.render_widget(
                Paragraph::new(Span::styled(error, Style::default().fg(theme::error())))
                    .alignment(Alignment::Center),
                layout[2],
            );
        } else if let Some(ref error) = self.test_error {
            frame.render_widget(
                Paragraph::new(Span::styled(error, Style::default().fg(theme::error())))
                    .alignment(Alignment::Center),
                layout[2],
            );
        }

        let content = layout[1];
        match self.step {
            WizardStep::Welcome => self.render_welcome(frame, content),
            WizardStep::Url => self.render_url(frame, content),
            WizardStep::AuthMode => self.render_auth_mode(frame, content),
            WizardStep::Credentials => self.render_credentials(frame, content),
            WizardStep::Site => self.render_site(frame, content),
            WizardStep::Testing => self.render_testing(frame, content),
            WizardStep::Done => self.render_done(frame, content),
        }
    }

    fn render_centered_panel(&self, frame: &mut Frame, area: Rect) -> Rect {
        let panel_w = 62u16.min(area.width.saturating_sub(4));
        let panel_h = 22u16.min(area.height.saturating_sub(2));
        let x = (area.width.saturating_sub(panel_w)) / 2;
        let y = (area.height.saturating_sub(panel_h)) / 2;
        let panel = Rect::new(area.x + x, area.y + y, panel_w, panel_h);

        frame.render_widget(
            Block::default().style(Style::default().bg(theme::bg_base())),
            panel,
        );

        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    "UniFi Setup",
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

    fn render_step_indicator(&self, frame: &mut Frame, area: Rect) {
        let steps = ["URL", "Auth", "Keys", "Site", "Test"];
        let current = self.step.index();

        let spans: Vec<Span> = steps
            .iter()
            .enumerate()
            .flat_map(|(idx, label)| {
                let step_num = idx + 1;
                let style = match step_num.cmp(&current) {
                    std::cmp::Ordering::Equal => Style::default()
                        .fg(theme::accent_primary())
                        .add_modifier(Modifier::BOLD),
                    std::cmp::Ordering::Less => Style::default().fg(theme::success()),
                    std::cmp::Ordering::Greater => Style::default().fg(theme::border_unfocused()),
                };
                let separator = if idx < steps.len() - 1 {
                    Span::styled(" > ", Style::default().fg(theme::border_unfocused()))
                } else {
                    Span::raw("")
                };
                vec![
                    Span::styled(format!("{step_num} {label}"), style),
                    separator,
                ]
            })
            .collect();

        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );
    }
    fn render_key_hints(&self, frame: &mut Frame, area: Rect) {
        let hints = match self.step {
            WizardStep::Welcome => "Enter continue  Ctrl+C quit",
            WizardStep::Url | WizardStep::Site => "Enter next  Esc back  Ctrl+C quit",
            WizardStep::AuthMode => "Up/Down select  Enter confirm  Esc back",
            WizardStep::Credentials => "Tab next field  Enter next  Esc back",
            WizardStep::Testing => "Esc cancel",
            WizardStep::Done => "Enter connect!",
        };

        frame.render_widget(
            Paragraph::new(Span::styled(hints, theme::key_hint())).alignment(Alignment::Center),
            area,
        );
    }

    fn render_welcome(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                "Welcome to UniFi TUI",
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            )]))
            .alignment(Alignment::Center),
            layout[0],
        );

        let description = vec![
            Line::from(Span::styled(
                "No configuration found. This wizard will help you",
                Style::default().fg(theme::text_secondary()),
            )),
            Line::from(Span::styled(
                "connect to your UniFi controller.",
                Style::default().fg(theme::text_secondary()),
            )),
        ];
        frame.render_widget(
            Paragraph::new(description).alignment(Alignment::Center),
            layout[1],
        );

        frame.render_widget(
            Paragraph::new(Span::styled(
                "Press Enter to begin",
                Style::default().fg(theme::accent_primary()),
            ))
            .alignment(Alignment::Center),
            layout[2],
        );
    }

    fn render_url(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                "Enter your controller URL",
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            layout[0],
        );

        render_input_field(
            frame,
            layout[1],
            "  Controller URL",
            &self.draft.url,
            true,
            false,
        );
    }

    fn render_auth_mode(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                "Choose authentication method",
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            layout[0],
        );

        let list_area = Rect::new(
            layout[1].x + 3,
            layout[1].y,
            layout[1].width.saturating_sub(6),
            layout[1].height,
        );

        let mut lines = Vec::new();
        for (idx, mode) in AuthMode::ALL.iter().enumerate() {
            let selected = idx == self.auth_mode_index;
            let marker = if selected { "\u{25B8} " } else { "  " };
            let marker_style = if selected {
                Style::default().fg(theme::accent_primary())
            } else {
                Style::default().fg(theme::border_unfocused())
            };
            let label_style = if selected {
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::text_secondary())
            };

            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(mode.label(), label_style),
            ]));
            lines.push(Line::from(Span::styled(
                format!("    {}", mode.description()),
                Style::default().fg(theme::border_unfocused()),
            )));
            lines.push(Line::from(""));
        }

        frame.render_widget(Paragraph::new(lines), list_area);
    }

    fn render_credentials(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                "Enter credentials",
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            layout[0],
        );

        let fields_area = Rect::new(
            layout[1].x + 2,
            layout[1].y,
            layout[1].width.saturating_sub(4),
            layout[1].height,
        );

        let mut y_offset = 0u16;

        if matches!(self.draft.auth_mode, AuthMode::ApiKey | AuthMode::Hybrid) {
            let input_area = Rect::new(
                fields_area.x,
                fields_area.y + y_offset,
                fields_area.width,
                4,
            );
            render_input_field(
                frame,
                input_area,
                "  API Key",
                &self.draft.api_key,
                self.cred_field == CredentialField::ApiKey,
                true,
            );
            y_offset += 5;
        }

        if matches!(self.draft.auth_mode, AuthMode::Legacy | AuthMode::Hybrid) {
            let username_area = Rect::new(
                fields_area.x,
                fields_area.y + y_offset,
                fields_area.width,
                4,
            );
            render_input_field(
                frame,
                username_area,
                "  Username",
                &self.draft.username,
                self.cred_field == CredentialField::Username,
                false,
            );
            y_offset += 5;

            let password_area = Rect::new(
                fields_area.x,
                fields_area.y + y_offset,
                fields_area.width,
                4,
            );
            render_input_field(
                frame,
                password_area,
                "  Password",
                &self.draft.password,
                self.cred_field == CredentialField::Password,
                !self.show_password,
            );
        }
    }

    fn render_site(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                "Enter site name",
                Style::default()
                    .fg(theme::accent_secondary())
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            layout[0],
        );

        render_input_field(frame, layout[1], "  Site", &self.draft.site, true, false);
    }

    fn render_testing(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

        if self.testing {
            let throbber = throbber_widgets_tui::Throbber::default()
                .label("  Testing connection...")
                .style(Style::default().fg(theme::accent_secondary()))
                .throbber_style(Style::default().fg(theme::accent_primary()));

            frame.render_stateful_widget(throbber, layout[0], &mut self.throbber_state.clone());
            frame.render_widget(
                Paragraph::new(Span::styled(
                    format!("  Connecting to {}", self.draft.url.trim()),
                    Style::default().fg(theme::border_unfocused()),
                )),
                layout[1],
            );
        } else if let Some(ref error) = self.test_error {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(
                        "  Connection failed",
                        Style::default()
                            .fg(theme::error())
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  {error}"),
                        Style::default().fg(theme::error()),
                    )),
                ])
                .wrap(Wrap { trim: false }),
                area,
            );
        }
    }

    fn render_done(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "  \u{2713} ",
                    Style::default()
                        .fg(theme::success())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Connection successful!",
                    Style::default()
                        .fg(theme::success())
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .alignment(Alignment::Center),
            layout[0],
        );

        let saved_path = crate::config::config_path();
        let details = vec![
            Line::from(Span::styled(
                "  Profile: default".to_string(),
                Style::default().fg(theme::text_secondary()),
            )),
            Line::from(Span::styled(
                format!("  Controller: {}", self.draft.url.trim()),
                Style::default().fg(theme::text_secondary()),
            )),
            Line::from(Span::styled(
                format!("  Config saved: {}", saved_path.display()),
                Style::default().fg(theme::text_secondary()),
            )),
        ];
        frame.render_widget(Paragraph::new(details), layout[1]);

        frame.render_widget(
            Paragraph::new(Span::styled(
                "Press Enter to launch the dashboard",
                Style::default().fg(theme::accent_primary()),
            ))
            .alignment(Alignment::Center),
            layout[2],
        );
    }
}
