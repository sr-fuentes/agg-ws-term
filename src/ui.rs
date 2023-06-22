use agg_ws::{client::Exchange, trades::Trade};
use chrono::Utc;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Tabs},
    Frame,
};
use rust_decimal::prelude::*;

use crate::app::App;

fn px_fmt(s: &str) -> String {
    // Format the string of trade and price for the display by rounding to 5 significant figure
    Decimal::from_str(s)
        .unwrap()
        .round_sf(7)
        .unwrap()
        .to_string()
}

fn sz_fmt(s: &str, dp: u32) -> String {
    let mut dec = Decimal::from_str(s).unwrap();
    dec.rescale(dp);
    format!("{}", dec)
}

fn sz_fmt_dec(mut s: Decimal, dp: u32) -> String {
    s.rescale(dp);
    format!("{}", s)
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    // Render Full Screen Block
    let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::Black));
    f.render_widget(block, size);

    // Split Screen into Top and Bottom Chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(5)
        .vertical_margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    // Render Tabs into Top Chunk
    let tab_block_style = Style::default()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::BOLD);

    let assets = app
        .assets
        .iter()
        .map(|a| Line::from(Span::styled(a, Style::default().fg(Color::White))))
        .collect();
    let tabs = Tabs::new(assets)
        .block(Block::default().borders(Borders::ALL).title(" Assets "))
        .select(app.asset_idx)
        .style(tab_block_style)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)
                .fg(Color::LightYellow),
        );
    f.render_widget(tabs, chunks[0]);

    // Render Main Screen into Lower Chunk
    let exchange_block_style = Style::default()
        .fg(Color::LightYellow)
        // .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);

    let block = Block::default()
        .title(format!(" {} ", app.assets[app.asset_idx]))
        .borders(Borders::ALL)
        .style(exchange_block_style);
    f.render_widget(block, chunks[1]);

    // Split Main Screen into 3 Chunks
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(5)
        .vertical_margin(2)
        .constraints(
            [
                Constraint::Length(15),
                Constraint::Percentage(2),
                Constraint::Length(50),
                Constraint::Percentage(2),
                Constraint::Length(50),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    // Set Styles
    let header_style = Style::default()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::BOLD);

    let row_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    // Render Exchanges List into Left Main Chunk
    let exchanges: Vec<ListItem> = app
        .exchanges
        .get(&app.asset_idx)
        .unwrap()
        .iter()
        .map(|e| ListItem::new(Text::from(e.as_display())))
        .collect();
    let exchanges = List::new(exchanges)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Exchanges ")
                .title_alignment(Alignment::Center)
                .style(exchange_block_style)
                .padding(Padding::vertical(1)),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)
                .fg(Color::LightYellow),
        )
        .style(Style::default().fg(Color::White));
    let exchange_state = app.exchange_state.get_mut(&app.asset_idx).unwrap();
    f.render_stateful_widget(exchanges, main_chunks[0], exchange_state);

    // Render Trades into Middle Main Chunk
    // Split the Main Chunk into 4 Vertical Chunks - Size | Price | Time | Exchange
    // Split each Vertical Chunk into 51 Horizontal Chunks - One for each trade and a single header
    // Table Widget cannot be used as Alignment is not Available

    // Render Trades Box into Center Main Chunk
    let block = Block::default()
        .title(" Trades ")
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center)
        .padding(Padding::vertical(1))
        .style(exchange_block_style);
    f.render_widget(block, main_chunks[2]);

    let trade_columns = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints(
            [
                Constraint::Max(1),
                Constraint::Length(10), // Size
                Constraint::Length(2),  // Space
                Constraint::Length(10), // Price
                Constraint::Length(2),  // Space
                Constraint::Length(8),  // Time
                Constraint::Length(2),  // Space
                Constraint::Length(13), // Exchange
                Constraint::Percentage(1),
            ]
            .as_ref(),
        )
        .split(main_chunks[2]);

    let _test_trades = vec![
        Trade {
            size: "123.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "23.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "23.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "133.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "143.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "153.23".to_string(),
            price: "123215.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "143.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "133.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "3.23".to_string(),
            price: "123214.613".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "1253.23".to_string(),
            price: "123213.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "12343.23".to_string(),
            price: "123212.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "12.23".to_string(),
            price: "123216.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "13.23".to_string(),
            price: "123218.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "3.23".to_string(),
            price: "123218.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "13.23".to_string(),
            price: "123217.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "133.23".to_string(),
            price: "123216.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "143.23".to_string(),
            price: "123215.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "133.23".to_string(),
            price: "123215.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "143.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "133.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "143.23".to_string(),
            price: "123234.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "23.23".to_string(),
            price: "123256.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "23.23".to_string(),
            price: "123245.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "23.23".to_string(),
            price: "123243.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "3.23".to_string(),
            price: "123214.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "23.23".to_string(),
            price: "123344.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "0.23".to_string(),
            price: "123244.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "13.23".to_string(),
            price: "123414.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "13.23".to_string(),
            price: "123314.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
        Trade {
            size: "3.23".to_string(),
            price: "123254.213".to_string(),
            dt: Utc::now(),
            exchange: Exchange::Kraken,
        },
    ];
    // app.trades = test_trades;
    let n = app.trades.len();
    let (sizes, prices, dts, exchanges) = app.trades.iter().fold(
        {
            let mut sizes = Vec::with_capacity(n + 2);
            sizes.push(Line::from(Span::styled("Size ", header_style)).alignment(Alignment::Right));
            sizes.push(Line::from("").alignment(Alignment::Right));
            let mut prices = Vec::with_capacity(n + 2);
            prices.push(Line::from("Price ").alignment(Alignment::Right));
            prices.push(Line::from("").alignment(Alignment::Right));
            let mut dts = Vec::with_capacity(n + 2);
            dts.push(Line::from("Time ").alignment(Alignment::Right));
            dts.push(Line::from("").alignment(Alignment::Right));
            let mut exchanges = Vec::with_capacity(n + 2);
            exchanges.push(Line::from("Exchange ").alignment(Alignment::Right));
            exchanges.push(Line::from("").alignment(Alignment::Right));
            (sizes, prices, dts, exchanges)
        },
        |(mut s, mut p, mut d, mut e), t| {
            s.push(
                Line::from(Span::styled(
                    sz_fmt(&t.size, *app.dp.get(&app.asset_idx).unwrap()),
                    row_style,
                ))
                .alignment(Alignment::Right),
            );
            p.push(
                Line::from(Span::styled(px_fmt(&t.price), row_style)).alignment(Alignment::Right),
            );
            d.push(
                Line::from(Span::styled(format!("{}", t.dt.time()), row_style))
                    .alignment(Alignment::Right),
            );
            e.push(
                Line::from(Span::styled(t.exchange.as_display(), row_style))
                    .alignment(Alignment::Right),
            );
            (s, p, d, e)
        },
    );
    let size_paragraph = Paragraph::new(sizes.clone()).alignment(Alignment::Right);
    f.render_widget(size_paragraph, trade_columns[1]);

    let price_paragraph = Paragraph::new(prices.clone()).alignment(Alignment::Right);
    f.render_widget(price_paragraph, trade_columns[3]);

    let time_paragraph = Paragraph::new(dts.clone()).alignment(Alignment::Right);
    f.render_widget(time_paragraph, trade_columns[5]);

    let exchange_paragraph = Paragraph::new(exchanges.clone()).alignment(Alignment::Right);
    f.render_widget(exchange_paragraph, trade_columns[7]);

    // Render Book Box into Left Main Chunk
    // Split the Main Chunk into 6 Vertical Chunks - BidExchange | BidSize | BidPrice | AskPrice | AskSize | AskExchange
    let block = Block::default()
        .title(" Book ")
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center)
        .padding(Padding::vertical(1))
        .style(exchange_block_style);
    f.render_widget(block, main_chunks[4]);

    let book_columns = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints(
            [
                Constraint::Max(1),
                Constraint::Length(13), // Exchange
                Constraint::Length(2),  // Space
                Constraint::Length(10), // Size
                Constraint::Length(2),  // Space
                Constraint::Length(10), // Price
                Constraint::Length(2),  // Space
                Constraint::Length(10), // Price
                Constraint::Length(2),  // Space
                Constraint::Length(10), // Size
                Constraint::Length(2),  // Space
                Constraint::Length(13), // Exchange
                Constraint::Percentage(1),
            ]
            .as_ref(),
        )
        .split(main_chunks[4]);

    // Set Styles
    let bid_row_style = Style::default().fg(Color::Cyan);

    let n_bid = app.book.bids.len();
    let (prices, sizes, exchanges) = app.book.bids.iter().rev().fold(
        {
            let mut sizes = Vec::with_capacity(n_bid + 2);
            sizes.push(Line::from(Span::styled("Size", header_style)).alignment(Alignment::Right));
            sizes.push(Line::from(Span::styled("", header_style)).alignment(Alignment::Right));
            let mut prices = Vec::with_capacity(n + 2);
            prices
                .push(Line::from(Span::styled("Price", header_style)).alignment(Alignment::Right));
            prices.push(Line::from("").alignment(Alignment::Right));
            let mut exchanges = Vec::with_capacity(n + 2);
            exchanges.push(
                Line::from(Span::styled("Exchange", header_style)).alignment(Alignment::Right),
            );
            exchanges.push(Line::from("").alignment(Alignment::Right));
            (prices, sizes, exchanges)
        },
        |(mut p, mut s, mut e), l| {
            s.push(
                Line::from(Span::styled(
                    sz_fmt_dec(l.1.size, *app.dp.get(&app.asset_idx).unwrap()),
                    row_style,
                ))
                .alignment(Alignment::Right),
            );
            p.push(
                Line::from(Span::styled(
                    l.0.round_sf(7).unwrap().to_string(),
                    bid_row_style,
                ))
                .alignment(Alignment::Right),
            );
            e.push(
                Line::from(Span::styled(l.1.exchange.as_display(), row_style))
                    .alignment(Alignment::Right),
            );
            (p, s, e)
        },
    );

    let size_paragraph = Paragraph::new(sizes.clone()).alignment(Alignment::Right);
    f.render_widget(size_paragraph, book_columns[3]);

    let price_paragraph = Paragraph::new(prices.clone()).alignment(Alignment::Right);
    f.render_widget(price_paragraph, book_columns[5]);

    let exchange_paragraph = Paragraph::new(exchanges.clone()).alignment(Alignment::Right);
    f.render_widget(exchange_paragraph, book_columns[1]);

    let ask_row_style = Style::default().fg(Color::Red);

    let n_ask = app.book.asks.len();
    let (prices, sizes, exchanges) = app.book.asks.iter().fold(
        {
            let mut sizes = Vec::with_capacity(n_ask + 2);
            sizes.push(Line::from(Span::styled("Size ", header_style)).alignment(Alignment::Right));
            sizes.push(Line::from("").alignment(Alignment::Right));
            let mut prices = Vec::with_capacity(n_ask + 2);
            prices.push(Line::from("Price ").alignment(Alignment::Right));
            prices.push(Line::from("").alignment(Alignment::Right));
            let mut exchanges = Vec::with_capacity(n_ask + 2);
            exchanges.push(Line::from("Exchange ").alignment(Alignment::Right));
            exchanges.push(Line::from("").alignment(Alignment::Right));
            (prices, sizes, exchanges)
        },
        |(mut p, mut s, mut e), l| {
            s.push(
                Line::from(Span::styled(
                    sz_fmt_dec(l.1.size, *app.dp.get(&app.asset_idx).unwrap()),
                    row_style,
                ))
                .alignment(Alignment::Right),
            );
            p.push(
                Line::from(Span::styled(
                    l.0.round_sf(7).unwrap().to_string(),
                    ask_row_style,
                ))
                .alignment(Alignment::Right),
            );
            e.push(
                Line::from(Span::styled(l.1.exchange.as_display(), row_style))
                    .alignment(Alignment::Right),
            );
            (p, s, e)
        },
    );

    let size_paragraph = Paragraph::new(sizes.clone()).alignment(Alignment::Right);
    f.render_widget(size_paragraph, book_columns[9]);

    let price_paragraph = Paragraph::new(prices.clone()).alignment(Alignment::Right);
    f.render_widget(price_paragraph, book_columns[7]);

    let exchange_paragraph = Paragraph::new(exchanges.clone()).alignment(Alignment::Right);
    f.render_widget(exchange_paragraph, book_columns[11]);
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::*;

    #[test]
    pub fn rounding() {
        let price = "234.123324234".to_string();
        // Convert to Decimal -> Format to 5g -> Convert to Decimal -> Round to 6
        let price_decimal = Decimal::from_str(&price).unwrap();
        let price_formatted = price_decimal.round_sf(5).unwrap().to_string();
        println!("Formated: {}", price_formatted);
    }

    #[test]
    pub fn scaling() {
        let px1 = "194.23423";
        let px2 = "0.12321421";

        let dp = 8;

        let mut d1 = Decimal::from_str(px1).unwrap();
        let mut d2 = Decimal::from_str(px2).unwrap();

        println!("Px1:\t{}", d1);
        println!("Px2:\t{}", d2);

        d1.rescale(dp);
        d2.rescale(dp);

        println!("Px1:\t{}", d1);
        println!("Px2:\t{}", d2);
    }
}
