//! Drawing. The colours mean the same as in every PromptShelf app: rose for a
//! value from the conversation, green for a value from the repo or files, and a
//! dashed blank for a value nobody worked out yet.

use promptshelf_core::template::{Piece, Source};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use super::app::{App, Mode};

// The brand's dark-mode OKLCH tokens, converted to sRGB.
const ROSE: Color = Color::Rgb(241, 141, 181); // oklch(76% 0.13 355)
const GREEN: Color = Color::Rgb(104, 200, 148); // oklch(76% 0.12 158)
const ACCENT: Color = Color::Rgb(140, 186, 247); // oklch(78% 0.1 255)
const PIN: Color = Color::Rgb(242, 140, 92); // oklch(74% 0.14 45)
const MUTED: Color = Color::Rgb(161, 165, 172); // oklch(72% 0.012 265)

fn source_color(source: Source) -> Option<Color> {
    match source {
        Source::Conversation => Some(ROSE),
        Source::Branch | Source::Command => Some(GREEN),
        _ => None,
    }
}

fn source_label(source: Source, detail: Option<&str>) -> String {
    match (source, detail) {
        (Source::Command, Some(cmd)) => cmd.to_string(),
        (Source::Given, _) => "typed".into(),
        (Source::Default, _) => "default".into(),
        (Source::LastUsed, _) => "last used".into(),
        (Source::Branch, _) => "git branch".into(),
        (Source::Clipboard, _) => "clipboard".into(),
        (Source::Selection, _) => "selection".into(),
        (Source::Conversation, _) => "conversation".into(),
        (Source::Command, None) => "command".into(),
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let [top, body, help] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(f.area());
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)]).areas(body);

    let search = Line::from(vec![
        Span::styled(" / ", Style::new().fg(ACCENT)),
        Span::raw(app.query.clone()),
        Span::styled("▏", Style::new().fg(ACCENT)),
    ]);
    f.render_widget(Paragraph::new(search), top);

    draw_list(f, app, left);
    match app.mode {
        Mode::Browse => draw_preview(f, app, right),
        Mode::Fill { .. } => draw_form(f, app, right),
    }

    let keys = match app.mode {
        Mode::Browse => "↵ use  ↑↓ move  1-9 pins  ^T pin  esc quit",
        Mode::Fill { .. } => "↵ next/done  tab move  ←→ choose  esc back",
    };
    let line = match &app.status {
        Some(s) => Line::styled(format!(" {s}"), Style::new().fg(PIN)),
        None => Line::styled(format!(" {keys}"), Style::new().fg(MUTED)),
    };
    f.render_widget(Paragraph::new(line), help);
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|p| {
            let mut spans = Vec::new();
            match app.pins.position(&p.full_id()) {
                Some(n) => spans.push(Span::styled(format!("{n} "), Style::new().fg(PIN))),
                None => spans.push(Span::raw("  ")),
            }
            spans.push(Span::raw(p.title.clone()));
            spans.push(Span::styled(
                format!("  {}", p.full_id()),
                Style::new().fg(MUTED),
            ));
            ListItem::new(Line::from(spans))
        })
        .collect();
    let title = format!(" Prompts ({}) ", app.items.len());
    let list = List::new(items)
        .block(Block::new().borders(Borders::RIGHT).title(title))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default().with_selected(Some(app.selected));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_preview(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(p) = app.current() else {
        let empty = Paragraph::new("No prompts. Add one with `shelf new` or `shelf save`.")
            .style(Style::new().fg(MUTED));
        f.render_widget(empty, area.inner(ratatui::layout::Margin::new(1, 0)));
        return;
    };
    let title = format!(" {} · {} ", p.title, p.kind.as_str());
    let block = Block::new().title(title);
    let text = match app.preview() {
        Some(Ok(preview)) => styled(&preview.pieces),
        Some(Err(e)) => Text::styled(e.clone(), Style::new().fg(PIN)),
        None => Text::default(),
    };
    let para = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, area.inner(ratatui::layout::Margin::new(1, 0)));
}

/// The filled prompt with numbered, coloured values and footnotes under it.
fn styled(pieces: &[Piece]) -> Text<'static> {
    let mut lines: Vec<Line> = vec![Line::default()];
    let mut notes: Vec<Line> = Vec::new();
    let mut n = 0;
    let push = |lines: &mut Vec<Line>, s: &str, style: Style| {
        for (i, part) in s.split('\n').enumerate() {
            if i > 0 {
                lines.push(Line::default());
            }
            if !part.is_empty() {
                lines
                    .last_mut()
                    .expect("one line")
                    .push_span(Span::styled(part.to_string(), style));
            }
        }
    };
    for piece in pieces {
        match piece {
            Piece::Text(t) => push(&mut lines, t, Style::new()),
            Piece::Value { name, value } => {
                n += 1;
                let color = source_color(value.source);
                let mut style = Style::new().add_modifier(Modifier::BOLD);
                if let Some(c) = color {
                    style = style.fg(c);
                }
                push(&mut lines, &value.value, style);
                push(
                    &mut lines,
                    &superscript(n),
                    Style::new().fg(color.unwrap_or(MUTED)),
                );
                notes.push(Line::from(vec![
                    Span::styled(format!("{n} "), Style::new().fg(color.unwrap_or(MUTED))),
                    Span::styled(
                        format!(
                            "{name}: {}",
                            source_label(value.source, value.detail.as_deref())
                        ),
                        Style::new().fg(MUTED),
                    ),
                ]));
            }
            Piece::Open { name } => {
                n += 1;
                push(
                    &mut lines,
                    &format!("╌{name}╌"),
                    Style::new().fg(MUTED).add_modifier(Modifier::UNDERLINED),
                );
                notes.push(Line::styled(
                    format!("{n} {name}: open; ↵ to fill"),
                    Style::new().fg(MUTED),
                ));
            }
        }
    }
    if !notes.is_empty() {
        lines.push(Line::default());
        lines.push(Line::styled("───", Style::new().fg(MUTED)));
        lines.extend(notes);
    }
    Text::from(lines)
}

fn superscript(n: usize) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    n.to_string()
        .chars()
        .map(|c| DIGITS[c.to_digit(10).unwrap_or(0) as usize])
        .collect()
}

fn draw_form(f: &mut Frame, app: &App, area: Rect) {
    let Mode::Fill { fields, focus } = &app.mode else {
        return;
    };
    let mut lines: Vec<Line> = vec![Line::styled(
        "Fill the open blanks",
        Style::new().add_modifier(Modifier::BOLD),
    )];
    for (i, field) in fields.iter().enumerate() {
        lines.push(Line::default());
        let marker = if i == *focus { "› " } else { "  " };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::new().fg(ACCENT)),
            Span::styled(
                field.name.clone(),
                Style::new().add_modifier(Modifier::BOLD),
            ),
        ]));
        let hint = field
            .spec
            .hint
            .clone()
            .or_else(|| field.reason.clone())
            .unwrap_or_default();
        if !hint.is_empty() {
            lines.push(Line::styled(format!("  {hint}"), Style::new().fg(MUTED)));
        }
        let value = if field.spec.choices.is_empty() {
            format!("  {}{}", field.value, if i == *focus { "▏" } else { "" })
        } else {
            format!("  ‹ {} ›", field.value)
        };
        lines.push(Line::styled(value, Style::new().fg(ACCENT)));
    }
    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, area.inner(ratatui::layout::Margin::new(1, 0)));
}
