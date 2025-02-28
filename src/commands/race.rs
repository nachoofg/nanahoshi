use rand::Rng;
use std::future::Future;
use std::{error::Error, pin::Pin, sync::Arc};
use tokio::time::{self, Duration};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::EmbedBuilder;

pub fn handle_race(
    msg: Message,
    http: Arc<HttpClient>,
    _args: Vec<&str>,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>> {
    Box::pin(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        let val: Vec<&str> = msg.content.split(" ").skip(1).collect();
        http.create_message(msg.channel_id).content(val[0]).await?;
        let e1: &str = val[0];
        let mut pe1: i8 = 0;
        let e2: &str = val[1];
        let mut pe2: i8 = 0;
        let mut embed1 = EmbedBuilder::new()
            .title(format!("equipo {e1} vs equipo {e2}"))
            .description(format!("puntuacion: {pe1}\npuntuacion:{pe2}"))
            .validate()?
            .build();
        let mensaje = http
            .create_message(msg.channel_id)
            .embeds(&[embed1])
            .await?
            .model()
            .await?;
        while pe1 < 20 && pe2 < 20 {
            interval.tick().await;
            let randomnumero: i8 = rand::thread_rng().gen_range(-2..4);
            let porcentaje: i8 = rand::thread_rng().gen_range(1..2);
            let result: &String = &randomnumero.to_string();
            if randomnumero / porcentaje == 1 {
                pe1 += randomnumero;
            } else {
                pe2 += randomnumero;
            }
            embed1 = EmbedBuilder::new()
                .title(format!("equipo {e1} vs equipo {e2}"))
                .description(format!("puntuacion: {pe1}\npuntuacion:{pe2}"))
                .validate()?
                .build();
            http.update_message(msg.channel_id, mensaje.id)
                .embeds(Some(&[embed1]))
                .await?
                .model()
                .await?;
        }
        Ok(())
    })
}
