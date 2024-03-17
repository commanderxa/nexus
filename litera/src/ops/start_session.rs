use std::io::Result;

use orbis::request::{EmptyRequestBody, Request};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
};

pub async fn start_session(stream: &mut TcpStream, req: Request<EmptyRequestBody>) -> Result<()> {
    let (reader, writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    // Composing the request for session commence
    let mut req_json = serde_json::to_vec(&req).unwrap();
    // Appending `\n` in the end of the request
    let mut new_line = String::from("\n").as_bytes().to_vec();
    req_json.append(&mut new_line);

    // Sends the Request
    writer.write_all(&req_json).await.unwrap();
    writer.flush().await.unwrap();

    // Reads the answer
    let mut buf = String::new();
    let _read = reader.read_line(&mut buf).await.unwrap();
    println!("{buf}");

    Ok(())
}
