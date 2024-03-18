use std::{
    env,
    io::stdout,
    process,
    sync::Arc,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{self, MoveToColumn, RestorePosition, SavePosition},
    execute,
    style::{Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use scylla::Session;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{handler::handle_stream, state::connection::ConnectionState};

pub async fn run_tcp(session: Arc<Mutex<Session>>, state: Arc<Mutex<ConnectionState>>) {
    let listener = tcp_listener_setup().await.unwrap();

    // TCP Listener
    while let Ok((stream, socket)) = listener.accept().await {
        let session: Arc<Mutex<Session>> = session.clone();
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            match handle_stream(stream, socket, session, state).await {
                Ok(_) => (),
                Err(_) => {
                    log::error!("Error handling request");
                }
            }
        });
    }

    // TCP TLS Listener
    // let config = get_tls_config()?;
    // let acceptor = TlsAcceptor::from(Arc::new(config));
    // loop {
    //     while let Ok((stream, _socket)) = listener.accept().await {
    //         // let acceptor = acceptor.clone();
    //         let session: Arc<Mutex<Session>> = session_copy.clone();
    //         let fut = async move {
    //             let mut tls_stream = acceptor.accept(stream).await?;
    //             match handle_stream(tls_stream, session, state).await {
    //                 Ok(_) => (),
    //                 Err(_) => panic!("Error handling request."),
    //             }
    //             Ok(()) as Result<()>
    //         };
    //         tokio::spawn(async move {
    //             if let Err(err) = fut.await {
    //                 log::error!("{:?}", err);
    //                 panic!("Error handling request.");
    //             }
    //         });
    //     }
    // }
}

async fn tcp_listener_setup() -> Result<TcpListener, ()> {
    let mut stdout = stdout();
    execute!(stdout, cursor::Hide).unwrap();

    let action = String::from("TCP Server ");
    let action_len = action.len() as u16;

    execute!(
        stdout,
        SetAttribute(crossterm::style::Attribute::Bold),
        Print(action),
        SetAttribute(crossterm::style::Attribute::Reset),
        SetForegroundColor(Color::Yellow),
        Print("\tstarting")
    )
    .unwrap();

    execute!(stdout, SavePosition).unwrap();

    let start_time = Instant::now();
    let mut tcp_listener = _tcp_listener_setup().await;
    let duration = Duration::from_secs(15);

    while tcp_listener.is_err() {
        if Instant::now().duration_since(start_time) > duration {
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

            log::error!("Exiting, due to: {err}", err = tcp_listener.unwrap_err());
            process::exit(1);
        }

        let mut dots = 0;

        let _start_time = Instant::now();

        while Instant::now().duration_since(start_time) < Duration::from_secs(3) {
            if dots > 3 {
                execute!(
                    stdout,
                    RestorePosition,
                    Clear(crossterm::terminal::ClearType::UntilNewLine)
                )
                .unwrap();
                dots = 0;
            }
            std::thread::sleep(Duration::from_millis(750));
            execute!(stdout, Print(".")).unwrap();
            dots += 1;
        }
        tcp_listener = _tcp_listener_setup().await;
    }

    execute!(
        stdout,
        MoveToColumn(action_len),
        Clear(ClearType::UntilNewLine),
        MoveToColumn(action_len),
        SetForegroundColor(Color::Green),
        Print("\tstarted"),
        ResetColor,
        Print(format!("\t\tat `{}`\n", std::env::var("ADDR").unwrap())),
        cursor::Show,
    )
    .unwrap();

    // log::info!(
    //     "TCP server listener set up at `{}`",
    //     std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_owned())
    // );

    Ok(tcp_listener.unwrap())
}

async fn _tcp_listener_setup() -> Result<TcpListener, std::io::Error> {
    TcpListener::bind(env::var("ADDR").unwrap()).await
}
