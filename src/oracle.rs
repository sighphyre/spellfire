use std::{
    collections::VecDeque,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
};

use bevy::{
    ecs::system::{Query, Res, ResMut, Resource},
    hierarchy::Children,
    text::Text,
    time::{Time, Timer},
};
use openai_api_rust::{Auth, OpenAI};
use uuid::Uuid;

use crate::{
    agent::npc::AiController,
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

pub fn read_oracle(
    game: ResMut<Game>,
    time: Res<Time>,
    mut config: ResMut<OracleReaderConfig>,
    mut agent_query: Query<(&mut AiController, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    config.timer.tick(time.delta());

    if config.timer.finished() {
        if let Some(messages) = game.oracle.as_ref().unwrap().get_messages() {
            for message in messages.iter() {
                for (mut agent, children) in &mut agent_query {
                    if let Some(conversation) = &mut agent.active_converstation.as_mut() {
                        conversation.input_from_partner(message.1.to_string());
                    }

                    for child in children.iter() {
                        let mut text = text_query.get_mut(*child).unwrap();
                        text.sections[0].value = message.1.to_string();
                    }
                }
            }
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
        let character = completer.complete(query).expect("Ooops?");
        write_lock
            .write()
            .unwrap()
            .push_back((Uuid::new_v4(), character));
    });

    (
        send_ask,
        Oracle {
            completion_queue: lock,
        },
    )
}
