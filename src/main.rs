use reqwest::Client;
use serenity::Http;
use std::{
    collections::VecDeque,
    fs::{self, File},
    io::Write,
};

use ::serenity::all::{
    ChannelId, CreateAttachment, CreateWebhook, EditWebhook, ExecuteWebhook, GetMessages, Message,
    MessageId, Webhook,
};
use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn attachments(
    ctx: Context<'_>,
    #[description = "ID del primer mensaje"] id: String,
) -> Result<(), Error> {
    let mut ids: Vec<MessageId> = Vec::new();
    let builder = GetMessages::new()
        .before(MessageId::new(
            id.parse().expect("Error al parsear input de usuario"),
        ))
        .limit(100);
    let mut messages = ctx.channel_id().messages(ctx, builder).await?;
    while messages.len() > 0 {
        let mut last_id = MessageId::default();
        println!("Count: {}", messages.len());
        for message in messages {
            if !message.attachments.is_empty() {
                println!("{}", message.id.to_string());
                ids.push(message.id);
            }
            last_id = message.id;
        }
        let builder = GetMessages::new().before(last_id).limit(100);
        messages = ctx.channel_id().messages(ctx, builder).await?;
    }
    let path = "ids.txt";

    let channel_id = ctx.channel_id();
    let _ = guardar_ids(ids, channel_id, path);

    Ok(())
}
fn guardar_ids(ids: Vec<MessageId>, channel_id: ChannelId, path: &str) -> Result<(), Error> {
    let mut file = File::create(path)?;
    writeln!(file, "{}", channel_id)?;

    for id in ids {
        println!("{}", id.to_string());
        writeln!(file, "{}", id)?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn start(
    ctx: Context<'_>,
    #[description = "ID del canal de origen"] channel_id_string: String,
) -> Result<(), Error> {
    let directory = "ids.txt";
    let channel_id: ChannelId = ChannelId::new(
        channel_id_string
            .parse()
            .expect("Error al parsear input de usuario"),
    );
    generate_ids(directory, channel_id);
    let ids = fs::read_to_string(directory).expect("file not found");
    let ids: VecDeque<u64> = ids
        .split("\n")
        .filter(|id| !id.is_empty())
        .map(|id| id.parse().expect("Error al convertir a u64"))
        .collect();
    let mut vec: Vec<u64> = ids.into();
    vec.sort();
    let ids = VecDeque::from(vec);

    for message_id in ids.clone() {
        println!("--{message_id}")
    }
    let builder = CreateWebhook::new("Test");
    let mut webhook = ctx.channel_id().create_webhook(ctx, builder).await?;
    for message_id in ids {
        if message_id == channel_id.get() {
            continue;
        }
        println!("{message_id}");
        let message = channel_id
            .message(ctx, message_id)
            .await
            .expect("No se ha encontrado el mensaje");
        let _ = send_as_user(ctx.http(), &mut webhook, &message).await;

        println!("FIN")
    }
    webhook.delete(ctx).await.expect("Error al borrar webhook");
    Ok(())
}

async fn send_as_user(
    http: &Http,
    webhook: &mut Webhook,
    message: &Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Start, send");
    let start = std::time::Instant::now();

    // Crear un webhook en el canal (puedes usar un webhook existente si ya tienes uno)
    let user = &message.author;
    let user_name = user
        .global_name
        .clone()
        .or(Some(user.name.clone()))
        .expect("Error al recibir el nombre de usuario");
    let duration = start.elapsed();
    println!("Usuario en {:?}", duration);

    let user_avatar_url = user.avatar_url().expect("Error al recivir el avatar");
    let duration = start.elapsed();
    println!("Avatar en {:?}", duration);

    let url = &message
        .attachments
        .first()
        .expect("Error al recivir el attachment")
        .url;
    let duration = start.elapsed();
    println!("URL en {:?}", duration);

    let duration = start.elapsed();
    println!("Edit en {:?}", duration);
    let duration = start.elapsed();
    println!("Webhook en {:?}", duration);

    // Ejecutar el webhook con el nombre y avatar del usuario

    let builder = ExecuteWebhook::new()
        .username(user_name.clone())
        .avatar_url(user_avatar_url.clone())
        .add_file(
            CreateAttachment::url(http, url)
                .await
                .expect("Error al añadir archivo a la envia"),
        );
    webhook.execute(http, false, builder).await?;

    let fecha: String = format_date_es(&mut message.timestamp.to_string());
    let builder = ExecuteWebhook::new()
        .username(user_name)
        .avatar_url(user_avatar_url.clone())
        .content(&fecha);
    webhook.execute(http, false, builder).await?;
    println!("Mensaje :{}", &fecha);
    let duration = start.elapsed();
    println!("Envio en {:?}", duration);

    Ok(())
}

fn format_date_es(timestamp: &mut String) -> String {
    // Lista de nombres de meses en español
    let meses = [
        "Enero",
        "Febrero",
        "Marzo",
        "Abril",
        "Mayo",
        "Junio",
        "Julio",
        "Agosto",
        "Septiembre",
        "Octubre",
        "Noviembre",
        "Diciembre",
    ];
    let binding = timestamp.split_at(10).0;
    println!("{binding}");
    let mut fecha = binding.split("-");

    // Extraer día, mes y año
    let anio = fecha.next().expect("Error de formato de fecha");
    let mes: usize = fecha
        .next()
        .expect("Error de formato de fecha")
        .parse()
        .expect("Error al parsear");
    let dia: usize = fecha
        .next()
        .expect("Error de formato de fecha")
        .parse()
        .expect("Error al parsear");

    // Crear el string en formato español
    format!("{} de {}, {}", dia, meses[mes - 1], anio)
}

fn generate_ids(path: &str, channel_id: ChannelId) {
    let mut ids: Vec<MessageId> = Vec::new();
    let dir = fs::read_dir("imagenes").expect("Error al abrir el directorio");
    for archivo in dir {
        let id: u64 = archivo
            .expect("Error al acceder al archivo")
            .file_name()
            .into_string()
            .expect("Error al leer el nombre del archivo")
            .split(".")
            .next()
            .expect("Error al formatear el nombre del archivo")
            .parse()
            .expect("Error al parsear");
        ids.push(MessageId::new(id));
    }

    let _ = guardar_ids(ids, channel_id, path);
}

#[poise::command(slash_command, prefix_command)]
async fn folder(ctx: Context<'_>) -> Result<(), Error> {
    let ids = fs::read_to_string("ids.txt").expect("file not found");
    let mut ids: VecDeque<u64> = ids
        .split("\n")
        .filter(|id| !id.is_empty())
        .map(|id| id.parse().expect("Error al convertir a u64"))
        .collect();
    let channel_id = ChannelId::new(ids.pop_front().expect("Fallo con el id del canal"));

    let directory = "imagenes";
    fs::create_dir_all(directory)?;
    let client = Client::new();

    for message_id in ids {
        let message = channel_id
            .message(ctx, message_id)
            .await
            .expect("No se ha encontrado el mensaje");
        let attachment = &message
            .attachments
            .first()
            .expect("error al encontrar el attachment");

        let file_name = format!(
            "{}/{}.{}",
            directory,
            message.id,
            attachment.filename.split('.').last().unwrap_or("png")
        );

        let response = client.get(&attachment.url).send().await?;
        let bytes = response.bytes().await?;

        let mut file = File::create(file_name)?;
        file.write_all(&bytes)?;
        println!("Archivo descargado: {}", attachment.url);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![attachments(), start(), folder()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
