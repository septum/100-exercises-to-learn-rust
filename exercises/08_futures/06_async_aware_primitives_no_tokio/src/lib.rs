/// TODO: Can you understand the sequence of events that can lead to a deadlock?
use std::sync::mpsc;
use tokio::task::yield_now;

pub struct Message {
    payload: String,
    response_channel: mpsc::Sender<Message>,
}

pub async fn pong(mut receiver: mpsc::Receiver<Message>) {
    loop {
        // `.recv()` is blocking, so we switch to `.try_recv()`
        if let Ok(msg) = receiver.try_recv() {
            println!("Pong received: {}", msg.payload);
            let (sender, new_receiver) = mpsc::channel();
            msg.response_channel
                .send(Message {
                    payload: "pong".into(),
                    response_channel: sender,
                })
                .unwrap();
            receiver = new_receiver;
        } else {
            // call `yield_now()` to unblock the runtime
            // and continue the execution of other tasks
            yield_now().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::task::yield_now;

    use crate::{pong, Message};
    use std::sync::mpsc::{self, TryRecvError};

    #[tokio::test]
    async fn ping() {
        let (sender, receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();
        sender
            .send(Message {
                payload: "pong".into(),
                response_channel: response_sender,
            })
            .unwrap();

        tokio::spawn(pong(receiver));

        // Using `.try_recv()` here allow us to also `yield_now()`
        let answer: String = loop {
            match response_receiver.try_recv() {
                Ok(message) => break message.payload,
                Err(TryRecvError::Empty) => {
                    // Weirdly enough, there is no need to `continue` here
                    yield_now().await;
                }
                Err(TryRecvError::Disconnected) => unreachable!(),
            }
        };

        assert_eq!(answer, "pong");
    }
}
