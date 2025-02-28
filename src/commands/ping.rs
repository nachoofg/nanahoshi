use std::future::Future;
use std::{error::Error, pin::Pin, sync::Arc};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

pub fn handle_ping(
    msg: Message,
    http: Arc<HttpClient>,
    _args: Vec<&str>,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>> {
    Box::pin(async move {
        http.create_message(msg.channel_id).content("pong").await?;
        Ok(())
    })
}
