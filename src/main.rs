use dotenv::dotenv;
use std::env;
use std::future::Future;
use std::{collections::HashMap, error::Error, pin::Pin, sync::Arc};
use twilight_cache_inmemory::{DefaultInMemoryCache, ResourceType};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;

mod commands;
// Alias para el tipo de función de comando
type CommandFn = fn(Message, Arc<HttpClient>, Vec<&str>) -> CommandFuture;
type CommandFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>>;

struct CommandHandler {
    commands: HashMap<&'static str, CommandFn>,
}

impl CommandHandler {
    fn new() -> Self {
        let mut commands = HashMap::new();
        commands.insert("$ping", commands::ping::handle_ping as CommandFn);
        Self { commands }
    }

    async fn handle_message(
        &self,
        msg: Message,
        http: Arc<HttpClient>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if msg.author.bot {
            return Ok(());
        }

        let content = msg.content.clone();
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        let command = parts[0];
        let args = parts[1..].to_vec();

        if !command.starts_with("$") {
            return Ok(());
        }
        if let Some(cmd_fn) = self.commands.get(command) {
            cmd_fn(msg, http, args).await?;
        } else {
            // responder que el comando no existe
            http.create_reaction(
                msg.channel_id,
                msg.id,
                &RequestReactionType::Unicode { name: "❓" },
            )
            .await?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv().ok();
    let mut tokenp = String::new();
    match env::var("token") {
        Ok(token) => {
            tokenp = token;
        }
        Err(e) => {
            println!("error {}", e);
        }
    }
    let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::MESSAGE_CONTENT;

    let mut shard = Shard::new(ShardId::ONE, tokenp.to_string().clone(), intents);
    let http = Arc::new(HttpClient::new(tokenp.to_string()));

    let cache = DefaultInMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    let command_handler = Arc::new(CommandHandler::new());

    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error recibiendo evento: {:?}", e);
                continue;
            }
        };

        cache.update(&event);

        let http_clone = Arc::clone(&http);
        let handler_clone = Arc::clone(&command_handler);

        tokio::spawn(async move {
            if let Event::MessageCreate(msg_box) = event {
                let msg: Message = msg_box.0;
                if let Err(e) = handler_clone.handle_message(msg, http_clone).await {
                    eprintln!("Error manejando comando: {:?}", e);
                }
            }
        });
    }
    Ok(())
}
