use crate::app::{Screen, StartStopState, State};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Borders, Clear, Padding, Paragraph, Row, ScrollbarState, Table, Wrap},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INFO_COLOR: Color = Color::Indexed(208);
const SELECTED_BUTTON_FG_COLOR: Color = Color::Black;
const SELECTED_BUTTON_BG_COLOR: Color = Color::Gray;
const UNSELECTED_BUTTON_FG_COLOR: Color = Color::Gray;
const UNSELECTED_BUTTON_BG_COLOR: Color = Color::Black;
const POPUP_BORDER_COLOR: Color = Color::Indexed(74);
const RUNNING_VM_FG: Color = Color::Indexed(74);
const NON_RUNNING_VM_FG: Color = Color::Indexed(202);
const ACTION_COLOR: Color = Color::Magenta;
const DEFAULT_SPACING_PADDING: u16 = 1;

pub fn render(frame: &mut Frame, app_state: &mut State) {
    let screen = app_state.current_screen.clone();

    let [header_chunk, vms_list_chunk] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(frame.area());

    match screen {
        Screen::List => {
            render_header(frame, app_state, header_chunk);
            render_vms_list(frame, app_state, vms_list_chunk);
        }
        Screen::StartStop(start_stop_state) => {
            render_header(frame, app_state, header_chunk);
            render_vms_list(frame, app_state, vms_list_chunk);

            match start_stop_state.err_str {
                Some(_) => render_start_stop_error_popup(frame, app_state),
                None => match app_state.start_stop_vm() {
                    Ok(_) => app_state.current_screen = Screen::List, // Everything is fine, going back to the main screen
                    Err(err) => {
                        app_state.current_screen = Screen::StartStop(StartStopState {
                            err_str: Some(err),
                            vertical_scroll_bar_pos: 0,
                            vertical_scroll_bar_state: ScrollbarState::default(),
                        })
                    }
                },
            }
        }
        Screen::DeleteConfirmation(ok) => {
            render_header(frame, app_state, header_chunk);
            render_vms_list(frame, app_state, vms_list_chunk);
            render_delete_confirmation_popup(frame, app_state, ok);
        }
    }
    // TODO : toggable log block ?
}

fn render_header(frame: &mut Frame, _app_state: &mut State, area: Rect) {
    let [tier1, tier2, tier3] = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec!["Version: ".fg(INFO_COLOR), VERSION.into()]))
            .block(Block::new().padding(Padding::left(1))),
        tier1,
    );
    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::from(vec!["<Esc|q>".fg(ACTION_COLOR), " Quit".into()]),
            Line::from(vec!["    <s>".fg(ACTION_COLOR), " Start/Stop".into()]),
            Line::from(vec!["    <d>".fg(ACTION_COLOR), " Delete".into()]),
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
                Some(_) => ("  Yes".to_owned(), RUNNING_VM_FG),
                None => ("   No".to_owned(), NON_RUNNING_VM_FG),
            };
            Row::new(vec![vm.name.clone(), running_str]).style(Style::new().fg(running_color))
        })
        .collect();
    let widths = [Constraint::Min(5), Constraint::Length(10)];
    let table = Table::new(rows, widths)
        .column_spacing(DEFAULT_SPACING_PADDING)
        .fg(RUNNING_VM_FG)
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
                    left: DEFAULT_SPACING_PADDING,
                    right: DEFAULT_SPACING_PADDING,
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

fn render_start_stop_error_popup(frame: &mut Frame, app_state: &mut State) {
    // We first check if we have an error to display
    // If not, we don't do anything
    let (err_str, ref mut start_stop_state) = match app_state.current_screen.clone() {
        Screen::StartStop(start_stop_state) => match &start_stop_state.err_str {
            None => return,
            Some(err) => (err.replace('\t', " "), start_stop_state), // TODO: calculate tab length
        },
        _ => return,
    };

    if let Some(current_vm) = app_state.vms.get(app_state.selected_vm_idx) {
        let title = format!(
            " ❌ Error {} '{}' VM ❌ ",
            match current_vm.pid {
                Some(_) => "stopping", // The VM is running, we want to stop it
                None => "starting",    // The VM is *not* running, we want to start it
            },
            current_vm.name
        );

        let area = get_centered_area_fit_to_content(
            frame,
            err_str.lines().count() as u16 + 4,
            (err_str
                .lines()
                .map(|line| line.len())
                .max()
                .unwrap_or_default() as u16)
                .max(title.len() as u16),
        );

        let top_block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .title_style(Color::White)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(POPUP_BORDER_COLOR);

        frame.render_widget(Clear, area);

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
            .border_style(POPUP_BORDER_COLOR);
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
                .fg(SELECTED_BUTTON_FG_COLOR)
                .bg(SELECTED_BUTTON_BG_COLOR)
                .centered(),
            button_chunk,
        );
    } else {
        // app_state.selected_vm_idx is invalid, so something pretty bad happened!
        // Going back to the main screen
        app_state.current_screen = Screen::List;
    }
}

fn render_delete_confirmation_popup(frame: &mut Frame, app_state: &mut State, ok: bool) {
    if let Some(current_vm) = app_state.vms.get(app_state.selected_vm_idx) {
        let title = " ❗ Delete VM ❗ ";
        let msg = format!("Are you sure you want to delete VM '{}'", current_vm.name);

        let area = get_centered_area_fit_to_content(frame, 1 + 3, msg.len() as u16);

        let top_block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .title_style(Style::new().white())
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(POPUP_BORDER_COLOR);

        frame.render_widget(Clear, area);

        let [top_chunk, bottom_chunk] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(4)]).areas(area);

        frame.render_widget(
            Paragraph::new(msg)
                .fg(Color::White)
                .wrap(Wrap { trim: false })
                .block(top_block.padding(Padding {
                    left: DEFAULT_SPACING_PADDING,
                    right: DEFAULT_SPACING_PADDING,
                    top: DEFAULT_SPACING_PADDING,
                    bottom: DEFAULT_SPACING_PADDING,
                }))
                .centered(),
            top_chunk,
        );

        // Render "OK" and "Cancel" buttons
        let bottom_block = Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(POPUP_BORDER_COLOR);
        frame.render_widget(bottom_block, bottom_chunk);
        let [_, ok_button_chunk, _, cancel_button_chunk, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .areas(bottom_chunk);
        let [_, ok_button_chunk, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(ok_button_chunk);
        let [_, cancel_button_chunk, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(cancel_button_chunk);
        frame.render_widget(
            Paragraph::new("OK")
                .fg(match ok {
                    true => SELECTED_BUTTON_FG_COLOR,
                    false => UNSELECTED_BUTTON_FG_COLOR,
                })
                .bg(match ok {
                    true => SELECTED_BUTTON_BG_COLOR,
                    false => UNSELECTED_BUTTON_BG_COLOR,
                })
                .centered(),
            ok_button_chunk,
        );
        frame.render_widget(
            Paragraph::new("Cancel")
                .fg(match !ok {
                    true => SELECTED_BUTTON_FG_COLOR,
                    false => UNSELECTED_BUTTON_FG_COLOR,
                })
                .bg(match !ok {
                    true => SELECTED_BUTTON_BG_COLOR,
                    false => UNSELECTED_BUTTON_BG_COLOR,
                })
                .centered(),
            cancel_button_chunk,
        );
    } else {
        // app_state.selected_vm_idx is invalid, so something pretty bad happened!
        // Going back to the main screen
        app_state.current_screen = Screen::List;
    }
}

fn get_centered_area_fit_to_content(
    frame: &mut Frame,
    content_height: u16,
    content_width: u16,
) -> Rect {
    // TODO: handle case where popup_height > max_y and popup_width > max_x
    // => return a ScrollBarState ?

    // Max popup size : 80% of the frame size
    let max_y = (frame.area().height as f64 * 0.8).trunc() as u16;
    let max_x = (frame.area().width as f64 * 0.8).trunc() as u16;

    // Adding DEFAULT_SPACING_PADDING * 4 to account for the spacing/padding, the title and the bottom border
    let popup_height = content_height + DEFAULT_SPACING_PADDING * 4;
    // Adding DEFAULT_SPACING_PADDING * 4 to account for the spacing/padding and the left and right borders
    let popup_width = content_width + DEFAULT_SPACING_PADDING * 4;

    let horizontal = Layout::horizontal([popup_width]).flex(Flex::Center);
    let vertical = Layout::vertical([popup_height]).flex(Flex::Center);
    let [area] = vertical.areas(frame.area());
    let [area] = horizontal.areas(area);
    area
}
