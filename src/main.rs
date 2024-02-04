mod entity;

use openai_api_rust::chat::*;
use openai_api_rust::*;
use serde::de::DeserializeOwned;

use entity::character::Character;
use entity::SelfDescribe;

struct Completer {
    client: OpenAI,
}

#[derive(Debug)]
enum AiError {
    OpenAIError(String),
    SerdeError(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::OpenAIError(s) => write!(f, "OpenAIError: {}", s),
            AiError::SerdeError(e) => write!(f, "SerdeError: {}", e),
        }
    }
}

impl Completer {
    fn materialize<'a, Q, T: SelfDescribe<Input = Q> + DeserializeOwned + Default>(
        &self,
        input: &Q,
    ) -> Result<T, AiError> {
        let t = T::default();
        let message = t.describe(input);

        println!("Sending the following message: \"{message}\"");

        let body = ChatBody {
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: None,
            temperature: Some(0_f32),
            top_p: Some(0_f32),
            n: Some(1),
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: vec![Message {
                role: Role::User,
                content: message,
            }],
        };
        let result = self
            .client
            .chat_completion_create(&body)
            .map_err(|e| AiError::OpenAIError(e.to_string()))?;

        let message = result
            .choices
            .first()
            .ok_or_else(|| AiError::OpenAIError("No choices returned".into()))?
            .message
            .as_ref()
            .ok_or_else(|| AiError::OpenAIError("No message returned".into()))?
            .content
            .clone();

        serde_json::from_str::<T>(&message).map_err(|e| {
            AiError::SerdeError(format!(
                "Could not parse the following: \n {message} \n {e}"
            ))
        })
    }
}

fn main() {
    // Load API key from environment OPENAI_API_KEY.
    // You can also hadcode through `Auth::new(<your_api_key>)`, but it is not recommended.

    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    let completer = Completer { client: openai };

    let input: String = "a beautiful sorceress, dark hair, adept in fire magic".into();
    let thing = completer.materialize::<String, Character>(&input);

    println!("Got the following response {thing:?}");
}
