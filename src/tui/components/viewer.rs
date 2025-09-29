// Colorized markdown viewer
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use html_escape::decode_html_entities;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Default)]
pub struct Viewer {
    pub content: String,
    pub title: String,
    pub scroll: usize,
    wrapped_lines: Vec<Line<'static>>, // parsed and wrapped lines for current width
    last_known_width: u16,
}

impl Viewer {
    pub fn new(content: String, title: String) -> Self {
        Self {
            content,
            title,
            scroll: 0,
            wrapped_lines: Vec::new(),
            last_known_width: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, area_height: u16) {
        let area_height = area_height as usize;
        let lines = self.wrapped_lines.len();
        let mut page_size = area_height.saturating_sub(2);
        if page_size == 0 {
            page_size = 1;
        }

        match key.code {
            KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            KeyCode::Down => {
                if self.scroll < lines.saturating_sub(page_size) {
                    self.scroll += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            KeyCode::Char('j') => {
                if self.scroll < lines.saturating_sub(page_size) {
                    self.scroll += 1;
                }
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(page_size);
            }
            KeyCode::PageDown => {
                self.scroll = (self.scroll + page_size).min(lines.saturating_sub(page_size));
            }
            KeyCode::Char('b') => {
                self.scroll = self.scroll.saturating_sub(page_size);
            }
            KeyCode::Char(' ') => {
                self.scroll = (self.scroll + page_size).min(lines.saturating_sub(page_size));
            }
            KeyCode::Home => {
                self.scroll = 0;
            }
            KeyCode::End => {
                self.scroll = lines.saturating_sub(page_size);
            }
            KeyCode::Char('g') => {
                self.scroll = 0;
            }
            KeyCode::Char('G') => {
                self.scroll = lines.saturating_sub(page_size);
            }
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area_height: u16) {
        let area_height = area_height as usize;
        let lines = self.wrapped_lines.len();
        let mut page_size = area_height.saturating_sub(2);
        if page_size == 0 {
            page_size = 1;
        }

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if self.scroll > 0 {
                    self.scroll = self.scroll.saturating_sub(1);
                }
            }
            MouseEventKind::ScrollDown => {
                if self.scroll < lines.saturating_sub(page_size) {
                    self.scroll += 1;
                }
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let view_width = area.width.saturating_sub(2) as usize;

        if area.width != self.last_known_width || self.wrapped_lines.is_empty() {
            let decoded_content = decode_html_entities(&self.content).to_string();
            self.wrapped_lines = parse_markdown_to_lines(&decoded_content, view_width);
            self.last_known_width = area.width;
            // clamp scroll if width change reduced content height
            let visible = area.height.saturating_sub(2) as usize;
            let max_scroll = self.wrapped_lines.len().saturating_sub(visible);
            if self.scroll > max_scroll {
                self.scroll = max_scroll;
            }
        }

        let title = format!(
            "Viewer: {}",
            std::path::Path::new(&self.title)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );

        let total_lines = self.wrapped_lines.len();
        let visible_lines = area.height.saturating_sub(2) as usize;
        let scroll_info = if total_lines > visible_lines {
            format!(
                " (Line {}-{} of {})",
                self.scroll + 1,
                (self.scroll + visible_lines).min(total_lines),
                total_lines
            )
        } else {
            String::new()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("{title}{scroll_info}"));

        // Slice the lines for current viewport
        let slice: Vec<Line> = self
            .wrapped_lines
            .iter()
            .skip(self.scroll)
            .take(visible_lines)
            .cloned()
            .collect();

        let paragraph = Paragraph::new(slice)
            .block(block)
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    #[allow(dead_code)]
    pub fn set_content(&mut self, content: String, file_path: String) {
        self.content = content;
        self.title = file_path;
        self.scroll = 0;
        self.wrapped_lines = Vec::new();
        self.last_known_width = 0;
    }
}

fn parse_markdown_to_lines(src: &str, width: usize) -> Vec<Line<'static>> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(src, opts);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current = String::new();
    let mut mods_stack: Vec<Modifier> = Vec::new();
    // let mut in_code_block = false;
    let mut header_level: Option<u32> = None;

    // Table accumulation state
    let mut in_table = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut in_table_head = false;

    for ev in parser {
        match ev {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    header_level = Some(level as u32);
                    mods_stack.push(Modifier::BOLD);
                }
                Tag::Emphasis => mods_stack.push(Modifier::ITALIC),
                Tag::Strong => mods_stack.push(Modifier::BOLD),
                Tag::Strikethrough => mods_stack.push(Modifier::CROSSED_OUT),
                Tag::CodeBlock(_) => {
                    // in_code_block = true;
                }
                Tag::Item => {
                    // prepend bullet to current buffer
                    current.push_str("\u{2022} ");
                }
                Tag::Link { .. } => {
                    mods_stack.push(Modifier::UNDERLINED);
                }
                Tag::Table(_) => {
                    // Flush any running paragraph
                    if !current.is_empty() {
                        let mut style = style_from_mods(&mods_stack);
                        if mods_stack.contains(&Modifier::BOLD) && header_level.is_some() {
                            style = style.fg(Color::Cyan);
                        }
                        for wrapped in wrap(current.trim_end(), width) {
                            lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
                        }
                        current.clear();
                    }
                    in_table = true;
                    table_headers.clear();
                    table_rows.clear();
                    current_row.clear();
                    in_table_head = false;
                }
                Tag::TableHead => {
                    in_table_head = true;
                }
                Tag::TableRow => {
                    current_row.clear();
                }
                Tag::TableCell => { /* cells handled via Event::Text accumulation */ }
                Tag::Paragraph | Tag::List(_) | Tag::BlockQuote(_) => { /* no-op */ }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    let mut mods = mods_stack.clone();
                    if let Some(level) = header_level
                        && level <= 2
                    {
                        mods.push(Modifier::UNDERLINED);
                    }
                    let style = style_from_mods(&mods).fg(Color::Cyan);
                    if !current.is_empty() {
                        for wrapped in wrap(current.trim_end(), width) {
                            lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
                        }
                        current.clear();
                    }
                    lines.push(Line::from(""));
                    header_level = None;
                    if let Some(pos) = mods_stack.iter().rposition(|m| *m == Modifier::BOLD) {
                        mods_stack.remove(pos);
                    }
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                    // flush current with existing mods (including link blue) before popping?
                    // We keep behavior: just pop style marker
                    mods_stack.pop();
                }
                TagEnd::CodeBlock => {
                    if !current.is_empty() {
                        let style = style_from_mods(&mods_stack);
                        for wrapped in wrap(current.trim_end(), width) {
                            lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
                        }
                        current.clear();
                    }
                    lines.push(Line::from(""));
                }
                TagEnd::Item => {
                    if !current.is_empty() {
                        let style = style_from_mods(&mods_stack);
                        for wrapped in wrap(current.trim_end(), width) {
                            lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
                        }
                        current.clear();
                    }
                }
                TagEnd::TableCell => {
                    if in_table {
                        current_row.push(std::mem::take(&mut current));
                    }
                }
                TagEnd::TableRow => {
                    if in_table {
                        if in_table_head {
                            table_headers = current_row.clone();
                        } else {
                            table_rows.push(current_row.clone());
                        }
                        current_row.clear();
                    }
                }
                TagEnd::TableHead => {
                    in_table_head = false;
                }
                TagEnd::Table => {
                    if in_table {
                        let mut table_lines = render_table(&table_headers, &table_rows, width);
                        lines.append(&mut table_lines);
                        lines.push(Line::from(""));
                        in_table = false;
                    }
                }
                TagEnd::Paragraph | TagEnd::List(_) | TagEnd::BlockQuote(_) => {
                    if !current.is_empty() {
                        let style = style_from_mods(&mods_stack);
                        for wrapped in wrap(current.trim_end(), width) {
                            lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
                        }
                        current.clear();
                    }
                    lines.push(Line::from(""));
                }
                _ => {}
            },
            Event::Text(t) => {
                current.push_str(&t);
            }
            Event::Code(code) => {
                // inline code: yellow + reversed
                if !current.is_empty() {
                    let style = style_from_mods(&mods_stack);
                    for wrapped in wrap(current.trim_end(), width) {
                        lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
                    }
                    current.clear();
                }
                for wrapped in wrap(&code, width) {
                    lines.push(Line::from(Span::styled(
                        wrapped.to_string(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::REVERSED),
                    )));
                }
            }
            Event::SoftBreak => current.push(' '),
            Event::HardBreak => {
                if !current.is_empty() {
                    let style = style_from_mods(&mods_stack);
                    for wrapped in wrap(current.trim_end(), width) {
                        lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
                    }
                    current.clear();
                }
            }
            _ => {}
        }
    }

    if in_table {
        let mut table_lines = render_table(&table_headers, &table_rows, width);
        lines.append(&mut table_lines);
    }

    if !current.is_empty() {
        let style = style_from_mods(&mods_stack);
        for wrapped in wrap(current.trim_end(), width) {
            lines.push(Line::from(Span::styled(wrapped.to_string(), style)));
        }
    }

    lines
}

fn style_from_mods(mods: &[Modifier]) -> Style {
    let mut style = Style::default();
    for &m in mods {
        style = style.add_modifier(m);
    }
    if mods.contains(&Modifier::UNDERLINED) {
        style = style.fg(Color::Blue);
    }
    style
}

fn render_table(headers: &[String], rows: &[Vec<String>], max_width: usize) -> Vec<Line<'static>> {
    // Determine column count
    let cols = headers
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    if cols == 0 {
        return Vec::new();
    }

    // Prepare normalized data
    let norm_headers: Vec<String> = (0..cols)
        .map(|i| headers.get(i).cloned().unwrap_or_default())
        .collect();
    let norm_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            (0..cols)
                .map(|i| r.get(i).cloned().unwrap_or_default())
                .collect()
        })
        .collect();

    // Compute natural widths
    let mut col_widths: Vec<usize> = vec![0; cols];
    for (i, h) in norm_headers.iter().enumerate() {
        col_widths[i] = col_widths[i].max(display_width(h));
    }
    for row in &norm_rows {
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(display_width(cell));
        }
    }

    // Account for borders/padding and shrink if needed
    let content_w: usize = col_widths.iter().sum();
    let paddings = 2 * cols;
    let verticals = cols + 1;
    let total_needed = content_w + paddings + verticals;
    if total_needed > max_width {
        let mut over = total_needed - max_width;
        let mut idxs: Vec<usize> = (0..cols).collect();
        idxs.sort_by_key(|&i| std::cmp::Reverse(col_widths[i]));
        for &i in &idxs {
            if over == 0 {
                break;
            }
            let min_keep = 3;
            if col_widths[i] > min_keep {
                let reducible = col_widths[i] - min_keep;
                let reduce = reducible.min(over);
                col_widths[i] -= reduce;
                over -= reduce;
            }
        }
    }

    // Helper to wrap row into multiple physical lines per the col_widths
    let wrap_row = |cells: &[String]| -> Vec<Vec<String>> {
        let mut wrapped_cols: Vec<Vec<String>> = Vec::with_capacity(cols);
        let mut max_lines = 0;
        for (i, &w) in col_widths.iter().enumerate() {
            let w = w.max(1);
            let cell_text = cells.get(i).cloned().unwrap_or_default();
            let wr = wrap(&cell_text, w);
            let segs: Vec<String> = wr.into_iter().map(|s| s.to_string()).collect();
            max_lines = max_lines.max(segs.len().max(1));
            wrapped_cols.push(segs);
        }
        let mut out: Vec<Vec<String>> = Vec::with_capacity(max_lines);
        for line_idx in 0..max_lines {
            let mut row_line: Vec<String> = Vec::with_capacity(cols);
            for wrapped_col in wrapped_cols.iter().take(cols) {
                let seg = wrapped_col.get(line_idx).cloned().unwrap_or_default();
                row_line.push(seg);
            }
            out.push(row_line);
        }
        out
    };

    // Render borders
    let mut out: Vec<Line<'static>> = Vec::new();
    let top = draw_border('┌', '┬', '┐', '─', &col_widths);
    let sep = draw_border('├', '┼', '┤', '─', &col_widths);
    let bottom = draw_border('└', '┴', '┘', '─', &col_widths);

    // borders gray
    let gray = Style::default().fg(Color::Gray);
    out.push(Line::from(Span::styled(top, gray)));

    // Header (centered + bold)
    if cols > 0 {
        for phys in wrap_row(&norm_headers) {
            out.push(render_row_styled(&phys, &col_widths, true));
        }
        out.push(Line::from(Span::styled(sep.clone(), gray)));
    }

    // Body rows
    for row in &norm_rows {
        for phys in wrap_row(row) {
            out.push(render_row_styled(&phys, &col_widths, false));
        }
        out.push(Line::from(Span::styled(sep.clone(), gray)));
    }

    // Replace last separator with bottom border
    if let Some(last) = out.last_mut() {
        *last = Line::from(Span::styled(bottom, gray));
    }

    out
}

fn draw_border(left: char, mid: char, right: char, horiz: char, col_widths: &[usize]) -> String {
    let mut s = String::new();
    s.push(left);
    for (i, w) in col_widths.iter().enumerate() {
        s.push_str(&horiz.to_string().repeat(w + 2)); // include padding
        if i + 1 < col_widths.len() {
            s.push(mid);
        }
    }
    s.push(right);
    s
}

fn render_row_styled(cells: &[String], col_widths: &[usize], header: bool) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    // left border in gray
    spans.push(Span::styled("│", Style::default().fg(Color::Gray)));
    for (i, cell) in cells.iter().enumerate() {
        let w = col_widths[i];
        let content = if header {
            center_text(cell, w)
        } else {
            pad_right(cell, w)
        };
        let mut styled = Span::raw(format!(" {content} "));
        if header {
            styled = Span::styled(
                format!(" {content} "),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            );
        }
        spans.push(styled);
        // sep border between cols
        spans.push(Span::styled("│", Style::default().fg(Color::Gray)));
    }
    Line::from(spans)
}

fn center_text(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        return s.to_string();
    }
    let total = width - w;
    let left = total / 2;
    let right = total - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

fn pad_right(s: &str, width: usize) -> String {
    let w = display_width(s);
    if w >= width {
        return s.to_string();
    }
    format!("{}{}", s, " ".repeat(width - w))
}

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}
