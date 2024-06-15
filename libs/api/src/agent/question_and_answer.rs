use std::pin::Pin;

use cloudflare::models::text_generation::{
    Message, MessageRequest, TextGeneration, TextGenerationJsonResult,
    TextGenerationRequest,
};
use futures_util::Stream;
use tracing::info;

use super::Agent;

pub struct QuestionAnswerAgent {
    client: cloudflare::models::Models,
    system_prompt: String,
    history: Vec<Message>,
    max_tokens: Option<i32>,
}

impl QuestionAnswerAgent {
    pub fn new(
        client: cloudflare::models::Models,
        system_prompt: String,
        history: Vec<Message>,
        max_tokens: Option<i32>,
    ) -> Self {
        Self {
            client,
            system_prompt,
            history,
            max_tokens,
        }
    }
}

impl Agent for QuestionAnswerAgent {
    type Item = anyhow::Result<Vec<TextGenerationJsonResult>>;

    async fn prompt_with_stream(
        self,
        prompt: &str,
        context: Option<&str>,
    ) -> Pin<Box<dyn Stream<Item = Self::Item> + Send>> {
        let contexts: Vec<_> = self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "system")
            .map(|m| m.content)
            .collect();
        let mut user_messages = vec![];
        for (i, mut message) in self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "user")
            .enumerate()
        {
            message.content = format!(
                "# Prompt\n{}\n# Blog resources\n{}",
                message.content,
                contexts.get(i).unwrap_or(&"".to_string()),
            );
            user_messages.push(message);
        }

        let mut messages = vec![];
        let assitant_messages: Vec<_> = self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "assistant")
            .collect();
        for (i, user_message) in user_messages.into_iter().enumerate() {
            messages.push(user_message);
            messages.push(assitant_messages.get(i).unwrap().clone());
        }

        messages.insert(
            0,
            Message {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
            },
        );

        messages.push(Message {
            role: "user".to_string(),
            content: format!(
                "# Prompt\n{}\n# Blog resources\n{}",
                prompt,
                context.unwrap_or_default(),
            )
            .to_string(),
        });

        let stream = self.client.llama_3_8b_instruct_with_stream(
            TextGenerationRequest::Message(MessageRequest {
                messages,
                stream: Some(true),
                ..Default::default()
            }),
        );

        Box::pin(stream)
    }

    async fn prompt(self, prompt: &str, _context: Option<&str>) -> Self::Item {
        let contexts: Vec<_> = self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "system")
            .map(|m| m.content)
            .collect();
        let mut user_messages = vec![];
        for (i, mut message) in self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "user")
            .enumerate()
        {
            message.content = format!(
                "# Prompt\n{}\n#Context\n{}",
                message.content,
                contexts.get(i).unwrap_or(&"".to_string()),
            );
            user_messages.push(message);
        }

        let mut messages = vec![];
        let assitant_messages: Vec<_> = self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "assistant")
            .collect();
        for (i, user_message) in user_messages.into_iter().enumerate() {
            messages.push(user_message);
            messages.push(assitant_messages.get(i).unwrap().clone());
        }

        messages.insert(
            0,
            Message {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
            },
        );
        messages.push(Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let messages = messages
            .into_iter()
            .filter(|m| !m.content.is_empty())
            .collect();

        let response = self
            .client
            .llama_3_8b_instruct(TextGenerationRequest::Message(
                MessageRequest {
                    messages,
                    max_tokens: self.max_tokens,
                    ..Default::default()
                },
            ))
            .await?;

        info!(
            "QuestionAnswerAgent response: {}, prompt:{}",
            response.result.response, prompt
        );

        Ok(vec![response.result])
    }
}
