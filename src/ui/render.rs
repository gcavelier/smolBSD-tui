use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType::Rounded, Borders, Clear, Padding, Paragraph, Row, Table, Wrap},
};
use ratatui_image::StatefulImage;

use crate::{
    app::{State, VERSION},
    ui::{
        ACTION_COLOR, DEFAULT_SPACING_PADDING, INFO_COLOR, POPUP_BORDER_COLOR,
        SELECTED_BUTTON_BG_COLOR, SELECTED_BUTTON_FG_COLOR, Screen, UNSELECTED_BUTTON_BG_COLOR,
        UNSELECTED_BUTTON_FG_COLOR,
    },
};

pub fn render(frame: &mut Frame, app: &mut State) {
    let screen = app.current_screen.clone();

    let [header_chunk, vms_list_chunk] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(frame.area());

    match screen {
        Screen::List => {
            render_header(frame, app, header_chunk);
            render_vms_list(frame, app, vms_list_chunk);
        }

        Screen::DeleteConfirmation(ok) => {
            render_header(frame, app, header_chunk);
            render_vms_list(frame, app, vms_list_chunk);
            if let Some(current_vm) = app.vms.get(app.table_state.selected().unwrap()) {
                render_popup(
                    frame,
                    " ⚠️ Delete VM ⚠️ ",
                    Paragraph::new(format!(
                        "Are you sure you want to delete VM '{}'",
                        current_vm.name
                    ))
                    .centered(),
                    Some(ok),
                );
            }
        }

        Screen::StartNbFailed {
            vm_name,
            error,
            stdout,
            stderr,
        } => {
            render_header(frame, app, header_chunk);
            render_vms_list(frame, app, vms_list_chunk);

            let mut lines = vec![
                Line::from(error).centered(),
                Line::from(""),
                Line::from("stdout:").underlined(),
            ];

            lines.extend(stdout.lines().map(Line::from));
            lines.push(Line::from(""));
            lines.push(Line::from("stderr:").underlined());
            lines.extend(stderr.lines().map(Line::from));

            render_popup(
                frame,
                &format!(" ❌ Failed to start VM '{}' ❌ ", vm_name),
                Paragraph::new(lines),
                None,
            );
        }

        Screen::KillFailed { vm_name, error } => {
            render_header(frame, app, header_chunk);
            render_vms_list(frame, app, vms_list_chunk);

            let lines = vec![Line::from(error).centered()];

            render_popup(
                frame,
                &format!(" ❌ Failed to kill VM '{}' ❌ ", vm_name),
                Paragraph::new(lines),
                None,
            );
        }
    }
}

fn render_header(frame: &mut Frame, _app: &mut State, area: Rect) {
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

    // TODO: load logo in tier3 (9x13 ?)
    // Only works with the "official" ratatui crate, not my github fork :(
    //frame.render_stateful_widget(StatefulImage::default(), tier3, &mut _app.logo);
}

fn render_vms_list(frame: &mut Frame, app: &mut State, area: Rect) {
    let rows: Vec<_> = app
        .vms
        .iter()
        .map(|vm| {
            let (state_str, state_color) = vm.state();
            Row::new(vec![vm.name.clone(), state_str]).style(Style::new().fg(state_color))
        })
        .collect();
    let widths = [Constraint::Min(5), Constraint::Max(24)];
    let table = Table::new(rows, widths)
        .column_spacing(DEFAULT_SPACING_PADDING)
        .fg(Color::Indexed(74))
        .header(Row::new(vec!["NAME", "STATE"]).style(Style::new().white()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled(" Configured VMs [", Style::new()),
                    Span::styled(format!("{}", &app.vms.len()), Style::new().fg(Color::White)),
                    Span::styled("] ", Style::new()),
                ]))
                .padding(Padding {
                    left: DEFAULT_SPACING_PADDING,
                    right: DEFAULT_SPACING_PADDING,
                    top: 0,
                    bottom: 0,
                })
                .border_type(Rounded)
                .title_alignment(Alignment::Center),
        )
        .row_highlight_style(Style::new().reversed());
    // VMs list
    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_popup(frame: &mut Frame, title: &str, msg: Paragraph, confirmation: Option<bool>) {
    let area = get_centered_area_fit_to_content(frame, &msg);

    let top_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(Style::new().gray())
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_style(POPUP_BORDER_COLOR)
        .border_type(Rounded);

    frame.render_widget(Clear, area);

    let [top_chunk, bottom_chunk] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(4)]).areas(area);

    frame.render_widget(
        msg.wrap(Wrap { trim: false })
            .block(top_block.padding(Padding {
                left: DEFAULT_SPACING_PADDING,
                right: DEFAULT_SPACING_PADDING,
                top: DEFAULT_SPACING_PADDING,
                bottom: DEFAULT_SPACING_PADDING,
            })),
        top_chunk,
    );

    // Render bottom border
    let bottom_block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_style(POPUP_BORDER_COLOR)
        .border_type(Rounded);
    frame.render_widget(bottom_block, bottom_chunk);

    // Render buttons based on confirmation mode
    match confirmation {
        None => {
            // Simple popup: "OK" button only
            let [_, ok_button_chunk, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Fill(1),
            ])
            .areas(bottom_chunk);
            let [_, ok_button_chunk, _] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(2),
            ])
            .areas(ok_button_chunk);

            frame.render_widget(
                Paragraph::new("OK")
                    .fg(SELECTED_BUTTON_FG_COLOR)
                    .bg(SELECTED_BUTTON_BG_COLOR)
                    .centered(),
                ok_button_chunk,
            );
        }
        Some(ok) => {
            // Confirmation popup: "OK" and "Cancel" buttons
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
        }
    }
}

fn get_centered_area_fit_to_content(frame: &mut Frame, msg: &Paragraph) -> Rect {
    // TODO: handle case where popup_height > max_y and popup_width > max_x
    // => return a ScrollBarState ?

    // Max popup size : 80% of the frame size
    let max_y = (frame.area().height as f64 * 0.8).trunc() as u16;
    let max_x = (frame.area().width as f64 * 0.8).trunc() as u16;

    // Adding DEFAULT_SPACING_PADDING * 4 to account for the spacing/padding, the title and the bottom border
    // Adding 3 to account for the bottom buttons (1 for the buttons, 1 blank line below and 1 blank line above)
    let popup_height = (msg.line_count(max_y) as u16 + DEFAULT_SPACING_PADDING * 4 + 3).min(max_y);
    // Adding 2 * 4 to account for the spacing/padding and the left and right borders
    let popup_width = (msg.line_width() as u16 + 2 * 4).min(max_x);

    let horizontal = Layout::horizontal([popup_width]).flex(Flex::Center);
    let vertical = Layout::vertical([popup_height]).flex(Flex::Center);
    let [area] = vertical.areas(frame.area());
    let [area] = horizontal.areas(area);
    area
}
