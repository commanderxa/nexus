use std::{error::Error, fs, path::Path, sync::Arc};

use chrono::Duration;
use futures::StreamExt;
use scylla::{
    frame::value::Timestamp, prepared_statement::PreparedStatement, QueryResult, Session,
};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};
use tokio_util::codec::{BytesCodec, Framed};
use uuid::Uuid;

use nexuslib::{
    models::file::{media_file::MediaFile, FileContent},
    request::{file::FileRequest, Request},
};

use crate::{errors::db::DbError, state::connection::ConnectionState};

pub async fn stream_file(
    stream: TcpStream,
    file: String,
    session: Arc<Mutex<scylla::Session>>,
    state: Arc<Mutex<ConnectionState>>,
    peer_uuid: Uuid,
) -> Result<Framed<TcpStream, BytesCodec>, Box<dyn Error>> {
    let file_request: Request<FileRequest> = serde_json::from_str(&file).unwrap();

    let mut bytes = Framed::new(stream, BytesCodec::new());

    let file = file_request.body.file;
    let filename_split = file.name.split('.').collect::<Vec<&str>>();
    let ext = filename_split[filename_split.len() - 1];
    let object_name = file.uuid.to_string() + "." + ext;
    let filename = Path::new(&std::env::var("STORAGE_MEDIA").unwrap()).join(&object_name);

    fs::create_dir_all(Path::new(&std::env::var("STORAGE_MEDIA").unwrap())).unwrap();

    let mut written_file = tokio::fs::File::create(&filename).await.unwrap();

    while let Some(result) = bytes.next().await {
        match result {
            Ok(chunk) => {
                if let Err(e) = written_file.write_all(&chunk).await {
                    eprintln!("Error writing to file: {:?}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        }
    }

    // storage
    //     .upload_object(
    //         &mut UploadObjectArgs::new(
    //             &file.media_type.to_string().to_lowercase(),
    //             &object_name,
    //             &filename.to_str().unwrap(),
    //         )
    //         .unwrap(),
    //     )
    //     .await
    //     .unwrap();

    if add_file(session, &file, &object_name, "", peer_uuid)
        .await
        .is_err()
    {
        log::error!("Error adding message to the DB!");
    }

    // check if the file is secret
    // if !file_request.body.file.secret {
    //     // if it is not a secret and
    //     if file_request.body.index == IndexToken::Start {
    //         // if this is an initial file request
    //         // => add to the DB
    //         if add_file(session, &file_request.body.file).await.is_err() {
    //             log::error!("Error adding file to the DB!");
    //         }
    //     }
    // }

    for peer in state
        .lock()
        .await
        .peers
        .get_mut(&file.sender)
        .unwrap()
        .iter_mut()
    {
        if peer.0 == &peer_uuid {
            // sending the call to the sender session
            // let _ = peer.1.tcp_sender.send(call_str.clone());
        }
    }

    Ok(bytes)
}

/// Add file to the DB
pub async fn add_file(
    session: Arc<Mutex<Session>>,
    file: &MediaFile,
    filename: &str,
    filepath: &str,
    sender: Uuid,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare(
            "
            INSERT INTO nexus.media 
            (uuid, name, path, sender, type, created_at) 
            VALUES(?, ?, ?, ?, ?, ?);
        ",
        )
        .await
        .unwrap();

    match session
        .lock()
        .await
        .execute(
            &prepared,
            (
                file.uuid,
                filename,
                filepath,
                sender,
                file.get_type().unwrap().get_index() as i8,
                Timestamp(Duration::try_seconds(file.get_created_at().timestamp()).unwrap()),
            ),
        )
        .await
    {
        Ok(result) => Ok(result),
        Err(_e) => {
            log::debug!("{_e:?}");
            Err(DbError::FailedToAdd)
        }
    }
}
