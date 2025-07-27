use crate::tui::app::{App, AppState, FileFilter};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    match &app.state {
        AppState::Home => draw_home(f, app),
        AppState::NewTranscript => draw_new_transcript(f, app),
        AppState::Processing { video_id, .. } => draw_processing(f, app, video_id),
        AppState::Browser { .. } => draw_browser(f, app),
        AppState::Viewer { .. } => draw_viewer(f, app),
        AppState::Settings => draw_settings(f, app),
    }
}

fn draw_home(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Menu
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("YTranscript TUI")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Menu options
    let options = [
        "● Nueva Transcripción",
        "○ Ver Transcripciones",
        "○ Ver Reportes",
        "○ Configuración",
    ];

    let menu_items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let style = if i == app.selected_option {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let text = if i == app.selected_option {
                option.replace("○", "●")
            } else {
                option.replace("●", "○")
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let menu = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Modo"))
        .style(Style::default().fg(Color::White));
    f.render_widget(menu, chunks[1]);

    // Help
    let help = Paragraph::new("[↑↓] Navegar  [Enter] Seleccionar  [q] Salir")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_new_transcript(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // URL input
            Constraint::Length(3), // Languages input
            Constraint::Length(5), // Checkboxes
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Nueva Transcripción")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // URL input
    app.url_input.render(f, chunks[1]);

    // Languages input
    app.languages_input.render(f, chunks[2]);

    // Checkboxes
    let checkbox_block = Block::default().borders(Borders::ALL).title("Opciones");
    f.render_widget(checkbox_block, chunks[3]);

    let checkbox_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[3]);

    let preserve_style = if app.input_focus == 2 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let report_style = if app.input_focus == 3 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let preserve_checkbox = if app.preserve_formatting {
        "☑"
    } else {
        "☐"
    };
    let report_checkbox = if app.generate_report { "☑" } else { "☐" };

    let preserve_text =
        Paragraph::new(format!("{preserve_checkbox} Preservar formato")).style(preserve_style);
    f.render_widget(preserve_text, checkbox_area[0]);

    let report_text = Paragraph::new(format!("{report_checkbox} Generar reporte automáticamente"))
        .style(report_style);
    f.render_widget(report_text, checkbox_area[1]);

    // Help
    let help = Paragraph::new("[Enter] Procesar  [Esc] Volver  [Tab] Siguiente  [Space] Toggle")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[4]);
}

fn draw_processing(f: &mut Frame, app: &App, video_id: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Progress area
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Procesando...")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Progress area
    app.progress_bar.render(f, chunks[1], video_id);

    // Help
    let help = Paragraph::new("[Esc] Cancelar")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_browser(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(1)])
        .split(f.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Filters
            Constraint::Length(3), // Search
        ])
        .split(chunks[0]);

    // Filter panel
    let filter_options = ["● Todos", "○ Transcripciones", "○ Reportes"];
    let filter_items: Vec<ListItem> = filter_options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let is_selected = matches!(
                (&app.filter, i),
                (FileFilter::All, 0) | (FileFilter::Transcripts, 1) | (FileFilter::Reports, 2)
            );

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let text = if is_selected {
                option.replace("○", "●")
            } else {
                option.replace("●", "○")
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let filters =
        List::new(filter_items).block(Block::default().borders(Borders::ALL).title("Filtros"));
    f.render_widget(filters, left_chunks[0]);

    // Search
    app.search_input.render(f, left_chunks[1]);

    // File list
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(chunks[1]);

    app.file_list.render(f, right_chunks[0], "Archivos");

    // Help
    let help = Paragraph::new(
        "[Enter] Abrir  [Del] Eliminar  [Space] Seleccionar  [/] Buscar  [1-3] Filtros",
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, right_chunks[1]);
}

fn draw_viewer(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.area());

    // Content viewer
    if let Some(viewer) = &app.content_viewer {
        viewer.render(f, chunks[0]);
    }

    // Help
    let help =
        Paragraph::new("[↑↓] Scroll  [PgUp/PgDn] Página  [Home/End] Inicio/Fin  [Esc] Volver")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[1]);
}

fn draw_settings(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Settings content
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Configuración")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Settings content (placeholder)
    let settings_content = Paragraph::new("Configuraciones próximamente...")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(settings_content, chunks[1]);

    // Help
    let help = Paragraph::new("[Esc] Volver")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}
