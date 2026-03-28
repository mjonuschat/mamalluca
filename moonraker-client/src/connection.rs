//! Low-level WebSocket connection wrapper for Moonraker communication.
//!
//! This module provides a thin abstraction over `tokio-tungstenite` for
//! connecting to a Moonraker instance via WebSocket. It handles only raw I/O:
//! connecting, sending text frames, receiving messages, and closing.
//!
//! **No protocol logic or reconnection** lives here — those concerns belong
//! in higher-level modules (`jsonrpc` and the future reconnect module).

// This module is `pub(crate)` and will be consumed by the reconnect module
// (Task 8). Until then, the compiler sees these items as unused.
#![allow(dead_code)]

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use url::Url;

/// The concrete WebSocket stream type returned by `connect_async`.
///
/// `MaybeTlsStream` transparently handles both plain TCP and TLS connections
/// depending on whether the URL uses `ws://` or `wss://`.
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// A WebSocket connection to a Moonraker instance.
///
/// Wraps `tokio-tungstenite` with a simple send/receive API. The stream is
/// split into independent sink (send) and stream (receive) halves so that
/// future callers can send and receive concurrently from separate tasks.
///
/// Does **not** handle reconnection — that is the responsibility of the
/// reconnect module (Task 8).
pub(crate) struct Connection {
    /// The write half of the WebSocket stream.
    sink: SplitSink<WsStream, Message>,
    /// The read half of the WebSocket stream.
    stream: SplitStream<WsStream>,
}

impl Connection {
    /// Connect to a Moonraker WebSocket endpoint.
    ///
    /// Performs the WebSocket handshake and splits the resulting stream into
    /// independent send/receive halves.
    ///
    /// # Parameters
    /// - `url`: The Moonraker WebSocket URL (e.g. `ws://192.168.1.100:7125/websocket`).
    ///
    /// # Errors
    /// Returns [`ConnectionError::WebSocket`] if the TCP connection or
    /// WebSocket handshake fails.
    pub async fn connect(url: &Url) -> Result<Self, ConnectionError> {
        // Use `as_str()` because tungstenite's `IntoClientRequest` impl for
        // `url::Url` requires the optional "url" feature. The `&str` impl is
        // always available.
        let (ws_stream, _response) = connect_async(url.as_str()).await?;
        let (sink, stream) = ws_stream.split();
        Ok(Self { sink, stream })
    }

    /// Send a text message over the WebSocket.
    ///
    /// Moonraker's JSON-RPC protocol uses only text frames, so this is the
    /// sole send method needed.
    ///
    /// # Parameters
    /// - `text`: The UTF-8 text to send as a WebSocket text frame.
    ///
    /// # Errors
    /// Returns [`ConnectionError::WebSocket`] if sending fails (e.g. the
    /// connection has been closed).
    pub async fn send_text(&mut self, text: &str) -> Result<(), ConnectionError> {
        self.sink.send(Message::Text(text.into())).await?;
        Ok(())
    }

    /// Receive the next message from the WebSocket.
    ///
    /// Blocks until a message arrives or the connection closes. Ping and pong
    /// frames are silently consumed (tungstenite handles pong replies
    /// automatically).
    ///
    /// # Returns
    /// - `Some(Ok(text))` — a text message was received
    /// - `Some(Err(e))` — a protocol error or unexpected message type
    /// - `None` — the connection was closed (close frame received or stream ended)
    ///
    /// # Errors
    /// Returns [`ConnectionError::UnexpectedBinaryMessage`] if a binary frame
    /// arrives (Moonraker only sends text). Returns [`ConnectionError::WebSocket`]
    /// for protocol-level errors.
    pub async fn next_message(&mut self) -> Option<Result<String, ConnectionError>> {
        loop {
            match self.stream.next().await? {
                Ok(Message::Text(text)) => return Some(Ok(text.to_string())),
                Ok(Message::Close(_)) => return None,
                // Ping/pong frames are handled automatically by tungstenite's
                // codec layer (it sends pong replies). We just skip them here.
                Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                Ok(Message::Binary(_) | Message::Frame(_)) => {
                    return Some(Err(ConnectionError::UnexpectedBinaryMessage));
                }
                Err(e) => return Some(Err(e.into())),
            }
        }
    }

    /// Send a close frame and shut down the connection gracefully.
    ///
    /// After calling this, no further messages should be sent. The remote end
    /// will respond with its own close frame, which `next_message` will surface
    /// as `None`.
    ///
    /// # Errors
    /// Returns [`ConnectionError::WebSocket`] if sending the close frame fails.
    pub async fn close(&mut self) -> Result<(), ConnectionError> {
        self.sink.send(Message::Close(None)).await?;
        Ok(())
    }
}

/// Errors that can occur during WebSocket communication.
///
/// These represent low-level transport failures, not application-level
/// JSON-RPC errors (those are handled in the `jsonrpc` module).
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    /// WebSocket protocol or transport error (handshake failure, broken pipe, etc.).
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// Received an unexpected binary WebSocket message.
    ///
    /// Moonraker communicates exclusively via text (JSON) frames.
    /// A binary frame indicates either a misconfigured proxy or a
    /// protocol mismatch.
    #[error("Received unexpected binary WebSocket message")]
    UnexpectedBinaryMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the Display implementation for the binary-message error variant.
    #[test]
    fn connection_error_display_unexpected_binary() {
        let err = ConnectionError::UnexpectedBinaryMessage;
        assert_eq!(
            err.to_string(),
            "Received unexpected binary WebSocket message"
        );
    }

    /// Verifies that a tungstenite error converts into `ConnectionError::WebSocket`
    /// via the `From` implementation.
    #[test]
    fn connection_error_from_tungstenite() {
        // `tungstenite::Error::ConnectionClosed` is a simple variant we can construct
        // without any dependencies.
        let ws_err = tokio_tungstenite::tungstenite::Error::ConnectionClosed;
        let conn_err = ConnectionError::from(ws_err);

        // The Display output should wrap the inner error.
        assert!(
            conn_err.to_string().contains("WebSocket error"),
            "expected 'WebSocket error' prefix, got: {}",
            conn_err
        );
    }
}
