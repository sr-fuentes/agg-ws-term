use std::collections::{HashMap, HashSet, VecDeque};

use agg_ws::{
    book::Book,
    client::{AsyncClient, Channel, ChannelType, ClientResp, ClientRespMsg, Exchange},
    trades::Trade,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::{AggBook, AggExchange, Level, Result};

pub struct App {
    pub screens: Vec<AppFocus>,
    pub screen_idx: usize,
    pub assets: Vec<String>,
    pub asset_idx: usize,
    pub exchanges: HashMap<usize, Vec<AggExchange>>,
    pub exchange_state: HashMap<usize, ListState>,
    pub trades: Vec<Trade>,
    pub book: AggBook,
    pub client: AsyncClient,
    pub subscriptions: HashMap<usize, HashSet<Channel>>,
    pub tickers: HashMap<usize, HashMap<Exchange, String>>, // TODO: Load from db or config
    pub sub_queue: HashSet<Channel>,
    pub tapes: HashMap<Channel, VecDeque<Trade>>,
    pub books: HashMap<Channel, Book>,
    pub dp: HashMap<usize, u32>,
}

pub enum AppFocus {
    AssetTab,
    Exchange,
}

impl App {
    pub fn new() -> Result<App> {
        let mut default_list_state = ListState::default();
        default_list_state.select(Some(0));
        Ok(App {
            screens: vec![AppFocus::AssetTab, AppFocus::Exchange],
            screen_idx: 0,
            assets: vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()],
            asset_idx: 2,
            exchanges: HashMap::from([
                (
                    0,
                    vec![
                        AggExchange::Aggregate,
                        AggExchange::Exchange(Exchange::Kraken),
                        AggExchange::Exchange(Exchange::Gdax),
                        AggExchange::Exchange(Exchange::Hyperliquid),
                    ],
                ),
                (
                    1,
                    vec![
                        AggExchange::Aggregate,
                        AggExchange::Exchange(Exchange::Kraken),
                        AggExchange::Exchange(Exchange::Gdax),
                        AggExchange::Exchange(Exchange::Hyperliquid),
                    ],
                ),
                (
                    2,
                    vec![
                        AggExchange::Aggregate,
                        AggExchange::Exchange(Exchange::Kraken),
                        AggExchange::Exchange(Exchange::Gdax),
                        AggExchange::Exchange(Exchange::Hyperliquid),
                    ],
                ),
            ]),
            exchange_state: HashMap::from([
                (0, default_list_state.clone()),
                (1, default_list_state.clone()),
                (2, default_list_state.clone()),
                (3, default_list_state.clone()),
            ]),
            trades: Vec::with_capacity(50),
            book: AggBook::new(),
            client: AsyncClient::new(),
            subscriptions: HashMap::new(),
            tickers: HashMap::from([
                (
                    0,
                    HashMap::from([
                        (Exchange::Gdax, "BTC-USD".to_string()),
                        (Exchange::Kraken, "XBT/USD".to_string()),
                        (Exchange::Hyperliquid, "BTC".to_string()),
                    ]),
                ),
                (
                    1,
                    HashMap::from([
                        (Exchange::Gdax, "ETH-USD".to_string()),
                        (Exchange::Kraken, "ETH/USD".to_string()),
                        (Exchange::Hyperliquid, "ETH".to_string()),
                    ]),
                ),
                (
                    2,
                    HashMap::from([
                        (Exchange::Gdax, "SOL-USD".to_string()),
                        (Exchange::Kraken, "SOL/USD".to_string()),
                        (Exchange::Hyperliquid, "SOL".to_string()),
                    ]),
                ),
            ]),
            sub_queue: HashSet::new(),
            tapes: HashMap::new(),
            books: HashMap::new(),
            dp: HashMap::from([(0, 8), (1, 8), (2, 3)]),
        })
    }

    pub fn response_handler(&mut self, resp_msg: ClientRespMsg) {
        tracing::info!("Response handler {:?}", resp_msg.channel);
        match resp_msg.resp {
            ClientResp::Subscribed => self.handle_subscribed(resp_msg.channel),
            ClientResp::Unsubscribed => {}
            ClientResp::Tape(t) => self.handle_tape(resp_msg.channel, t),
            ClientResp::Book(b) => self.handle_book(resp_msg.channel, b),
            ClientResp::Last(_) => {}
        }
    }

    pub fn handle_subscribed(&mut self, channel: Channel) {
        self.subscriptions
            .entry(self.asset_idx)
            .and_modify(|hs| {
                hs.insert(channel.clone());
            })
            .or_insert_with(|| HashSet::from([channel.clone()]));
    }

    pub fn handle_tape(&mut self, channel: Channel, tape: VecDeque<Trade>) {
        tracing::info!("Tape: {:?}", tape);
        self.tapes.insert(channel, tape);
    }

    pub fn handle_book(&mut self, channel: Channel, book: Book) {
        self.books.insert(channel, book);
    }

    fn get_channels(&self) -> Vec<Channel> {
        let mut channels = Vec::new();
        // Get trade channels
        for ticker in self.tickers.get(&self.asset_idx).unwrap().iter() {
            let channel = Channel {
                exchange: *ticker.0,
                channel: ChannelType::Tape,
                market: ticker.1.clone(),
            };
            channels.push(channel);
            let channel = Channel {
                exchange: *ticker.0,
                channel: ChannelType::Book,
                market: ticker.1.clone(),
            };
            channels.push(channel);
        }
        channels
    }

    pub async fn manage_state(&mut self) {
        if !self.sub_queue.is_empty() {
            self.subscribe_channel().await;
        }
        self.poll_data().await;
        // tokio::time::sleep(Duration::from_millis(250)).await;
    }

    async fn subscribe_channel(&mut self) {
        if let Some(channel) = self.sub_queue.iter().next().cloned() {
            self.sub_queue.remove(&channel);
            tracing::info!("Subscribing to {:?}", channel);
            match self.client.start_and_subscribe(channel.clone()).await {
                Ok(resp) => {
                    tracing::info!("Sub req resp: {:?}", resp);
                }
                Err(e) => tracing::error!("Sub error: {:?}", e),
            }
        }
    }

    pub async fn queue_subs(&mut self) {
        // Get channels and send to client to subscirbe
        let channels = self.get_channels();
        tracing::debug!("Channels: {:?}", channels);
        for channel in channels.iter() {
            // Skip if already subscribed to channel
            if let Some(subs) = self.subscriptions.get(&self.asset_idx) {
                if subs.contains(channel) {
                    tracing::debug!("Channel {:?} already subbed.", channel);
                    continue;
                };
            }
            tracing::info!("Inserting channel into sub queue: {:?}", channel);
            self.sub_queue.insert(channel.clone());
        }
    }

    // Poll data for asset and aggregate if needed
    pub async fn poll_data(&mut self) {
        if let Some(exchange_idx) = self.exchange_state.get(&self.asset_idx).unwrap().selected() {
            // There is an exchange selected, poll trades and insert into app data storage
            if exchange_idx == 0 {
                let channels = self.get_channels();
                tracing::info!("Polling all tapes and books.");
                for channel in channels.iter() {
                    match channel.channel {
                        ChannelType::Tape => {
                            self.client.get_tape(channel.clone()).await.unwrap();
                        }
                        ChannelType::Book => {
                            self.client.get_book(channel.clone()).await.unwrap();
                        }
                    };
                }
            } else {
                let exchange = self.exchanges.get(&self.asset_idx).unwrap()[exchange_idx].clone();
                if let AggExchange::Exchange(ex) = exchange {
                    let ticker = self.tickers.get(&self.asset_idx).unwrap().get(&ex).unwrap();
                    let channel = Channel {
                        exchange: ex,
                        channel: ChannelType::Tape,
                        market: ticker.clone(),
                    };
                    tracing::info!("Polling {:?} - {:?} tape and book.", ex, ticker);
                    self.client.get_tape(channel.clone()).await.unwrap();
                    // Get Book for Exchange
                    let channel = Channel {
                        exchange: ex,
                        channel: ChannelType::Book,
                        market: ticker.clone(),
                    };
                    self.client.get_book(channel.clone()).await.unwrap();
                }
            }
        }
    }

    // Update the app Book and Trades state based on the selected Asset and Exchange
    pub fn update_state(&mut self) {
        self.trades = Vec::with_capacity(50);
        self.book = AggBook::new();
        if let Some(exchange_idx) = self.exchange_state.get(&self.asset_idx).unwrap().selected() {
            // Aggregated Exchange is selected - Merge all tapes and book for Asset
            if exchange_idx == 0 {
                let channels = self.get_channels();
                self.update_state_agg_trades(&channels);
                // Get all books and aggregate them into one
                self.update_state_agg_book(&channels);
            } else {
                // Copy the book and tape for the given Exchange and Asset
                let exchange = self.exchanges.get(&self.asset_idx).unwrap()[exchange_idx].clone();
                if let AggExchange::Exchange(ex) = exchange {
                    let ticker = self
                        .tickers
                        .get(&self.asset_idx)
                        .unwrap()
                        .get(&ex)
                        .unwrap()
                        .clone();
                    // Copy the tape
                    self.update_state_trades(&ex, &ticker);
                    // Get book for exchange and convert into AggBook structure
                    self.update_state_book(&ex, &ticker);
                }
            }
        }
    }

    pub fn update_state_agg_trades(&mut self, channels: &[Channel]) {
        let mut trades: Vec<Trade> = Vec::with_capacity(100 * channels.len());
        for channel in channels.iter() {
            if let Some(trades_vd) = self.tapes.get(channel).cloned() {
                trades.append(&mut trades_vd.into());
            }
        }
        trades.sort_by_key(|t| t.dt);
        self.trades = trades.into_iter().rev().take(25).collect();
    }

    pub fn update_state_trades(&mut self, exchange: &Exchange, ticker: &str) {
        let channel = Channel {
            exchange: *exchange,
            channel: ChannelType::Tape,
            market: ticker.to_string(),
        };
        if let Some(trades_vd) = self.tapes.get(&channel).cloned() {
            let trades_v: Vec<Trade> = trades_vd.into();
            self.trades = trades_v.into_iter().rev().take(25).collect();
        } else {
            self.trades = Vec::with_capacity(50);
        }
    }

    pub fn update_state_agg_book(&mut self, channels: &[Channel]) {
        let mut agg_book = AggBook::new();
        for channel in channels.iter() {
            if let Some(book) = self.books.get(channel).cloned() {
                self.merge_exchange_book(&mut agg_book, &book, channel.exchange);
            }
        }
        self.book = agg_book;
    }

    pub fn update_state_book(&mut self, exchange: &Exchange, ticker: &str) {
        let channel = Channel {
            exchange: *exchange,
            channel: ChannelType::Book,
            market: ticker.to_string(),
        };
        if let Some(book) = self.books.get(&channel).cloned() {
            self.map_exchange_book(channel.exchange, book);
        } else {
            self.book = AggBook::new();
        }
    }

    pub fn map_exchange_book(&mut self, exchange: Exchange, book: Book) {
        let mut agg_book = AggBook::new();
        agg_book.asks.extend(book.asks.iter().map(|l| {
            (
                *l.0,
                Level {
                    size: *l.1,
                    exchange: AggExchange::Exchange(exchange),
                },
            )
        }));
        agg_book.bids.extend(book.bids.iter().map(|l| {
            (
                *l.0,
                Level {
                    size: *l.1,
                    exchange: AggExchange::Exchange(exchange),
                },
            )
        }));
        self.book = agg_book;
    }

    pub fn merge_exchange_book(&self, agg_book: &mut AggBook, book: &Book, exchange: Exchange) {
        // Merge asks
        for ask in book.asks.iter() {
            agg_book
                .asks
                .entry(*ask.0)
                .and_modify(|l| {
                    l.size += ask.1;
                    l.exchange = AggExchange::Aggregate;
                })
                .or_insert_with(|| Level {
                    size: *ask.1,
                    exchange: AggExchange::Exchange(exchange),
                });
        }
        // Merge bids
        for bid in book.bids.iter() {
            agg_book
                .bids
                .entry(*bid.0)
                .and_modify(|l| {
                    l.size += bid.1;
                    l.exchange = AggExchange::Aggregate;
                })
                .or_insert_with(|| Level {
                    size: *bid.1,
                    exchange: AggExchange::Exchange(exchange),
                });
        }
    }

    // Screen focus is not currently implemented. Key event is structure to change key functionality
    // based on what screen is selected
    pub async fn handle_key_press(&mut self, key_event: &KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('q') => {
                return false;
            }
            KeyCode::Right => self.next_asset().await,
            KeyCode::Left => self.prev_asset().await,
            KeyCode::Down => self.next_exchange(),
            KeyCode::Up => self.prev_exchange(),
            KeyCode::Enter => self.unselect(),
            KeyCode::Tab => self.next_focus(),
            _ => (),
        }
        true
    }

    pub fn next_focus(&mut self) {
        self.screen_idx = (self.screen_idx + 1) % self.screens.len();
    }

    pub async fn next_asset(&mut self) {
        self.asset_idx = (self.asset_idx + 1) % self.assets.len();
        self.queue_subs().await;
    }

    pub async fn prev_asset(&mut self) {
        if self.asset_idx > 0 {
            self.asset_idx -= 1;
        } else {
            self.asset_idx = self.assets.len() - 1;
        }
        self.queue_subs().await;
    }

    pub fn next_exchange(&mut self) {
        self.exchange_state.entry(self.asset_idx).and_modify(|ls| {
            let i = match ls.selected() {
                Some(i) => {
                    if i >= self.exchanges.get(&self.asset_idx).unwrap().len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            ls.select(Some(i));
        });
    }

    pub fn prev_exchange(&mut self) {
        self.exchange_state.entry(self.asset_idx).and_modify(|ls| {
            let i = match ls.selected() {
                Some(i) => {
                    if i == 0 {
                        self.exchanges.get(&self.asset_idx).unwrap().len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            ls.select(Some(i));
        });
    }

    pub fn unselect(&mut self) {
        self.exchange_state.entry(self.asset_idx).and_modify(|ls| {
            ls.select(None);
        });
    }

    pub async fn _last_message(&mut self) {
        for channel in self.get_channels() {
            tracing::info!("Getting last message for {:?}", channel);
            let last_message = self.client.get_last(channel).await;
            tracing::info!("Last message: {:?}", last_message);
        }
    }
}
