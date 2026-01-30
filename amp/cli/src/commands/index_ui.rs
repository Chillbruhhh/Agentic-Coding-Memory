use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Terminal,
};
use figlet_rs::FIGfont;
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Debug, Default, Clone)]
pub struct IndexUiState {
    pub phase: String,
    pub total_files: usize,
    pub supported_files: usize,
    pub processed_files: usize,
    pub created_symbols: usize,
    pub created_directories: usize,
    pub errors: usize,
    pub warnings: usize,
    pub current_path: String,
    pub status_message: String,
    pub done: bool,
}

pub struct IndexUiHandle {
    stop_flag: Arc<AtomicBool>,
    join_handle: std::thread::JoinHandle<Result<()>>,
}

impl IndexUiHandle {
    pub fn stop(self) -> Result<()> {
        self.stop_flag.store(true, Ordering::Relaxed);
        match self.join_handle.join() {
            Ok(result) => result,
            Err(_) => Ok(()),
        }
    }

    pub fn wait_for_exit(self) -> Result<()> {
        match self.join_handle.join() {
            Ok(result) => result,
            Err(_) => Ok(()),
        }
    }
}

pub fn start_index_ui(
    state: Arc<Mutex<IndexUiState>>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<IndexUiHandle> {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_clone = Arc::clone(&stop_flag);
    let cancel_clone = Arc::clone(&cancel_flag);

    let join_handle = std::thread::spawn(move || {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut spinner_index = 0usize;
        let spinner_chars = ['|', '/', '-', '\\'];
        let mut last_tick = Instant::now();
        let start_time = Instant::now();
        let fig_font = FIGfont::from_content(include_str!("../../assets/fonts/Small.flf")).ok();
        let amp_fig = fig_font
            .as_ref()
            .and_then(|font| font.convert("AMP"))
            .map(|figure| figure.to_string());
        let phrase_fig = fig_font
            .as_ref()
            .and_then(|font| font.convert("Agentic Memory"))
            .map(|figure| figure.to_string());
        let protocol_fig = fig_font
            .as_ref()
            .and_then(|font| font.convert("Protocol"))
            .map(|figure| figure.to_string());

        loop {
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }

            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let is_ctrl_c = key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL);
                        if is_ctrl_c {
                            cancel_clone.store(true, Ordering::Relaxed);
                            break;
                        }
                        let done = {
                            let guard = state.lock().unwrap();
                            guard.done
                        };
                        if done {
                            break;
                        }
                    }
                }
            }

            if last_tick.elapsed() >= Duration::from_millis(80) {
                spinner_index = (spinner_index + 1) % spinner_chars.len();
                last_tick = Instant::now();
            }

            let snapshot = {
                let guard = state.lock().unwrap();
                guard.clone()
            };

            terminal.draw(|f| {
                let elapsed = start_time.elapsed().as_secs_f64();

                // Animation phases (looping with smooth transitions):
                // Each phase is 10 seconds, with decode in the last 2 seconds
                // and a 1-second blend period at the start of each new phase
                let cycle = 20.0;
                let phase_duration = 10.0;
                let decode_window = 2.0;
                let blend_window = 1.0; // Smooth blend after decode completes

                let phase_time = elapsed % cycle;
                let in_amp = phase_time < phase_duration;
                let phase_pos = if in_amp { phase_time } else { phase_time - phase_duration };

                // Calculate decode progress (0.0 = not decoding, 0.0-1.0 = decoding)
                let decode_progress = if phase_pos >= phase_duration - decode_window {
                    (phase_pos - (phase_duration - decode_window)) / decode_window
                } else {
                    0.0
                };

                // Calculate blend factor for smooth transition from decode to live
                // This smoothly fades from decode-style to live-style after transition
                let blend_factor = if phase_pos < blend_window && phase_pos > 0.0 {
                    phase_pos / blend_window // 0.0 -> 1.0 over blend window
                } else {
                    1.0 // Fully in live mode
                };

                let header_lines = build_header_lines(
                    in_amp,
                    decode_progress,
                    blend_factor,
                    elapsed,
                    spinner_chars[spinner_index],
                    amp_fig.as_deref(),
                    phrase_fig.as_deref(),
                    protocol_fig.as_deref(),
                );
                let header_height = header_lines.len().min((f.size().height.saturating_sub(6)) as usize) as u16 + 1;
                let outer = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(header_height),
                        Constraint::Min(8),
                        Constraint::Length(3),
                    ])
                    .split(f.size());

                let header = Paragraph::new(header_lines);
                f.render_widget(header, outer[0]);

                let body = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                    .split(outer[1]);

                let left = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(body[0]);

                let progress_ratio = if snapshot.supported_files > 0 {
                    snapshot.processed_files as f64 / snapshot.supported_files as f64
                } else if snapshot.total_files > 0 {
                    snapshot.supported_files as f64 / snapshot.total_files as f64
                } else {
                    0.0
                };

                let progress = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Progress"))
                    .gauge_style(Style::default().fg(Color::Red))
                    .ratio(progress_ratio);
                f.render_widget(progress, left[0]);

                let details_text = vec![
                    Line::from(format!("Phase: {}", snapshot.phase)),
                    Line::from(format!(
                        "Processed: {}/{} files",
                        snapshot.processed_files, snapshot.supported_files
                    )),
                    Line::from(format!(
                        "Directories: {}",
                        snapshot.created_directories
                    )),
                    Line::from(format!("Symbols: {}", snapshot.created_symbols)),
                    Line::from(format!("Errors: {}", snapshot.errors)),
                    Line::from(format!("Warnings: {}", snapshot.warnings)),
                    Line::from(""),
                    Line::from("Current file:"),
                    Line::from(snapshot.current_path),
                ];

                let details = Paragraph::new(details_text)
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .wrap(Wrap { trim: true });
                f.render_widget(details, left[1]);

                let stats_text = vec![
                    Line::from(format!("Total files: {}", snapshot.total_files)),
                    Line::from(format!("Supported: {}", snapshot.supported_files)),
                    Line::from(""),
                    Line::from("Tip: press Ctrl+C to abort"),
                ];
                let stats = Paragraph::new(stats_text)
                    .block(Block::default().borders(Borders::ALL).title("Stats"))
                    .wrap(Wrap { trim: true });
                f.render_widget(stats, body[1]);

                let status_line = if snapshot.done {
                    Line::from(Span::styled(
                        "Done! Press any key to exit...",
                        Style::default().fg(Color::Red),
                    ))
                } else {
                    Line::from(Span::raw(format!(
                        "[{}] {}",
                        spinner_chars[spinner_index], snapshot.status_message
                    )))
                };
                let status = Paragraph::new(status_line)
                    .block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(status, outer[2]);
            })?;

            std::thread::sleep(Duration::from_millis(16));
        }

        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    });

    Ok(IndexUiHandle {
        stop_flag,
        join_handle,
    })
}

fn build_header_lines(
    show_amp: bool,
    decode_progress: f64,
    blend_factor: f64,
    elapsed: f64,
    _spinner: char,
    amp_fig: Option<&str>,
    phrase_fig: Option<&str>,
    protocol_fig: Option<&str>,
) -> Vec<Line<'static>> {
    let amp_block = render_figlet_block(amp_fig).unwrap_or_else(|| render_text_block("AMP"));
    let phrase_block = render_phrase_figlet_block(phrase_fig, protocol_fig)
        .unwrap_or_else(render_phrase_block);
    let height = amp_block.len().max(phrase_block.len());
    let max_width = amp_block
        .iter()
        .map(|line| line.len())
        .chain(phrase_block.iter().map(|line| line.len()))
        .max()
        .unwrap_or(amp_block[0].len());
    let amp_block = pad_block(amp_block, height, max_width);
    let phrase_block = pad_block(phrase_block, height, max_width);
    let amp_outline: Vec<String> = amp_block.iter().map(|line| outline_line(line)).collect();
    let phrase_outline: Vec<String> = phrase_block.iter().map(|line| outline_line(line)).collect();
    let _amp_solid: Vec<String> = amp_block.iter().map(|line| solidify_line(line)).collect();
    let _phrase_solid: Vec<String> = phrase_block.iter().map(|line| solidify_line(line)).collect();

    let mut lines = Vec::new();
    if show_amp && decode_progress > 0.0 {
        for (idx, _line) in amp_outline.iter().enumerate() {
            let target = phrase_outline.get(idx).cloned().unwrap_or_default();
            lines.push(render_decode_transition_line(
                &amp_block[idx],
                &target,
                &phrase_block[idx],
                decode_progress,
                elapsed,
                idx,
                height,
            ));
        }
    } else if !show_amp && decode_progress > 0.0 {
        for (idx, _line) in phrase_outline.iter().enumerate() {
            let target = amp_outline.get(idx).cloned().unwrap_or_default();
            lines.push(render_decode_transition_line(
                &phrase_block[idx],
                &target,
                &amp_block[idx],
                decode_progress,
                elapsed,
                idx,
                height,
            ));
        }
    } else if show_amp {
        for (idx, _line) in amp_outline.iter().enumerate() {
            lines.push(render_live_line(
                &amp_block[idx],
                elapsed,
                idx,
                height,
                blend_factor,
            ));
        }
    } else {
        for (idx, _line) in phrase_outline.iter().enumerate() {
            lines.push(render_live_line(
                &phrase_block[idx],
                elapsed,
                idx,
                height,
                blend_factor,
            ));
        }
    }

    lines
}

fn pad_line(line: &str, width: usize) -> String {
    let mut out = String::with_capacity(width);
    out.push_str(line);
    let remaining = width.saturating_sub(line.len());
    out.push_str(&" ".repeat(remaining));
    out
}

#[allow(dead_code)]
fn center_line(line: &str, width: usize) -> String {
    if line.len() >= width {
        return line.to_string();
    }
    let left = (width - line.len()) / 2;
    let right = width - line.len() - left;
    format!("{}{}{}", " ".repeat(left), line, " ".repeat(right))
}

fn pad_block(mut block: Vec<String>, height: usize, width: usize) -> Vec<String> {
    while block.len() < height {
        block.push(pad_line("", width));
    }
    block
}

fn render_live_line(
    line: &str,
    elapsed: f64,
    row: usize,
    height: usize,
    blend_factor: f64,
) -> Line<'static> {
    let mut spans = Vec::with_capacity(line.len());
    let waterfall = waterfall_intensity(elapsed, row, height);

    for (idx, ch) in line.chars().enumerate() {
        if ch == ' ' {
            spans.push(Span::raw(" "));
            continue;
        }
        // Wave intensity (used during decode transition)
        let wave = wave_intensity(elapsed, idx, row as f64, 1.7);
        // Live intensity (waterfall + twinkle)
        let twinkle = ((elapsed * 6.0 + idx as f64 * 0.4 + row as f64).sin() * 0.5 + 0.5)
            .clamp(0.0, 1.0);
        let live = (0.35 + 0.5 * waterfall + 0.15 * twinkle).clamp(0.0, 1.0);
        // Smoothly blend from wave-style to live-style based on blend_factor
        let intensity = wave * (1.0 - blend_factor) + live * blend_factor;
        let style = glow_style(intensity);
        let shine = shine_active(elapsed, idx, row, height, line.len());
        let glyph = ch;
        let final_style = if shine {
            style.add_modifier(Modifier::BOLD).fg(Color::White)
        } else {
            style
        };
        spans.push(Span::styled(glyph.to_string(), final_style));
    }
    Line::from(spans)
}

fn render_decode_transition_line(
    from_text: &str,
    _to_outline: &str,
    to_text: &str,
    progress: f64,
    elapsed: f64,
    row: usize,
    height: usize,
) -> Line<'static> {
    let width = from_text.len().max(to_text.len());
    let reveal = (progress.clamp(0.0, 1.0) * (width as f64 / 2.0 + 1.0)) as usize;
    let mut spans = Vec::with_capacity(width);
    let from_text_chars: Vec<char> = from_text.chars().collect();
    let to_text_chars: Vec<char> = to_text.chars().collect();
    let center = width / 2;

    for idx in 0..width {
        let distance = idx.abs_diff(center);
        let show_to = distance <= reveal;
        let from_ch = *from_text_chars.get(idx).unwrap_or(&' ');
        let to_ch = *to_text_chars.get(idx).unwrap_or(&' ');
        let intensity = wave_intensity(elapsed, idx, row as f64, 1.7);
        let shine = shine_active(elapsed + progress, idx, row, height, width);
        let glyph = if show_to {
            to_ch
        } else if from_ch == ' ' && to_ch == ' ' {
            ' '
        } else {
            decode_glyph(elapsed + progress, idx)
        };
        let style = glow_style(intensity);
        let final_style = if shine { style.add_modifier(Modifier::BOLD).fg(Color::White) } else { style };
        spans.push(Span::styled(glyph.to_string(), final_style));
    }

    Line::from(spans)
}

fn wave_intensity(elapsed: f64, idx: usize, offset: f64, speed: f64) -> f64 {
    let phase = elapsed * speed + idx as f64 * 0.18 + offset * 0.5;
    (phase.sin() * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn waterfall_intensity(elapsed: f64, row: usize, height: usize) -> f64 {
    if height == 0 {
        return 0.0;
    }
    let position = (elapsed * 0.35 + row as f64 / height as f64) % 1.0;
    let band = 1.0 - (position - 0.5).abs() * 2.0;
    band.clamp(0.0, 1.0)
}

fn shine_active(elapsed: f64, idx: usize, row: usize, height: usize, width: usize) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    let tilt = 2.2;
    let total = width as f64 + (height.saturating_sub(1) as f64 * tilt);
    let sweep = (elapsed * 16.0) % total;
    let position = idx as f64 + row as f64 * tilt;
    let distance = (position - sweep).abs();
    distance < 1.4
}

fn glow_style(intensity: f64) -> Style {
    let base = (90.0, 0.0, 0.0);
    let glow = if intensity > 0.94 {
        (255.0, 120.0, 120.0)
    } else {
        (255.0, 50.0, 50.0)
    };
    let mix = intensity.powf(1.6);
    let r = base.0 + (glow.0 - base.0) * mix;
    let g = base.1 + (glow.1 - base.1) * mix;
    let b = base.2 + (glow.2 - base.2) * mix;
    let mut style = Style::default().fg(Color::Rgb(r as u8, g as u8, b as u8));
    if intensity > 0.78 {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

fn decode_glyph(elapsed: f64, idx: usize) -> char {
    let glyphs = ['.', ':', '*', '+', '#', '%', '@', '/', '\\'];
    let seed = ((elapsed * 12.0) as usize).wrapping_add(idx * 7);
    glyphs[seed % glyphs.len()]
}

fn outline_line(line: &str) -> String {
    line.chars()
        .map(|ch| if ch == ' ' { ' ' } else { '.' })
        .collect()
}

fn solidify_line(line: &str) -> String {
    line.chars()
        .map(|ch| if ch == ' ' { ' ' } else { '█' })
        .collect()
}

#[allow(dead_code)]
fn gradient_chars() -> Vec<char> {
    vec!['.', ':', '-', '=', '+', '*', '#', '%', '@', '█']
}

fn render_text_block(text: &str) -> Vec<String> {
    let font = font_map();
    let mut lines = vec![String::new(); 7];
    let chars: Vec<char> = text.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        let glyph = glyph_for_char(*ch, &font);
        for (row, line) in glyph.iter().enumerate() {
            lines[row].push_str(line);
            if i + 1 < chars.len() {
                lines[row].push(' ');
            }
        }
    }
    lines
}

fn blank_glyph() -> [&'static str; 7] {
    ["     ", "     ", "     ", "     ", "     ", "     ", "     "]
}

fn glyph_for_char(
    ch: char,
    font: &std::collections::HashMap<char, [&'static str; 7]>,
) -> [&'static str; 7] {
    if ch == ' ' {
        return blank_glyph();
    }
    let upper = ch.to_ascii_uppercase();
    let glyph = font.get(&upper).cloned().unwrap_or_else(blank_glyph);
    if ch.is_ascii_lowercase() {
        let mut lowered = glyph;
        lowered[0] = "     ";
        lowered
    } else {
        glyph
    }
}

fn render_phrase_block() -> Vec<String> {
    let top = render_text_block("Agentic Memory");
    let bottom = render_text_block("Protocol");
    let mut lines = Vec::new();
    lines.extend(top);
    lines.push(String::new());
    lines.extend(bottom);
    lines
}

fn render_figlet_block(figlet_text: Option<&str>) -> Option<Vec<String>> {
    let figlet_text = figlet_text?;
    let lines: Vec<String> = figlet_text
        .lines()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

fn render_phrase_figlet_block(
    phrase_fig: Option<&str>,
    protocol_fig: Option<&str>,
) -> Option<Vec<String>> {
    let top = render_figlet_block(phrase_fig)?;
    let bottom = render_figlet_block(protocol_fig)?;
    let mut lines = Vec::new();
    lines.extend(top);
    lines.push(String::new());
    lines.extend(bottom);
    Some(lines)
}

fn font_map() -> std::collections::HashMap<char, [&'static str; 7]> {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert('A', [" ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #"]);
    map.insert('C', [" ### ", "#   #", "#    ", "#    ", "#    ", "#   #", " ### "]);
    map.insert('D', ["#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### "]);
    map.insert('E', ["#####", "#    ", "#    ", "#####", "#    ", "#    ", "#####"]);
    map.insert('G', [" ### ", "#   #", "#    ", "# ###", "#   #", "#   #", " ### "]);
    map.insert('I', ["#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "#####"]);
    map.insert('L', ["#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####"]);
    map.insert('M', ["#   #", "## ##", "# # #", "#   #", "#   #", "#   #", "#   #"]);
    map.insert('N', ["#   #", "##  #", "# # #", "#  ##", "#   #", "#   #", "#   #"]);
    map.insert('O', [" ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### "]);
    map.insert('P', ["#### ", "#   #", "#   #", "#### ", "#    ", "#    ", "#    "]);
    map.insert('R', ["#### ", "#   #", "#   #", "#### ", "# #  ", "#  # ", "#   #"]);
    map.insert('T', ["#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  "]);
    map.insert('Y', ["#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  "]);
    map.insert(' ', blank_glyph());
    map
}
