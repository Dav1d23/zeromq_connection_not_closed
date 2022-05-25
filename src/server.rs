use zeromq::prelude::*;
use zeromq::ZmqMessage;

static ADDR: &str = "tcp://127.0.0.1:24042";

async fn single_loop(msg: zeromq::ZmqMessage) -> Result<(ZmqMessage, bool), ()> {
    let mut msg = msg.into_vec();
    assert_eq!(msg.len(), 1);
    let res: bytes::Bytes = msg.pop().unwrap();
    let res: Vec<u8> = res.into_iter().collect();
    let mut arr: [u8; 8] = [0; 8];
    arr.clone_from_slice(res.as_slice());
    let num = usize::from_le_bytes(arr);

    tracing::info!("Server received {}", num);

    let ans = ZmqMessage::from(Vec::from((num + 1000).to_le_bytes()));

    Ok((ans, num > 5))
}

#[tokio::main]
async fn main() {
    console_subscriber::ConsoleLayer::builder()
        .retention(tokio::time::Duration::from_secs(30))
        .server_addr(([127, 0, 0, 1], 5555))
        .init();

    async move {
        let mut socket = zeromq::RepSocket::new();
        let endpoint = socket.bind(ADDR).await.expect("unable to bind");
        tracing::info!("Server listening over {}", &ADDR);

        loop {
            let res = socket.recv().await.unwrap();

            match single_loop(res).await {
                Ok((ans, quit)) => {
                    tracing::info!("Server sending {:?}", &ans);
                    socket.send(ans).await.expect("unable to send the reply?");
                    if quit {
                        tracing::info!("Quitting.");
                        break;
                    }
                }
                Err(_) => {
                    tracing::error!("Error, quitting.");
                    break;
                }
            }
        }
        socket.unbind(endpoint).await.expect("unable to unbind");
        socket.close().await;
    }
    .await;

    tracing::info!("Server sleeping.");

    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;

    tracing::info!("Server closed.");
}
