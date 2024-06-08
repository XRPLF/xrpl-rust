use alloc::string::{String, ToString};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel::Channel};
use futures::channel::oneshot::{self, Receiver, Sender};
use hashbrown::HashMap;

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
}

pub(crate) trait MessageHandler {
    /// Setup an empty future for a request.
    async fn setup_request_future(&mut self, id: String);
    async fn handle_message(&mut self, message: String);
    async fn pop_message(&mut self) -> String;
    async fn request_impl(&mut self, id: String) -> String;
}

impl<M> MessageHandler for WebsocketBase<M>
where
    M: RawMutex,
{
    async fn setup_request_future(&mut self, id: String) {
        let (sender, receiver) = oneshot::channel::<String>();
        self.pending_requests.insert(id.clone(), receiver);
        self.request_senders.insert(id, sender);
    }

    async fn handle_message(&mut self, message: String) {
        let message_value = serde_json::to_value(&message).unwrap();
        let id = match message_value.get("id") {
            Some(id) => {
                let id = id.as_str().unwrap().to_string();
                if id == String::new() {
                    todo!("`id` must not be an empty string")
                }
                id
            }
            None => String::new(),
        };
        if let Some(_receiver) = self.pending_requests.get(&id) {
            let sender = self.request_senders.remove(&id).unwrap();
            sender.send(message).unwrap();
        } else {
            self.messages.send(message).await;
        }
    }

    async fn pop_message(&mut self) -> String {
        self.messages.receive().await
    }

    async fn request_impl(&mut self, id: String) -> String {
        self.setup_request_future(id.clone()).await;
        let fut = self.pending_requests.remove(&id).unwrap();
        let message = fut.await.unwrap();
        serde_json::from_str(&message).unwrap()
    }
}
