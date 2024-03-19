use std::{
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
use tokio::{net::UdpSocket, sync::Mutex};

use crate::state::connection::ConnectionState;

use super::udp_handler::handle_udp;

pub async fn run_udp(state: Arc<Mutex<ConnectionState>>) {
    let socket = udp_socket_setup().await.unwrap();

    tokio::spawn(async move {
        match handle_udp(socket, state).await {
            Ok(_) => (),
            Err(_) => {
                log::error!("Error handling request");
            }
        }
    });
}

async fn udp_socket_setup() -> Result<UdpSocket, ()> {
    let mut stdout = stdout();
    execute!(stdout, cursor::Hide).unwrap();

    let action = String::from("UDP Server ");
    let action_len = action.len() as u16;

    execute!(
        stdout,
        SetAttribute(crossterm::style::Attribute::Bold),
        Print(action),
        SetAttribute(crossterm::style::Attribute::Reset),
        SetForegroundColor(Color::Yellow),
        Print("\tstarting"),
        SavePosition
    )
    .unwrap();

    let start_time = Instant::now();
    let mut udp_socket = _udp_socket_setup().await;
    let duration = Duration::from_secs(15);

    while udp_socket.is_err() {
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
            log::error!("Exiting, due to: {err}", err = udp_socket.unwrap_err());
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
        udp_socket = _udp_socket_setup().await;
    }

    execute!(
        stdout,
        MoveToColumn(action_len),
        Clear(ClearType::UntilNewLine),
        MoveToColumn(action_len),
        SetForegroundColor(Color::Green),
        Print("\tstarted"),
        ResetColor,
        Print(format!("\t\tat `{}`\n", std::env::var("UDP_ADDR").unwrap())),
        cursor::Show
    )
    .unwrap();

    // log::info!(
    //     "UDP server set up at `{}`",
    //     std::env::var("C_ADDR").unwrap_or_else(|_| "127.0.0.1:8083".to_owned())
    // );

    Ok(udp_socket.unwrap())
}

async fn _udp_socket_setup() -> Result<UdpSocket, std::io::Error> {
    UdpSocket::bind(std::env::var("UDP_ADDR").unwrap()).await
}
