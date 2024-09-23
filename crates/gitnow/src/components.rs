use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod inline_command;
pub mod spinner;

#[derive(Debug, PartialEq)]
pub enum Msg {
    Quit,
    Tick,
    Success,
    Failure(String),
}

pub struct Command {
    func: Box<CommandFunc>,
}

impl Command {
    pub fn new<T: FnOnce(&Dispatch) -> Option<Msg> + 'static>(f: T) -> Self {
        Self { func: Box::new(f) }
    }

    pub fn execute(self, dispatch: &Dispatch) -> Option<Msg> {
        (self.func)(dispatch)
    }
}

pub trait IntoCommand {
    fn into_command(self) -> Command;
}

impl IntoCommand for () {
    fn into_command(self) -> Command {
        Command::new(|_| None)
    }
}

impl IntoCommand for Command {
    fn into_command(self) -> Command {
        self
    }
}

impl IntoCommand for Msg {
    fn into_command(self) -> Command {
        Command::new(|_| Some(self))
    }
}

type CommandFunc = dyn FnOnce(&Dispatch) -> Option<Msg>;

pub fn create_dispatch() -> (Dispatch, Receiver) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    (Dispatch { sender: tx }, Receiver { receiver: rx })
}

#[derive(Clone)]
pub struct Dispatch {
    sender: UnboundedSender<Msg>,
}

impl Dispatch {
    pub fn send(&self, msg: Msg) {
        if let Err(e) = self.sender.send(msg) {
            tracing::warn!("failed to send event: {}", e);
        }
    }
}

pub struct Receiver {
    receiver: UnboundedReceiver<Msg>,
}

impl Receiver {
    pub async fn next(&mut self) -> Option<Msg> {
        self.receiver.recv().await
    }
}

#[derive(Default)]
pub struct BatchCommand {
    commands: Vec<Command>,
}

impl BatchCommand {
    pub fn with(&mut self, cmd: impl IntoCommand) -> &mut Self {
        self.commands.push(cmd.into_command());

        self
    }
}

impl IntoCommand for Vec<Command> {
    fn into_command(self) -> Command {
        BatchCommand::from(self).into_command()
    }
}

impl From<Vec<Command>> for BatchCommand {
    fn from(value: Vec<Command>) -> Self {
        BatchCommand { commands: value }
    }
}

impl IntoCommand for BatchCommand {
    fn into_command(self) -> Command {
        Command::new(|dispatch| {
            for command in self.commands {
                let msg = command.execute(dispatch);
                if let Some(msg) = msg {
                    dispatch.send(msg);
                }
            }

            None
        })
    }
}
