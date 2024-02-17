use openai_api_rust::chat::*;
use openai_api_rust::*;
use uuid::Uuid;

pub struct Completer {
    pub client: OpenAI,
}

#[derive(Debug)]
pub enum AiError {
    OpenAIError(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::OpenAIError(s) => write!(f, "OpenAIError: {}", s),
        }
    }
}

pub type CompletionQuery = ChatBody;

#[derive(Clone, Debug)]
pub struct Conversation {
    pub messages: Vec<Message>,
}

impl Default for Conversation {
    fn default() -> Self {
        Conversation::new()
    }
}

impl Conversation {
    pub fn new() -> Self {
        let initiating_message = Message {
            role: Role::System,
            content: "You are Hamish the sentient skeleton, you're generally relatively grumpy and are short with people who try to interrupt your patrol. Keep your response terse".to_string(),
        };

        Self {
            messages: vec![initiating_message],
        }
    }

    fn input(&mut self, message: String, role: Role) {
        let new_message = Message {
            role,
            content: message.to_string(),
        };
        self.messages.push(new_message);
    }

    pub fn input_from_self(&mut self, message: String) {
        self.input(message, Role::Assistant);
    }

    pub fn input_from_partner(&mut self, message: String) {
        self.input(message, Role::User);
    }
}

impl From<Conversation> for CompletionQuery {
    fn from(val: Conversation) -> Self {
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
            messages: val.messages,
        }
    }
}

impl Completer {
    pub fn complete(&self, query: (Uuid, CompletionQuery)) -> Result<String, AiError> {
        let body = query.1;
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
}
