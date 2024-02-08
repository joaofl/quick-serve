use crate::ui::window::UI;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Logger {
    async fn logger(self: Arc<Self>);
}

#[async_trait]
impl Logger for UI {
    async fn logger(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut receiver = self.sender.subscribe();

            loop {
                let m = receiver.recv().await.unwrap();
                print!("{m}");
            };
        });
    }
}
