use super::super::{OnboardingScreen, WizardStep};

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::tui::theme;

use super::steps;

pub(super) fn render_screen(screen: &OnboardingScreen, frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Block::default().style(Style::default().bg(theme::bg_base())),
        area,
    );

    let inner = render_centered_panel(frame, area);
    let layout = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    render_step_indicator(screen, frame, layout[0]);
    render_key_hints(screen, frame, layout[3]);

    if let Some(ref error) = screen.error {
        frame.render_widget(
            Paragraph::new(Span::styled(error, Style::default().fg(theme::error())))
                .alignment(Alignment::Center),
            layout[2],
        );
    } else if let Some(ref error) = screen.test_error {
        frame.render_widget(
            Paragraph::new(Span::styled(error, Style::default().fg(theme::error())))
                .alignment(Alignment::Center),
            layout[2],
        );
    }

    let content = layout[1];
    match screen.step {
        WizardStep::Welcome => steps::render_welcome(screen, frame, content),
        WizardStep::Url => steps::render_url(screen, frame, content),
        WizardStep::AuthMode => steps::render_auth_mode(screen, frame, content),
        WizardStep::Credentials => steps::render_credentials(screen, frame, content),
        WizardStep::Site => steps::render_site(screen, frame, content),
        WizardStep::Testing => steps::render_testing(screen, frame, content),
        WizardStep::Done => steps::render_done(screen, frame, content),
    }
}

fn render_centered_panel(frame: &mut Frame, area: Rect) -> Rect {
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

fn render_step_indicator(screen: &OnboardingScreen, frame: &mut Frame, area: Rect) {
    let steps = ["URL", "Auth", "Keys", "Site", "Test"];
    let current = screen.step.index();

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

fn render_key_hints(screen: &OnboardingScreen, frame: &mut Frame, area: Rect) {
    let hints = match screen.step {
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
