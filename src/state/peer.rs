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
    pub peer_uuid: Uuid,
}

impl Peer {
    pub fn new(
        lines: Framed<TcpStream, LinesCodec>,
        user_uuid: Uuid,
        peer_uuid: Uuid,
    ) -> (Self, UnboundedSender<String>) {
        let (tx, rx) = mpsc::unbounded_channel();

        (
            Self {
                lines,
                user_uuid,
                peer_uuid,
                rx,
            },
            tx,
        )
    }
}
