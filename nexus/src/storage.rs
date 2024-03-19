use std::{io::stdout, process};

use crossterm::{
    cursor::{self, MoveToColumn, SavePosition},
    execute,
    style::{Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use minio::s3::{
    args::{BucketExistsArgs, MakeBucketArgs},
    client::Client,
    creds::StaticProvider,
    error::Error,
    http::BaseUrl,
};
use nexuslib::models::message::media::MediaType;

pub fn get_client() -> Result<Client, Box<Error>> {
    let host = std::env::var("MINIO_HOST").unwrap();
    let port = std::env::var("MINIO_PORT").unwrap();
    let endpoint = format!("http://{host}:{port}").parse::<BaseUrl>().unwrap();

    let provider = StaticProvider::new(
        &std::env::var("MINIO_ROOT_USER").unwrap(),
        &std::env::var("MINIO_ROOT_PASSWORD").unwrap(),
        None,
    );

    let client = Client::new(endpoint, Some(Box::new(provider)), None, None);
    if let Err(err) = client {
        return Err(Box::new(err));
    }
    Ok(client.unwrap())
}

/// Performs connection to MinIO
///
/// Calls initializations inside to create mandatory buckets (folders)
pub async fn minio_setup() {
    let mut stdout = stdout();
    execute!(stdout, cursor::Hide).unwrap();

    let action = String::from("MinIO session ");
    let action_len = action.len() as u16;

    execute!(
        stdout,
        SetAttribute(crossterm::style::Attribute::Bold),
        Print(action),
        SetAttribute(crossterm::style::Attribute::Reset),
        SetForegroundColor(Color::Yellow),
        Print("\tconnecting"),
        SavePosition
    )
    .unwrap();

    let _client = get_client();
    if let Ok(client) = _client {
        // create buckets
        minio_init(&client).await;
        execute!(
            stdout,
            MoveToColumn(action_len),
            Clear(ClearType::UntilNewLine),
            MoveToColumn(action_len),
            SetForegroundColor(Color::Green),
            Print("\tconnected\n"),
            ResetColor,
            cursor::Show
        )
        .unwrap();
    } else {
        execute!(
            stdout,
            MoveToColumn(action_len),
            Clear(ClearType::UntilNewLine),
            MoveToColumn(action_len),
            SetForegroundColor(Color::Red),
            Print("\tfailed\n"),
            ResetColor,
            cursor::Show
        )
        .unwrap();
        log::error!("Exiting, due to: {err}", err = _client.unwrap_err());
        process::exit(1);
    }
}

/// Necessary initialization for MinIO
pub async fn minio_init(client: &Client) {
    let buckets = MediaType::str_variants_vec();

    for mut bucket in buckets {
        bucket.push('s');
        if !client
            .bucket_exists(&BucketExistsArgs::new(&bucket).unwrap())
            .await
            .unwrap()
        {
            let res = client
                .make_bucket(&MakeBucketArgs::new(&bucket).unwrap())
                .await;
            if res.is_err() {
                log::error!(
                    "Error while creating a bucket `{bucket}`: {err}",
                    err = res.unwrap_err()
                );
            }
        }
    }
}
