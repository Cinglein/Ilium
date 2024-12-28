use crate::{
    data::UserData,
    send::{SendFrame, Sender},
    time::Ping,
};
use axum::{
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use fastwebsockets::{
    upgrade, FragmentCollectorRead, Frame, OpCode, Payload, WebSocketError, WebSocketWrite,
};
use session::{msg::Msg, queue::Queue, token::ClientToken};
use tokio::time::{sleep, Duration, Instant};

async fn parse_message<Q: Queue, U: UserData>(
    msg: &[u8],
    ip: std::net::SocketAddr,
    sender: &Sender<Q, U>,
    send_frame: SendFrame,
    ping: Ping,
) -> Option<ClientToken> {
    match postcard::from_bytes::<Msg<Q>>(msg) {
        Ok(msg) => {
            let token = msg.token;
            if let Err(e) = sender.send(msg, ip, send_frame, ping).await {
                leptos::logging::log!("error sending signal for {ip:?}: {e:?}");
            }
            Some(token)
        }
        Err(e) => {
            leptos::logging::log!("error parsing message for {ip:?}: {e:?}");
            None
        }
    }
}

async fn read<F: tokio::io::AsyncRead + Unpin, Q: Queue, U: UserData>(
    mut ws: FragmentCollectorRead<F>,
    token: &mut Option<ClientToken>,
    ip: std::net::SocketAddr,
    sender: Sender<Q, U>,
    send_frame: &SendFrame,
    recv_ts: tokio::sync::watch::Receiver<Option<Instant>>,
    send_ping: tokio::sync::watch::Sender<Option<u128>>,
    recv_ping: tokio::sync::watch::Receiver<Option<u128>>,
) -> eyre::Result<()> {
    let ping = Ping(recv_ping);
    loop {
        let mut frame = ws
            .read_frame::<_, WebSocketError>(&mut move |frame| async {
                // for handling obligated sends
                send_frame.send_raw(frame);
                Ok(())
            })
            .await?;
        match frame.opcode {
            OpCode::Close => break,
            OpCode::Binary => {
                *token = parse_message(
                    frame.payload.to_mut(),
                    ip,
                    &sender,
                    send_frame.clone(),
                    ping.clone(),
                )
                .await;
            }
            OpCode::Pong => match *recv_ts.borrow() {
                Some(ts) => {
                    let elapsed = Instant::now().duration_since(ts).as_millis();
                    send_ping.send(Some(elapsed))?;
                }
                _ => (),
            },
            _ => {}
        }
    }
    Ok(())
}

async fn write<S: tokio::io::AsyncWrite + Unpin>(
    mut ws: WebSocketWrite<S>,
    receive_frame: kanal::AsyncReceiver<Frame<'static>>,
) -> Result<(), WebSocketError> {
    while let Ok(frame) = receive_frame.recv().await {
        ws.write_frame(frame).await?;
    }
    Ok(())
}

async fn handle_client<Q: Queue, U: UserData>(
    fut: upgrade::UpgradeFut,
    sender: Sender<Q, U>,
    addr: std::net::SocketAddr,
) -> eyre::Result<()> {
    let (send_frame, receive_frame) = kanal::bounded::<Frame>(100);
    let send_heartbeat = send_frame.clone_async();
    let (send_ts, recv_ts) = tokio::sync::watch::channel(None);
    let (send_ping, recv_ping) = tokio::sync::watch::channel(None);
    let send_frame = SendFrame::new(send_frame);
    let ws = fut.await?;
    let (ws_read, ws_write) = ws.split(tokio::io::split);
    let ws_read = FragmentCollectorRead::new(ws_read);
    let handle = tokio::task::spawn(async move {
        if let Err(e) = write(ws_write, receive_frame.to_async()).await {
            leptos::logging::log!("Error in websocket connection: {e}");
        }
    });
    let heartbeat = tokio::task::spawn(async move {
        while let Ok(_) = {
            let frame = Frame::new(true, OpCode::Ping, None, Payload::Owned(Vec::new()));
            send_ts.send(Some(Instant::now()))?;
            send_heartbeat.send(frame).await?;
            Ok::<_, eyre::Report>(())
        } {
            sleep(Duration::from_secs(3)).await;
        }
        Ok::<_, eyre::Report>(())
    });
    let mut token: Option<ClientToken> = None;
    let res = read(
        ws_read,
        &mut token,
        addr,
        sender.clone(),
        &send_frame,
        recv_ts,
        send_ping,
        recv_ping.clone(),
    )
    .await;
    if let Some(token) = token {
        let ping = Ping(recv_ping);
        if let Err(err) = sender.send(Msg::leave(token), addr, send_frame, ping).await {
            leptos::logging::log!("Error sending leave message: {err:?}");
        }
    }
    heartbeat.abort();
    handle.abort();
    res
}

pub async fn ws_handler<Q: Queue, U: UserData>(
    State(sender): State<Sender<Q, U>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    ws: upgrade::IncomingUpgrade,
) -> impl IntoResponse {
    let (response, fut) = ws.upgrade().unwrap();
    tokio::task::spawn(async move {
        if let Err(e) = handle_client(fut, sender, addr).await {
            leptos::logging::log!("Error in websocket connection: {e}");
        }
    });
    response
}
