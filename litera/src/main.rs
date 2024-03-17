use std::{io::Write, net::SocketAddr};

use env_logger::Env;
use futures::StreamExt;
use ops::start_session::start_session;

use reqwest::Client;
use sysinfo::{System, SystemExt};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpStream, UdpSocket},
    sync::mpsc,
};

use orbis::{
    models::{calls::audio_call::AudioCall, command::Command, user::user::User},
    request::{
        auth::{AuthRequest, AuthRequestMeta},
        call::CallRequest,
        EmptyRequestBody, IndexToken, Request, RequestBody,
    },
    response::auth::AuthResponse,
};
use tokio_util::codec::{FramedRead, LinesCodec};
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
    const MAX_DATAGRAM_SIZE: usize = 65_000;
    socket.connect(&remote_addr).await.unwrap();

    let command: Command = Command::Message;
    
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
        Command::AudioCall => {
            let (reader, writer) = stream.split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            let stream = tokio::io::stdin();
            let mut lines = FramedRead::new(stream, LinesCodec::new());

            let mut call_stack: Vec<AudioCall> = Vec::new();

            let (tx, mut rx) = mpsc::channel::<(AudioCall, u32)>(1_000);

            loop {
                let mut buf = String::new();
                let token = resp.token.clone();

                tokio::select! {
                    result = reader.read_line(&mut buf) => {
                        let _result = result.unwrap();

                        let call_req: CallRequest<AudioCall> = serde_json::from_str(&buf).unwrap();
                        let req_act = &call_req.index;

                        println!("Received: {req_act:#?}");

                        let call = call_req.call;

                        call_stack.push(call);

                        if call_req.index == IndexToken::Accept {
                            let message = "Hello!".as_bytes().to_vec();
                            let data = AudioCall::new(user.uuid, receiver.uuid, message, vec![], false);
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
                            let call = AudioCall::new(user.uuid, receiver.uuid, vec![], vec![], false);
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
        Command::VideoCall => todo!(),
    }
}

pub fn slice_to_arr(slice: &[u8]) -> [u8; 32] {
    slice.try_into().expect("Wrong slice length")
}
