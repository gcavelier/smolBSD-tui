use crate::app::{CurrentScreen, State};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph, Row, Table, Wrap},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn render(frame: &mut Frame, app_state: &mut State) {
    let screen = app_state.current_screen.clone();

    let [header_chunk, vms_list_chunk] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(frame.area());

    match screen {
        CurrentScreen::List => {
            render_header(frame, app_state, header_chunk);
            render_vms_list(frame, app_state, vms_list_chunk);
        }
        CurrentScreen::StartStop(start_stop_state) => {
            render_header(frame, app_state, header_chunk);
            render_vms_list(frame, app_state, vms_list_chunk);

            match start_stop_state.err_str {
                Some(_) => render_start_stop_popup(frame, app_state),
                None => app_state.start_stop_vm(),
            }
        }
    }
    // TODO : toggable log block ?
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
/// Code from https://ratatui.rs/tutorials/json-editor/ui/
/// TODO: rewrite using https://docs.rs/ratatui/latest/src/demo2/destroy.rs.html#136
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

fn render_header(frame: &mut Frame, _app_state: &mut State, area: Rect) {
    let [tier1, tier2, tier3] = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            "Version: ".fg(Color::Indexed(208)),
            VERSION.into(),
        ]))
        .block(Block::new().padding(Padding::left(1))),
        tier1,
    );
    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::from(vec!["<Esc|q>".fg(Color::Magenta), " Quit".into()]),
            Line::from(vec!["    <s>".fg(Color::Magenta), " Start/Stop VM".into()]),
        ])),
        tier2,
    );
}

fn render_vms_list(frame: &mut Frame, app_state: &mut State, area: Rect) {
    let rows: Vec<_> = app_state
        .vms
        .iter()
        .map(|vm| {
            let (running_str, running_color) = match vm.pid {
                Some(_) => ("  Yes".to_owned(), Color::Indexed(74)),
                None => ("   No".to_owned(), Color::Indexed(202)),
            };
            Row::new(vec![vm.name.clone(), running_str]).style(Style::new().fg(running_color))
        })
        .collect();
    let widths = [Constraint::Min(5), Constraint::Length(10)];
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .fg(Color::Indexed(74))
        .header(Row::new(vec!["NAME", "RUNNING"]).style(Style::new().white()))
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
                .padding(Padding {
                    left: 1,
                    right: 1,
                    top: 0,
                    bottom: 0,
                })
                .title_alignment(Alignment::Center),
        )
        .row_highlight_style(Style::new().reversed());
    // VMs list
    app_state
        .table_state
        .select(Some(app_state.selected_vm_idx));
    frame.render_stateful_widget(table, area, &mut app_state.table_state);
}

fn render_start_stop_popup(frame: &mut Frame, app_state: &mut State) {
    // We first check if we have an error to display
    // If not, we don't do anything
    let (err_str, ref mut start_stop_state) = match app_state.current_screen.clone() {
        CurrentScreen::List => return,
        CurrentScreen::StartStop(start_stop_state) => match &start_stop_state.err_str {
            None => return,
            Some(err) => (err.replace('\t', " "), start_stop_state), // TODO: calculate tab length
        },
    };

    if let Some(current_vm) = app_state.vms.get(app_state.selected_vm_idx) {
        let top_block = Block::default()
            .title(format!(
                " Error {} '{}' VM ",
                match current_vm.pid {
                    Some(_) => "stopping", // The VM is running, we want to stop it
                    None => "starting",    // The VM is *not* running, we want to start it
                },
                current_vm.name
            ))
            .title_alignment(Alignment::Center)
            .title_style(Style::new().red())
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::new().white());
        let area = centered_rect(60, 60, frame.area());
        let [top_chunk, bottom_chunk] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(4)]).areas(area);

        // Display error in top_chunk
        frame.render_widget(
            Paragraph::new(err_str.as_str())
                .fg(Color::White)
                .wrap(Wrap { trim: false })
                .block(top_block.padding(Padding {
                    left: 1,
                    right: 1,
                    top: 1,
                    bottom: 1,
                }))
                .scroll((start_stop_state.vertical_scroll_bar_pos as u16, 0)),
            top_chunk,
        );
        // Display vertical scroll in top_chunk
        // TODO: uncomment when max scroll position is handled
        // frame.render_stateful_widget(
        //     Scrollbar::new(ScrollbarOrientation::VerticalRight),
        //     top_chunk,
        //     &mut start_stop_state.vertical_scroll_bar_state,
        // );

        let bottom_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::new().white());
        frame.render_widget(bottom_block, bottom_chunk);

        let [_, button_chunk, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Fill(1),
        ])
        .areas(bottom_chunk);
        let [_, button_chunk, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(button_chunk);

        frame.render_widget(
            Paragraph::new("OK")
                .fg(Color::Black)
                .bg(Color::Gray)
                .centered(),
            button_chunk,
        );
    } else {
        // app_state.selected_vm_idx is invalid, so something pretty bad happened!
        // Going back to the main screen
        app_state.current_screen = CurrentScreen::List;
    }
}
