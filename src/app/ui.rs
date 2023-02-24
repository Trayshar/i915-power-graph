use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, List, ListItem, Paragraph},
    Frame,
};

use super::data::RenderData;

pub fn draw<B>(rect: &mut Frame<B>, data: &RenderData)
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

    // TODO: Use default terminal colors if env is set

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Ratio(3, 4),
                Constraint::Ratio(1, 4),
            ]
            .as_ref(),
        )
        .split(size);

    // Title block
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    let chart_area = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)].as_ref())
    .split(chunks[1]);
    let power_graph_width = chart_area[0].width as usize + 10;
    let power_data = data.get_power_data(power_graph_width);
    let dataset = Dataset::default()
        .name("power")
        .marker(tui::symbols::Marker::Block)
        .style(Style::default().fg(Color::Cyan))
        .data(&power_data);

    
    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Time (s)")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("{}", power_graph_width),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ])
                .bounds([0.0, power_graph_width as f64]),
        )
        .y_axis(
            Axis::default()
                .title("Power Consumption (W)")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("50"),
                    Span::raw("100"),
                    Span::raw("150"),
                    Span::styled("200", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 200.0]),
        );
    rect.render_widget(chart, chart_area[0]);

    let current_power = if let Some(p) = power_data.last() { p.1 } else { 0.0 };
    let data_list = List::new(vec![
        ListItem::new("TODO: Graphics card name").style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
        ListItem::new(format!(
            "Power draw: {:.2}W",
            current_power
        )),
        ListItem::new(format!("Total Energy used: {:.2}kWh", data.total_energy)),
        ListItem::new("TODO: Uptime"),
    ])
    .block(Block::default().borders(Borders::ALL));
    rect.render_widget(data_list, chart_area[1]);

    let log_list =
        List::new(data.get_log()).block(Block::default().borders(Borders::ALL).title("Log"));
    rect.render_widget(log_list, chunks[2]);
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
