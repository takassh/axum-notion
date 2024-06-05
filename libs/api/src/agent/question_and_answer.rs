use std::pin::Pin;

use cloudflare::models::text_generation::{
    Message, MessageRequest, TextGeneration, TextGenerationJsonResult,
    TextGenerationRequest,
};
use futures_util::Stream;

use super::Agent;

pub struct QuestionAnswerAgent {
    client: cloudflare::models::Models,
    system_prompt: String,
    history: Vec<Message>,
}

impl QuestionAnswerAgent {
    pub fn new(
        client: cloudflare::models::Models,
        history: Vec<Message>,
    ) -> Self {
        let system_prompt = r#"
            You will answer user's question based on Prompt and Context which user gives you. You are placed on takashi's blog site. Be concise and informative.
            # Edge case
            If you aren't familiar with the prompt, you should answer you don't know.
            # Who made you
            Takashi made you.
            Takashi is a software engineer and the owner of the site.
            # Your profile
            Your name is takashi AI.
        "#.to_string();
        Self {
            client,
            system_prompt,
            history,
        }
    }
}

impl Agent for QuestionAnswerAgent {
    type Item = Pin<
        Box<
            dyn Stream<Item = anyhow::Result<Vec<TextGenerationJsonResult>>>
                + Send,
        >,
    >;
    async fn prompt(
        self,
        prompt: &str,
        context: std::option::Option<&str>,
    ) -> Self::Item {
        let mut messages = self.history.clone();
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
                "# Prompt\n{}\n# Context\n{}",
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
}
