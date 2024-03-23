use std::io;
use std::io::Write;

use futures_util::{SinkExt, StreamExt};
use futures_util::stream::SplitStream;
use sonic_rs::{get, JsonValueTrait};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let url = "wss://fstream.binance.com/ws";
    let (ws_stream, _response) = connect_async(url).await.expect("Failed to connect");
    let (mut sink, stream) = ws_stream.split();
    let sub_request = r#"{
        "method": "SUBSCRIBE",
        "params":
        [
        "btcusdt@trade"
        ],
        "id": 1
        }"#;
    let sub_msg = Message::Text(sub_request.into());
    sink.send(sub_msg).await.expect("Failed to send subscription message");
    let stream_handle = tokio::spawn(handle_stream(stream));
    let _ = tokio::try_join!(stream_handle);
}

async fn handle_stream(mut stream: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>) {
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => handle_message(msg),
            Err(e) => eprintln!("Error receiving message: {}", e),
        }
    }
}

fn handle_message(message: Message) {
    let get_event = get(message.to_text().unwrap(), ["e"]);
    let is_trade = get_event.is_ok() && get_event.as_str().unwrap() == "trade";
    if is_trade {
        let price = get(message.to_text().unwrap(), ["p"]);
        let price = price.as_str().unwrap();
        let is_sell = get(message.to_text().unwrap(), ["m"]).as_bool().unwrap();
        let formatted_price = if is_sell {
            format!("\x1b[31m{}\x1b[0m", price)
        } else {
            format!("\x1b[32m{}\x1b[0m", price)
        };
        print!("\r");
        print!("BINANCEFUT_BTCUSDT - Last price: {}", formatted_price);
        io::stdout().flush().unwrap();
    }
}