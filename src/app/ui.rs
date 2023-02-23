use tui::{
    backend::{Backend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, BorderType, Borders, Paragraph, List, ListItem, Chart, Dataset, Axis},
    Frame, text::Span
};

use super::PowerGraph;

pub fn draw<B>(rect: &mut Frame<B>, log: &Vec<String>, power_graph: &PowerGraph)
where
    B: Backend,
{
    let size = rect.size();
    if size.width < 20 {
        panic!("Require width >= 20, (got {})", size.width);
    }
    if size.height < 8 {
        panic!("Require height >= 8, (got {})", size.height);
    }

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), 
            Constraint::Percentage(50), 
            Constraint::Length(5)
        ].as_ref())
        .split(size);

    // Title block
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    let dataset = Dataset::default()
        .name("power")
        .marker(tui::symbols::Marker::Dot)
        .style(Style::default().fg(Color::Cyan))
        .data(power_graph);
    let chart = Chart::new(vec![dataset])
    .block(
        Block::default()
            .title(Span::styled(
                "Power Consumption (W)",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL),
    )
    .x_axis(
        Axis::default()
            .title("Time (s)")
            .style(Style::default().fg(Color::Gray))
            .labels(vec![
                Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}", super::GRAPH_DATA_LEN), Style::default().add_modifier(Modifier::BOLD)),
            ])
            .bounds([0.0, 100.0]),
    )
    .y_axis(
        Axis::default()
            .title("Power Consumption (W)")
            .style(Style::default().fg(Color::Gray))
            .labels(vec![
                Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("100"),
                Span::styled("200", Style::default().add_modifier(Modifier::BOLD)),
            ])
            .bounds([0.0, 200.0]),
    );
    rect.render_widget(chart, chunks[1]);

    let list = List::new(log.iter().map(|f| ListItem::new(f.as_str())).collect::<Vec<_>>())
        .block(Block::default()
        .borders(Borders::ALL)
        .title("Log"));
    rect.render_widget(list, chunks[2]);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("i915-power-graph")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}