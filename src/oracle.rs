use std::{
    collections::VecDeque,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
};

use bevy::{
    ecs::{
        event::{Event, EventWriter},
        system::{Res, ResMut, Resource},
    },
    time::{Time, Timer},
};
use openai_api_rust::{Auth, OpenAI};
use uuid::Uuid;

use crate::{
    generator::{Completer, CompletionQuery},
    Game,
};

pub struct Oracle {
    pub completion_queue: Arc<RwLock<VecDeque<(Uuid, String)>>>,
}

pub type OracleMessage = (Uuid, CompletionQuery);

impl Oracle {
    pub fn get_messages(&self) -> Option<Vec<(Uuid, String)>> {
        let mut lock = self.completion_queue.write().unwrap();
        if lock.is_empty() {
            None
        } else {
            let mut result = Vec::new();
            while let Some(item) = lock.pop_front() {
                result.push(item);
            }
            Some(result)
        }
    }
}

#[derive(Resource)]
pub struct OracleReaderConfig {
    pub timer: Timer,
}

#[derive(Event, Clone, Debug)]
pub struct CompletionCallback {
    pub id: Uuid,
    pub message: String,
}

pub fn read_oracle(
    game: ResMut<Game>,
    time: Res<Time>,
    mut config: ResMut<OracleReaderConfig>,
    mut completion_handler: EventWriter<CompletionCallback>,
) {
    config.timer.tick(time.delta());

    if config.timer.finished() {
        if let Some(messages) = game.oracle.get_messages() {
            completion_handler.send(CompletionCallback {
                id: messages[0].0,
                message: messages[0].1.clone(),
            });
        }
    }
}

pub fn start_oracle() -> (Sender<OracleMessage>, Oracle) {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let completer = Completer { client: openai };

    let (send_ask, receive_ask): (Sender<OracleMessage>, Receiver<OracleMessage>) = channel();

    let queue: VecDeque<(Uuid, String)> = VecDeque::default();
    let lock = Arc::new(std::sync::RwLock::new(queue));

    let write_lock = lock.clone();

    //spawn new thread first for our background processing, this is a terrible, terrible idea and needs fixing
    std::thread::spawn(move || loop {
        let query = receive_ask.recv().unwrap();
        let id = query.0;
        let character = completer.complete(query).expect("Ooops?");
        write_lock.write().unwrap().push_back((id, character));
    });

    (
        send_ask,
        Oracle {
            completion_queue: lock,
        },
    )
}
