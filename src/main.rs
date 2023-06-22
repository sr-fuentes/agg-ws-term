mod app;
mod ui;

use std::{collections::BTreeMap, error::Error, fs::File, io, panic, sync::Arc};

use agg_ws::client::Exchange;
use app::App;
use crossterm::{
    event::{Event, EventStream, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use rust_decimal::Decimal;
use tokio::{time, time::Duration};
use ui::ui;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub enum AggExchange {
    Aggregate,
    Exchange(Exchange),
}

impl AggExchange {
    pub fn as_display(&self) -> &'static str {
        match self {
            Self::Aggregate => "Aggregate",
            Self::Exchange(Exchange::Gdax) => "Coinbase",
            Self::Exchange(Exchange::Kraken) => "Kraken",
            Self::Exchange(Exchange::Hyperliquid) => "Hyperliquid",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggBook {
    pub bids: BTreeMap<Decimal, Level>,
    pub asks: BTreeMap<Decimal, Level>,
}

impl AggBook {
    pub fn new() -> Self {
        AggBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }
}

impl Default for AggBook {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Level {
    pub size: Decimal,
    pub exchange: AggExchange,
}

#[tokio::main]
async fn main() -> Result<()> {
    better_panic::install();
    setup_panic_hook();
    // tracing_subscriber::fmt::init();

    // A layer that logs events to a file.
    let file = File::create("debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };
    let log = tracing_subscriber::fmt().with_writer(Arc::new(file));
    log.init();

    let mut terminal = init_terminal()?;
    let mut app = App::new()?;
    // Add initial subs to queue
    app.queue_subs().await;

    run(&mut terminal, &mut app).await?;

    reset_terminal()?;
    Ok(())
}

async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut reader = EventStream::new();
    let mut interval = time::interval(Duration::from_millis(350));

    loop {
        app.update_state();
        terminal.draw(|f| ui(f, app))?;
        // app.manage_state().await;

        tokio::select! {
            _ = interval.tick() => app.manage_state().await,
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key_event))) => {
                        if key_event.kind == KeyEventKind::Press && !app.handle_key_press(&key_event).await {
                                break
                        }
                    },
                    Some(Ok(_)) => {},
                    Some(Err(_)) => { break },
                    None => {},
                }
            },
            msg = app.client.receiver.recv() => {
                if let Some(Ok(msg)) = msg {
                    app.response_handler(msg);
                }
            }
        };
    }
    Ok(())
}

/// Initializes the terminal.
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    Ok(terminal)
}

/// Resets the terminal.
fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        reset_terminal().unwrap();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}
