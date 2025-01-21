use crate::app::{CurrentScreen, State};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn render(frame: &mut Frame, app_state: &mut State) {
    match app_state.current_screen {
        CurrentScreen::List => {
            let layout =
                Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(frame.area());

            // Header
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    " Version: ".fg(Color::Indexed(208)),
                    VERSION.into(),
                ])),
                layout[0],
            );

            let rows: Vec<_> = app_state
                .vms
                .iter()
                .map(|vm| {
                    let (running_str, running_color) = match vm.running {
                        true => ("Yes".to_owned(), Color::Indexed(74)),
                        false => ("No".to_owned(), Color::Indexed(202)),
                    };
                    Row::new(vec![format!(" {}", vm.name), running_str])
                        .style(Style::new().fg(running_color))
                })
                .collect();
            let widths = [Constraint::Min(5), Constraint::Min(5)];
            let table = Table::new(rows, widths)
                .column_spacing(1)
                .fg(Color::Indexed(74))
                .header(Row::new(vec![" NAME", "RUNNING"]).style(Style::new().white()))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(vec![
                            Span::styled(" Configured VMs [", Style::new()),
                            Span::styled(
                                format!("{}", &app_state.vms.len()),
                                Style::new().fg(Color::White),
                            ),
                            Span::styled("] ", Style::new()),
                        ]))
                        .title_alignment(Alignment::Center),
                )
                .row_highlight_style(Style::new().reversed());
            // VMs list
            app_state
                .table_state
                .select(Some(app_state.selected_vm_idx));
            frame.render_stateful_widget(table, layout[1], &mut app_state.table_state);
        }
        CurrentScreen::StartStop => {
            let popup_block = Block::default()
                .title(format!(
                    " Starting '{}' VM ",
                    // TODO: replace w/ app_state.vms.get(app_state.selected_vm_idx)
                    app_state.vms[app_state.selected_vm_idx].name
                ))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL);
            //.style(Style::default().bg(Color::DarkGray));
            let area = centered_rect(60, 25, frame.area());
            frame.render_widget(popup_block, area);
        }
    }
    // TODO : toggable log block ?
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
/// Code from https://ratatui.rs/tutorials/json-editor/ui/
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
