use std::{
    io::stdout,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{self, MoveToColumn, RestorePosition, SavePosition},
    execute,
    style::{Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use scylla::Session;

pub mod database;
mod db_queries;
pub mod models_wrapper;

pub async fn session_setup() -> Session {
    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let mut session = _session_setup(&uri).await;

    let mut stdout = stdout();
    execute!(stdout, cursor::Hide).unwrap();

    let action = String::from("DB session ");
    let action_len = action.len() as u16;

    execute!(
        stdout,
        SetAttribute(crossterm::style::Attribute::Bold),
        Print(action),
        SetAttribute(crossterm::style::Attribute::Reset),
        SetForegroundColor(Color::Yellow),
        Print("\tconnecting")
    )
    .unwrap();

    execute!(stdout, SavePosition).unwrap();

    let duration = Duration::from_secs(5);
    let mut dots = 0;

    let mut start_time = Instant::now();
    while session.is_err() {
        if dots > 3 {
            execute!(
                stdout,
                RestorePosition,
                Clear(crossterm::terminal::ClearType::FromCursorDown)
            )
            .unwrap();
            dots = 0;
        }
        std::thread::sleep(Duration::from_millis(750));
        execute!(stdout, Print(".")).unwrap();
        dots += 1;

        if Instant::now().duration_since(start_time) >= duration {
            session = _session_setup(&uri).await;
            start_time = Instant::now();
        }
    }

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

    session.unwrap()
}

async fn _session_setup(uri: &str) -> Result<Session, ()> {
    let session = database::create_session(uri).await;
    if let Ok(s) = session {
        database::initialize(&s).await.unwrap();
        Ok(s)
    } else {
        Err(())
    }
}
