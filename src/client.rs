use zeromq::{prelude::*, ZmqMessage};

static ADDR: &str = "tcp://127.0.0.1:24042";
const MAX_CLIENTS: usize = 10;

/// Query a server listening to address `addr` with request `req`.
#[tracing::instrument]
async fn query_server<T: AsRef<str> + std::fmt::Debug>(
    addr: T,
    req: zeromq::ZmqMessage,
) -> Result<zeromq::ZmqMessage, String> {
    let mut socket = zeromq::ReqSocket::new();

    socket
        .connect(addr.as_ref())
        .await
        .map_err(|e| format!("Unable to bind {}: {:?}", addr.as_ref(), e))?;

    socket
        .send(req)
        .await
        .map_err(|e| return format!("Unable to sent the message: {}", e))?;

    let recv_reply: zeromq::ZmqMessage = socket
        .recv()
        .await
        .map_err(|e| return format!("Unable to receive the message: {}", e))?;

    socket.close().await;

    Ok(recv_reply)
}

#[tokio::main]
async fn main() {
    console_subscriber::ConsoleLayer::builder()
        .retention(tokio::time::Duration::from_secs(30))
        .server_addr(([127, 0, 0, 1], 5556))
        .init();

    let mut total = 0;
    for e in (0..MAX_CLIENTS).map(|idx| async move {
        let req = ZmqMessage::from(Vec::from(idx.to_le_bytes()));
        let mut res = query_server(&ADDR, req).await.unwrap().into_vec();
        assert_eq!(res.len(), 1);
        let res: bytes::Bytes = res.pop().unwrap();
        let res: Vec<u8> = res.into_iter().collect();
        let mut arr: [u8; 8] = [0; 8];
        arr.clone_from_slice(res.as_slice());
        usize::from_le_bytes(arr)
    }) {
        let e = e.await;
        tracing::info!("Client got back: {}", e);
        total += e;
    }

    tracing::info!("Clients sleeping");

    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;

    tracing::info!("Total back: {}", total);
}
