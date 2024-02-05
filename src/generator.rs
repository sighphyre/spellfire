use openai_api_rust::chat::*;
use openai_api_rust::*;
use serde::de::DeserializeOwned;

use entity::SelfDescribe;

use crate::entity;

/// use entity::character::Character;
/// use openai_api_rust::{Auth, OpenAI};
///
/// use crate::generator::Completer;
/// fn main() {
///     // Load API key from environment OPENAI_API_KEY.
///     let auth = Auth::from_env().unwrap();
///     let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
///
///     let completer = Completer { client: openai };
///
///     let input: String = "a beautiful sorceress, dark hair, adept in fire magic".into();
///     let thing = completer.materialize::<String, Character>(&input);
///
///     println!("Got the following response {thing:?}");
/// }

pub struct Completer {
    pub client: OpenAI,
}

#[derive(Debug)]
pub enum AiError {
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

pub type CompletionQuery = ChatBody;

pub fn query<Q, T: SelfDescribe<Input = Q> + Default>(input: &Q) -> CompletionQuery {
    let t = T::default();
    let message = t.describe(input);

    // println!("Building the following message: \"{message}\"");

    ChatBody {
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: None,
        temperature: Some(0.3_f32),
        top_p: None,
        n: None,
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
    }
}

impl Completer {
    pub fn complete(&self, body: CompletionQuery) -> Result<String, AiError> {
        let result = self
            .client
            .chat_completion_create(&body)
            .map_err(|e| AiError::OpenAIError(e.to_string()))?;

        Ok(result
            .choices
            .first()
            .ok_or_else(|| AiError::OpenAIError("No choices returned".into()))?
            .message
            .as_ref()
            .ok_or_else(|| AiError::OpenAIError("No message returned".into()))?
            .content
            .clone())
    }

    pub fn complete_as<T>(&self, body: CompletionQuery) -> Result<T, AiError>
    where
        T: DeserializeOwned,
    {
        let response = self.complete(body)?;

        serde_json::from_str::<T>(&response).map_err(|e| {
            AiError::SerdeError(format!(
                "Could not parse the following: \n {response} \n {e}"
            ))
        })
    }
}
