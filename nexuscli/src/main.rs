use std::{io::Write, net::SocketAddr};

use env_logger::Env;
use futures::StreamExt;
use ops::start_session::start_session;

use reqwest::Client;
use sysinfo::{System, SystemExt};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpStream, UdpSocket},
};

use nexuslib::{
    models::{
        call::media_call::MediaCall, command::Command, file::media_file::MediaFile,
        message::media::MediaType, user::User,
    },
    request::{
        auth::{AuthRequest, AuthRequestMeta},
        call::CallRequest,
        file::FileRequest,
        index_token::IndexToken,
        EmptyRequestBody, Request, RequestBody,
    },
    response::auth::AuthResponse,
};
use tokio_util::codec::{FramedRead, LinesCodec};
use uuid::Uuid;
use x25519_dalek::StaticSecret;

use crate::ops::{send_message::send_message, user::get_users};

mod ops;

#[tokio::main]
async fn main() {
    // Logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let remote_addr: SocketAddr = "127.0.0.1:8083".parse().unwrap();

    // use the same port as for tcp
    // let local_addr: SocketAddr = if remote_addr.is_ipv4() {
    //     "127.0.0.1:0"
    // } else {
    //     "[::]:0"
    // }
    // .parse()
    // .unwrap();

    let mut stream = TcpStream::connect("127.0.0.1:8081")
        .await
        .expect("Not connected");

    let socket = UdpSocket::bind(stream.local_addr().unwrap()).await.unwrap();
    // const MAX_DATAGRAM_SIZE: usize = 65_000;
    socket.connect(&remote_addr).await.unwrap();

    let command: Command = Command::File;

    let mut sys = System::new();
    sys.refresh_system();

    // login
    let mut username = String::from("");
    print!("Username: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut username).unwrap();
    let username = username.replace("\n", "");
    let password = rpassword::prompt_password("Password: ").unwrap();
    let auth_req = AuthRequest {
        username: username,
        password: password,
        meta: AuthRequestMeta {
            location: "California".to_owned(),
            device_name: "Asus".to_owned(),
            device_type: "Laptop".to_owned(),
            device_os: sys.long_os_version().unwrap(),
        },
    };

    let auth_req_json = serde_json::to_string(&auth_req).unwrap();

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .tls_sni(false)
        .build()
        .unwrap();

    let resp = client
        .post(format!("https://127.0.0.1:8082/api/auth/login"))
        .body(auth_req_json)
        .send()
        .await
        .unwrap()
        .json::<AuthResponse>()
        .await
        .unwrap();

    // starting a tcp session with the server
    let start_req: Request<EmptyRequestBody> =
        Request::new(command, EmptyRequestBody {}, resp.token.to_owned());
    start_session(&mut stream, start_req).await.unwrap();

    let user = client
        .get(format!("https://127.0.0.1:8082/api/users/{}", resp.uuid))
        .bearer_auth(&resp.token)
        .send()
        .await
        .unwrap()
        .json::<User>()
        .await
        .unwrap();

    log::info!("Logged in as: {}", user.username);

    // get the secret key
    let secret = client
        .post(format!(
            "https://127.0.0.1:8082/api/users/key/{}",
            user.uuid
        ))
        .bearer_auth(&resp.token)
        .send()
        .await
        .unwrap()
        .json::<Vec<u8>>()
        .await
        .unwrap();
    let secret = slice_to_arr(secret.as_slice());
    let secret = StaticSecret::from(secret);

    let users = get_users(client, resp.token.clone()).await.unwrap();
    let receiver = users
        .iter()
        .filter(|x| x.username != user.username)
        .collect::<Vec<_>>()[0]
        .clone();

    match command {
        Command::Message => {
            send_message(&mut stream, secret, resp.token.to_owned(), user, receiver)
                .await
                .unwrap();
        }
        Command::Call => {
            let (reader, writer) = stream.split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            let stream = tokio::io::stdin();
            let mut lines = FramedRead::new(stream, LinesCodec::new());

            let mut call_stack: Vec<MediaCall> = Vec::new();

            // let (tx, mut rx) = mpsc::channel::<(MediaCall, u32)>(1_000);

            loop {
                let mut buf = String::new();
                let token = resp.token.clone();

                tokio::select! {
                    result = reader.read_line(&mut buf) => {
                        let _result = result.unwrap();

                        let call_req: CallRequest<MediaCall> = serde_json::from_str(&buf).unwrap();
                        let req_act = &call_req.index;

                        println!("Received: {req_act:#?}");

                        let call = call_req.call;

                        call_stack.push(call);

                        if call_req.index == IndexToken::Accept {
                            let message = "Hello!".as_bytes().to_vec();
                            let data = MediaCall::new(user.uuid, receiver.uuid, message, vec![], false);
                            let data = bincode::serialize(&data).unwrap();

                            socket.send(&data).await.unwrap();
                        }
                    }
                    result = lines.next() => {
                        let _result = result.unwrap().unwrap().clone();
                        if _result.contains("y") {
                            let mut call = call_stack.last().unwrap().clone();
                            call.accepted = true;
                            let call_req = CallRequest::new(call, IndexToken::Accept);
                            let req = Request::new(call_req.op(), call_req, token);

                            let req_act = &req.body.index;
                            println!("Sending: {req_act:#?}");
                            let mut req_json = serde_json::to_vec(&req).unwrap();
                            // Appending `\n` in the end of the request
                            let mut new_line = String::from("\n").as_bytes().to_vec();
                            req_json.append(&mut new_line);

                            // Sends the Request
                            writer.write_all(&req_json).await.unwrap();
                            writer.flush().await.unwrap();
                        } else if _result.contains("c") {
                            let mut call = call_stack.pop().unwrap();
                            call.accepted = true;
                            let call_req = CallRequest::new(call, IndexToken::End);
                            let req = Request::new(call_req.op(), call_req, token);

                            let req_act = &req.body.index;
                            println!("Sending: {req_act:#?}");
                            let mut req_json = serde_json::to_vec(&req).unwrap();
                            // Appending `\n` in the end of the request
                            let mut new_line = String::from("\n").as_bytes().to_vec();
                            req_json.append(&mut new_line);

                            // Sends the Request
                            writer.write_all(&req_json).await.unwrap();
                            writer.flush().await.unwrap();
                        } else {
                            let call = MediaCall::new(user.uuid, receiver.uuid, vec![], vec![], false);
                            let req_body = CallRequest::new(call, IndexToken::Start);

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
            }
        }
        Command::File => {
            let (_, writer) = stream.split();
            // let reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);
            // let framed = FramedRead::new(reader, BytesCodec::new());

            let mut file = File::open("/home/spectre/Pictures/picture.png")
                .await
                .unwrap();
            let metadata = file.metadata().await.unwrap();
            let file_size = metadata.len();

            let f = MediaFile::new(
                Uuid::new_v4(),
                file_size as usize,
                (file_size as f64 / 1024.0).ceil() as usize,
                "picture.png".to_owned(),
                MediaType::Image,
                false,
                user.uuid,
            );
            let req_body = FileRequest::new(f);
            let req = Request::new(req_body.op(), req_body, resp.token);
            let mut req_json = serde_json::to_vec(&req).unwrap();
            // Appending `\n` in the end of the request
            let mut new_line = String::from("\n").as_bytes().to_vec();
            req_json.append(&mut new_line);

            // Sends the Request
            writer.write_all(&req_json).await.unwrap();
            writer.flush().await.unwrap();

            let mut buffer = vec![0; 1024];
            let mut bytes_sent: u64 = 0;

            while bytes_sent < file_size {
                let bytes_read = file.read(&mut buffer).await.unwrap();
                writer.write_all(&buffer[..bytes_read]).await.unwrap();
                bytes_sent += bytes_read as u64;
            }
        }
    }
}

pub fn slice_to_arr(slice: &[u8]) -> [u8; 32] {
    slice.try_into().expect("Wrong slice length")
}
