use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use crate::logo;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints([
            Constraint::Percentage(35), // left-side: ASCII art
            Constraint::Percentage(65), // right-side: system information
        ])
        .split(size);

    draw_ascii_art(f, main_chunks[0]);
    draw_all_system_info(f, main_chunks[1], app);

    draw_help_simple(f, size);
}

fn draw_ascii_art(f: &mut Frame, area: ratatui::layout::Rect) {
    let ascii_art = logo::get_logo();
    let paragraph = Paragraph::new(ascii_art).alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}

fn draw_all_system_info(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let info = &app.system_info;

    let mut text = vec![
        Line::from(vec![
            Span::styled(
                "  OS: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{} {}, {}",
                info.os_name, info.os_version, info.os_arch
            )),
        ]),
        Line::from(vec![
            Span::styled(
                "  Kernel: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.kernel_version),
        ]),
        Line::from(vec![
            Span::styled(
                "  Host: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.hostname),
        ]),
        Line::from(vec![
            Span::styled(
                "  User: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.username),
        ]),
        Line::from(vec![
            Span::styled(
                "  Uptime: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.uptime),
        ]),
        Line::from(""),
    ];

    for (i, cpu) in info.cpus.iter().enumerate() {
        if i == 0 {
            text.push(Line::from(Span::styled(
                " 🔥 CPUs",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let cpu_name = format!(
            "{} ({} cores) @ {:.2}GHz",
            cpu.model
                .split_whitespace()
                .take(4)
                .collect::<Vec<_>>()
                .join(" "),
            cpu.cores,
            cpu.frequency as f64 / 1000.0
        );

        text.push(Line::from(vec![
            Span::styled(
                format!("  - CPU {}: ", i + 1),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(cpu_name),
        ]));
    }

    for (i, gpu) in info.gpus.iter().enumerate() {
        if i == 0 {
            text.push(Line::from(Span::styled(
                " 🎮 GPUs",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        text.push(Line::from(vec![
            Span::styled(
                format!("  - GPU {}: ", i + 1),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(gpu.name.clone()),
        ]));
    }

    text.push(Line::from(""));

    text.push(Line::from(vec![
        Span::styled(
            " 🌐 Local IP: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&info.local_ip),
    ]));

    text.push(Line::from(vec![
        Span::styled(
            " 🐚 Shell: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&info.shell),
    ]));

    text.push(Line::from(vec![
        Span::styled(
            " 📟 Terminal: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&info.terminal),
    ]));

    let memory_percent = if info.memory_total > 0 {
        (info.memory_used as f64 / info.memory_total as f64 * 100.0) as u16
    } else {
        0
    };
    text.push(Line::from(vec![
        Span::styled(
            " 💾 Memory: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "{}MiB / {}MiB ({}%)",
            info.memory_used / (1024 * 1024),
            info.memory_total / (1024 * 1024),
            memory_percent
        )),
    ]));

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🖥️  Environments ")
                .title_alignment(Alignment::Center)
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn draw_help_simple(f: &mut Frame, size: ratatui::layout::Rect) {
    let help_area = ratatui::layout::Rect {
        x: 0,
        y: size.height.saturating_sub(1),
        width: size.width,
        height: 1,
    };

    let help_text = Paragraph::new("Press 'q' or 'Esc' to quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help_text, help_area);
}
