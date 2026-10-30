use alloc::string::{String, ToString};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Channel};
use futures::channel::oneshot::{self, Receiver, Sender};
use hashbrown::HashMap;
use serde_json::Value;

use crate::asynch::clients::exceptions::XRPLClientResult;

use super::exceptions::XRPLWebSocketException;

const _MAX_CHANNEL_MSG_CNT: usize = 10;

/// A struct that handles futures of websocket messages.
pub struct WebsocketBase<M>
where
    M: RawMutex,
{
    /// The messages the user requests, which means he is waiting for a specific `id`.
    pending_requests: HashMap<String, Receiver<String>>,
    request_senders: HashMap<String, Sender<String>>,
    /// The messages the user waits for when sending and receiving normally.
    messages: Channel<M, String, _MAX_CHANNEL_MSG_CNT>,
}

impl<M> WebsocketBase<M>
where
    M: RawMutex,
{
    pub fn new() -> Self {
        Self {
            pending_requests: HashMap::new(),
            request_senders: HashMap::new(),
            messages: Channel::new(),
        }
    }

    pub fn close(&mut self) {
        self.pending_requests.clear();
        self.request_senders.clear();
        self.messages.clear();
    }
}

#[allow(async_fn_in_trait)]
pub trait MessageHandler {
    /// Setup an empty future for a request.
    async fn setup_request_future(&mut self, id: String);
    async fn handle_message(&mut self, message: String) -> XRPLClientResult<()>;
    async fn pop_message(&mut self) -> String;
    async fn try_recv_request(&mut self, id: String) -> XRPLClientResult<Option<String>>;
}

impl<M> MessageHandler for WebsocketBase<M>
where
    M: RawMutex,
{
    async fn setup_request_future(&mut self, id: String) {
        if self.pending_requests.contains_key(&id) {
            return;
        }
        let (sender, receiver) = oneshot::channel::<String>();
        self.pending_requests.insert(id.clone(), receiver);
        self.request_senders.insert(id, sender);
    }

    async fn handle_message(&mut self, message: String) -> XRPLClientResult<()> {
        let message_value: Value = serde_json::from_str(&message)?;
        let id = match message_value.get("id") {
            Some(id) => match id.as_str() {
                Some(id) => id.to_string(),
                None => return Err(XRPLWebSocketException::InvalidMessage.into()),
            },
            None => String::new(),
        };
        if let Some(_receiver) = self.pending_requests.get(&id) {
            let sender = match self.request_senders.remove(&id) {
                Some(sender) => sender,
                None => return Err(XRPLWebSocketException::MissingRequestSender.into()),
            };
            // On failure `send` hands back the message that could not be
            // delivered; discard it so the raw response body is not surfaced
            // into error-reporting channels.
            sender
                .send(message)
                .map_err(|_| XRPLWebSocketException::MessageChannelError)?;
        } else {
            self.messages.send(message).await;
        }
        Ok(())
    }

    async fn pop_message(&mut self) -> String {
        self.messages.receive().await
    }

    async fn try_recv_request(&mut self, id: String) -> XRPLClientResult<Option<String>> {
        let fut = match self.pending_requests.get_mut(&id) {
            Some(fut) => fut,
            None => return Err(XRPLWebSocketException::MissingRequestReceiver.into()),
        };
        match fut.try_recv() {
            Ok(Some(message)) => {
                // Remove the future from the hashmap.
                self.pending_requests.remove(&id);
                Ok(Some(message))
            }
            Ok(None) => Ok(None),
            Err(error) => Err(XRPLWebSocketException::Canceled(error).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asynch::clients::SingleExecutorMutex;
    use embassy_futures::block_on;

    #[test]
    fn handle_message_routes_response_to_waiting_request() {
        let mut base = WebsocketBase::<SingleExecutorMutex>::new();
        block_on(async {
            base.setup_request_future("1".to_string()).await;
            base.handle_message(r#"{"id":"1","result":{}}"#.to_string())
                .await
                .unwrap();
            let received = base.try_recv_request("1".to_string()).await.unwrap();
            assert_eq!(received, Some(r#"{"id":"1","result":{}}"#.to_string()));
        });
    }

    #[test]
    fn unmatched_message_is_buffered_for_pop() {
        let mut base = WebsocketBase::<SingleExecutorMutex>::new();
        block_on(async {
            // No request is registered, so the message falls through to the
            // general message channel rather than a request receiver.
            base.handle_message(r#"{"result":{}}"#.to_string())
                .await
                .unwrap();
            assert_eq!(base.pop_message().await, r#"{"result":{}}"#.to_string());
        });
    }
}
