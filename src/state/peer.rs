use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_util::codec::{Framed, LinesCodec};
use uuid::Uuid;

pub struct Peer {
    pub lines: Framed<TcpStream, LinesCodec>,
    pub rx: UnboundedReceiver<String>,
    pub user_uuid: Uuid,
    pub token: String,
}

impl Peer {
    pub fn new(
        lines: Framed<TcpStream, LinesCodec>,
        user_uuid: Uuid,
        token: String,
    ) -> (Self, UnboundedSender<String>) {
        let (tx, rx) = mpsc::unbounded_channel();

        (
            Self {
                lines,
                user_uuid,
                token,
                rx,
            },
            tx,
        )
    }
}
