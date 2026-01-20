use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Sparkline, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Telemetry events sent from the networking core to the UI.
pub enum TelemetryUpdate {
    Throughput { tx_bytes: u64, rx_bytes: u64 },
    Log(String),
}
use rand::Rng; // Import Rng for mock metrics
struct TelemetryState {
    tx_history: Vec<u64>,
    rx_history: Vec<u64>,
    logs: Vec<String>,
    total_tx: u64,
    total_rx: u64,
    // Quality Metrics
    jitter_ms: f64,
    loss_rate: f64,
}

impl TelemetryState {
    fn new() -> Self {
        Self {
            tx_history: vec![0; 100],
            rx_history: vec![0; 100],
            logs: vec![],
            total_tx: 0,
            total_rx: 0,
            jitter_ms: 12.5,
            loss_rate: 0.01,
        }
    }

    fn on_tick(&mut self) {
        // Shift history window
        self.tx_history.remove(0);
        self.tx_history.push(0);
        self.rx_history.remove(0);
        self.rx_history.push(0);
    }
}

pub fn spawn_dashboard(rx: mpsc::Receiver<TelemetryUpdate>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // TUI boilerplate setup
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut app = TelemetryState::new();
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            // Draw UI
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),      // Status Bar
                        Constraint::Percentage(40), // Traffic Graphs
                        Constraint::Percentage(50), // System Logs
                    ].as_ref())
                    .split(f.size());

                // 1. Status Bar
                let header = Paragraph::new(format!(
                    "GHOST_TUNNEL | UPTIME: {:?} | TX: {} | RX: {}", 
                    Duration::from_secs(0), // TODO: Track actual uptime
                    format_bytes(app.total_tx),
                    format_bytes(app.total_rx)
                ))
                .block(Block::default().borders(Borders::ALL).title("CONTROL FRAME"));
                f.render_widget(header, chunks[0]);

                // 2. Traffic Graphs
                let graph_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[1]);

                let tx_spark = Sparkline::default()
                    .block(Block::default().title("Uplink (TX)").borders(Borders::ALL))
                    .data(&app.tx_history)
                    .style(Style::default().fg(Color::LightGreen)); // "Hacker" Green
                f.render_widget(tx_spark, graph_chunks[0]);

                let rx_spark = Sparkline::default()
                    .block(Block::default().title("Downlink (RX)").borders(Borders::ALL))
                    .data(&app.rx_history)
                    .style(Style::default().fg(Color::LightCyan)); // Sci-fi Cyan
                f.render_widget(rx_spark, graph_chunks[1]);

                // 3. Logs
                let log_items: Vec<ListItem> = app.logs.iter()
                    .rev()
                    .take(20)
                    .map(|l| ListItem::new(l.as_str()))
                    .collect();
                let log_list = List::new(log_items)
                    .block(Block::default().title("KERNEL EVENTS").borders(Borders::ALL));
                f.render_widget(log_list, chunks[2]);

            }).unwrap();

            // Input Handling
            if crossterm::event::poll(Duration::from_millis(0)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }

            // Data Ingestion
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    TelemetryUpdate::Throughput { tx_bytes, rx_bytes } => {
                        app.total_tx += tx_bytes;
                        app.total_rx += rx_bytes;
                        
                        // Update current tick bucket
                        let last_idx = app.tx_history.len() - 1;
                        app.tx_history[last_idx] += tx_bytes;
                        app.rx_history[last_idx] += rx_bytes;
                    }
                    TelemetryUpdate::Log(msg) => {
                        let timestamp = chrono::Local::now().format("%H:%M:%S");
                        app.logs.push(format!("[{}] {}", timestamp, msg));
                    }
                }
            }

            // Tick
            if last_tick.elapsed() >= tick_rate {
                app.on_tick();
                last_tick = Instant::now();
            }
        }

        // Cleanup
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();
        terminal.show_cursor().unwrap();
    })
}

// Simple helper for human-readable bytes
fn format_bytes(b: u64) -> String {
    if b < 1024 {
        format!("{} B", b)
    } else if b < 1024 * 1024 {
        format!("{:.1} KB", b as f64 / 1024.0)
    } else {
        format!("{:.2} MB", b as f64 / 1024.0 / 1024.0)
    }
}
