/*!
# Actor Example
(To heck with streams ... they're ultimately a funny way of funelling code into something serialized.)
[Rhyl: Actor Example](https://ryhl.io/blog/actors-with-tokio/)
*/

// ///////////////////////////////////////// [ use ] ///////////////////////////////////////// //
use std::error::Error;
use std::time::Duration;
use thiserror::Error;
use tokio::{
    select,
    sync::{Semaphore, mpsc, oneshot},
    task,
    time::{Interval, interval},
};
// ///////////////////////////////////////// [ error ] ///////////////////////////////////////// //
#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Message alreaedy set {0}")]
    MessageAlreadySet(String),
    // #[error("Failed to send message")]
    // SendError(#[from] mpsc::error::SendError<Message>),
    // #[error("Failed to receive message")]
    // ReceiveError(#[from] mpsc::error::RecvError),
}
// ///////////////////////////////////////// [ actor-elements ] ///////////////////////////////////////// //
struct PrintingActor {
    receiver: mpsc::Receiver<ActorMessage>,
    print_message: Option<String>,
    rate: Interval,
    print_count: u32,
    _resource: Semaphore,
    active: bool,
}
enum ActorMessage {
    _InitMessage(String),
    SetMessage(String),
    SetPeriod(Duration),
    SetActive,
    SetInactive,
    ToggleActive,
    GetPrintCount { resp_channel: oneshot::Sender<u32> },
    GetStatus { resp_channel: oneshot::Sender<bool> },
}
// ///////////////////////////////////////// [ actor-impls ] ///////////////////////////////////////// //
impl PrintingActor {
    pub async fn spawn() -> (
        mpsc::Sender<ActorMessage>,
        task::JoinHandle<Result<(), ActorError>>,
    ) {
        let (tx, rx) = mpsc::channel(5);
        let actor = Self {
            receiver: rx,
            print_message: None,
            rate: interval(Duration::from_millis(100)),
            print_count: 0,
            _resource: Semaphore::new(1),
            active: true,
        };
        let task_handle = task::spawn(async move { actor.run().await });
        (tx, task_handle)
    }

    /// Actor's run loop.
    async fn run(mut self) -> Result<(), ActorError> {
        loop {
            select! {
                Some(msg) = self.receiver.recv() => {
                        self.handle_message(msg).await?
                }
                _ = self.rate.tick(), if self.active => {
                    if let Some(message) = &self.print_message {
                        println!("{}", message);
                        self.print_count += 1;
                    }
                }
            }
        }
    }

    pub async fn handle_message(&mut self, msg: ActorMessage) -> Result<(), ActorError> {
        match msg {
            ActorMessage::_InitMessage(msg) => {
                if let Some(mssg) = &self.print_message {
                    Err(ActorError::MessageAlreadySet(mssg.clone()))?;
                }
                self.print_message = Some(msg);
            }
            ActorMessage::SetMessage(msg) => self.print_message = Some(msg),
            ActorMessage::SetPeriod(period) => self.rate = interval(period),
            ActorMessage::SetActive => self.active = true,
            ActorMessage::SetInactive => self.active = false,
            ActorMessage::ToggleActive => self.active = !self.active,
            ActorMessage::GetPrintCount { resp_channel } => {
                resp_channel.send(self.print_count).unwrap()
            }
            ActorMessage::GetStatus { resp_channel } => resp_channel.send(self.active).unwrap(),
        }
        Ok(())
    }
}
// ///////////////////////////////////////// [  ] ///////////////////////////////////////// //

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let mut controls = Vec::new();
    for _ in 0..6 {
        let (tx, handle) = PrintingActor::spawn().await;
        controls.push((tx, handle));
    }

    loop {
        let mut input = String::new();
        let commands = indoc::indoc!(
            "
                        \nCommands:
                        \"message N\": set message for actor N
                        \"activate N\": activate actor N
                        \"deactivate N\": deactivate actor N
                        \"toggle N\": toggle actor N
                        \"rate N MS\": set rate in ms for actor N
                        \"pause\": pause all actors
                        \"drop\": abort actors
                        \"report\": report status of all actors
                        \"printcount\": print count of actors
                        "
        );
        println!("{commands}");

        std::io::stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            ["drop"] => {
                for (_, handle) in &controls {
                    handle.abort();
                }
                break;
            }
            ["pause"] => {
                for (tx, _) in &controls {
                    tx.send(ActorMessage::SetInactive).await?;
                }
            }
            ["activate"] => {
                for (tx, _) in &controls {
                    tx.send(ActorMessage::SetActive).await?;
                }
            }
            ["message", n] => {
                if let Ok(idx) = n.parse::<usize>() {
                    if let Some((tx, _)) = controls.get(idx) {
                        println!("Enter message for Actor {}:", idx);
                        let mut msg = String::new();
                        std::io::stdin().read_line(&mut msg).unwrap();
                        tx.send(ActorMessage::SetMessage(msg.trim().to_string()))
                            .await
                            .unwrap_or_else(|e| eprintln!("Failed to send message: {}", e));
                    }
                }
            }
            ["activate", n] => {
                if let Ok(idx) = n.parse::<usize>() {
                    if let Some((tx, _)) = controls.get(idx) {
                        tx.send(ActorMessage::SetActive).await.unwrap_or_else(|e| {
                            eprintln!("Failed to activate actor {}: {}", idx, e)
                        });
                    }
                }
            }
            ["deactivate", n] => {
                if let Ok(idx) = n.parse::<usize>() {
                    if let Some((tx, _)) = controls.get(idx) {
                        tx.send(ActorMessage::SetInactive)
                            .await
                            .unwrap_or_else(|e| {
                                eprintln!("Failed to deactivate actor {}: {}", idx, e)
                            });
                    }
                }
            }
            ["toggle", n] => {
                if let Ok(idx) = n.parse::<usize>() {
                    if let Some((tx, _)) = controls.get(idx) {
                        tx.send(ActorMessage::ToggleActive)
                            .await
                            .unwrap_or_else(|e| eprintln!("Failed to toggle actor {}: {}", idx, e));
                    }
                }
            }
            ["rate", n, ms] => {
                if let (Ok(idx), Ok(ms)) = (n.parse::<usize>(), ms.parse::<u64>()) {
                    if let Some((tx, _)) = controls.get(idx) {
                        tx.send(ActorMessage::SetPeriod(Duration::from_millis(ms)))
                            .await
                            .unwrap_or_else(|e| {
                                eprintln!("Failed to set rate for actor {}: {}", idx, e)
                            });
                    }
                }
            }
            ["report"] => {
                for (idx, (tx, _)) in controls.iter().enumerate() {
                    let (otx, orx) = oneshot::channel();
                    tx.send(ActorMessage::GetStatus { resp_channel: otx })
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to report status for actor {}: {}", idx, e)
                        });
                    let active_status = orx.await?;
                    println!(
                        "Actor {}: {}",
                        idx,
                        if active_status { "active" } else { "inactive" }
                    );
                }
            }
            ["printcount"] => {
                for (idx, (tx, _)) in controls.iter().enumerate() {
                    let (otx, orx) = oneshot::channel();
                    tx.send(ActorMessage::GetPrintCount { resp_channel: otx })
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to get print count for actor {}: {}", idx, e)
                        });
                    let count = orx.await?;
                    println!("Actor {} print count: {}", idx, count);
                }
            }

            _ => println!("Invalid command. Use: {commands}"),
        }
    }

    println!("Exiting...");
    Ok(())
}
