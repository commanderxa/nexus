use std::io::{Result, Write};

use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use ansi_term::Color;
use futures::StreamExt;
use nexuslib::{
    models::{message::text::TextMessage, user::User},
    request::{message::MessageRequest, Request, RequestBody},
    utils::{string_to_vec, vec_to_string},
    Message,
};
use rand_core::RngCore;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
};
use tokio_util::codec::{FramedRead, LinesCodec};
use x25519_dalek::{PublicKey, StaticSecret};

pub async fn send_message(
    stream: &mut TcpStream,
    secret: StaticSecret,
    token: String,
    user: User,
    receiver: User,
) -> Result<()> {
    let (reader, writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let stream = tokio::io::stdin();
    let mut lines = FramedRead::new(stream, LinesCodec::new());

    loop {
        let mut buf = String::new();
        let token = token.clone();

        print!("> {}: ", Color::Green.bold().paint("Me"));
        std::io::stdout().flush().unwrap();

        tokio::select! {
            // stream
            result = reader.read_line(&mut buf) => {
                print!("\r");
                let result = result.unwrap();
                if result == 0 {
                    break;
                }
                log::debug!("> {}", buf);
                let buf = buf.replace('\n', "");
                let message: Message<TextMessage> = serde_json::from_str(&buf).unwrap();

                let pub_key: [u8; 32] = receiver.public_key().as_slice().try_into().unwrap();
                let shared_key = secret.diffie_hellman(&PublicKey::from(pub_key));

                let message_nonce = string_to_vec(message.get_nonce());
                let nonce = Nonce::from_slice(message_nonce.as_slice());

                let cipher = Aes256Gcm::new_from_slice(shared_key.as_bytes().as_slice()).unwrap();

                let decrypted = cipher.decrypt(nonce, string_to_vec(message.content.text.clone()).as_ref()).unwrap();
                let msg = String::from_utf8(decrypted).unwrap();

                let display_name = if message.sides.get_sender() == user.uuid {
                    Color::Green.bold().paint("Me".to_owned())
                } else {
                    Color::Red.bold().paint(receiver.username.clone())
                };
                println!("> {}: {}", display_name, Color::Blue.paint(msg));
            }
            // input
            result = lines.next() => {
                let pub_key: [u8; 32] = receiver.public_key().as_slice().try_into().unwrap();
                let shared_key = secret.diffie_hellman(&PublicKey::from(pub_key));

                let message = result.unwrap().unwrap();

                let mut raw_nonce = [0u8; 12];
                rand_core::OsRng.fill_bytes(&mut raw_nonce);
                let nonce = Nonce::from_slice(&raw_nonce);
                let cipher = Aes256Gcm::new_from_slice(shared_key.as_bytes().as_slice()).unwrap();

                let encrypted = cipher.encrypt(nonce, message.as_ref()).unwrap();
                let encrypted = vec_to_string(encrypted);

                let text_message = TextMessage::new(&encrypted);
                let message = Message::new(
                    text_message,
                    nonce.to_vec(),
                    user.uuid,
                    receiver.uuid,
                );

                // Composing the request for registering
                let req_body = MessageRequest::new(message);
                let req = Request::new(req_body.op(), req_body, token);
                let mut req_json = serde_json::to_vec(&req).unwrap();
                // Appending `\n` in the end of the request
                let mut new_line = String::from("\n").as_bytes().to_vec();
                req_json.append(&mut new_line);

                // Sends the Request
                writer.write_all(&req_json).await.unwrap();
                writer.flush().await.unwrap();
            }
        }
    }

    Ok(())
}
