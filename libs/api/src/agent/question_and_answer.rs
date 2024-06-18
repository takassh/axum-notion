use std::pin::Pin;

use cloudflare::models::text_generation::{
    Message, MessageRequest, ModelParameters, TextGeneration,
    TextGenerationJsonResult, TextGenerationRequest, LLAMA_3_8B_INSTRUCT,
};
use futures_util::Stream;
use langfuse::models::CreateGenerationBody;
use tracing::info;
use uuid::Uuid;

use super::Agent;

pub struct QuestionAnswerAgent {
    client: cloudflare::models::Models,
    name: String,
    system_prompt: String,
    history: Vec<Message>,
    model_parameters: Option<ModelParameters>,
}

impl QuestionAnswerAgent {
    pub fn new(
        client: cloudflare::models::Models,
        name: String,
        system_prompt: String,
        history: Vec<Message>,
        model_parameters: Option<ModelParameters>,
    ) -> Self {
        Self {
            client,
            name,
            system_prompt,
            history,
            model_parameters,
        }
    }
}

impl Agent for QuestionAnswerAgent {
    type Item = Vec<TextGenerationJsonResult>;

    async fn prompt_with_stream(
        self,
        user_prompt_template: &str,
        prompt: &str,
        context: Option<&str>,
    ) -> (
        Vec<Message>,
        Pin<Box<dyn Stream<Item = anyhow::Result<Self::Item>> + Send>>,
    ) {
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
            message.content = user_prompt_template
                .replace("{{prompt}}", &message.content)
                .replace(
                    "{{context}}",
                    contexts.get(i).unwrap_or(&"".to_string()),
                );
            user_messages.push(message);
        }

        let mut messages = vec![];
        let assistant_messages: Vec<_> = self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "assistant")
            .collect();
        for (i, user_message) in user_messages.into_iter().enumerate() {
            messages.push(user_message);
            messages.push(assistant_messages.get(i).unwrap().clone());
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
            content: user_prompt_template
                .replace("{{prompt}}", prompt)
                .replace("{{context}}", context.unwrap_or_default()),
        });

        let stream = self.client.llama_3_8b_instruct_with_stream(
            TextGenerationRequest::Message(MessageRequest {
                messages: messages.clone(),
                stream: Some(true),
                model_parameters: self.model_parameters.clone(),
            }),
        );

        (messages, Box::pin(stream))
    }

    async fn prompt(
        self,
        user_prompt_template: &str,
        prompt: &str,
        context: Option<&str>,
    ) -> anyhow::Result<(Self::Item, CreateGenerationBody)> {
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
            message.content = user_prompt_template
                .replace("{{prompt}}", &message.content)
                .replace(
                    "{{context}}",
                    contexts.get(i).unwrap_or(&"".to_string()),
                );
            user_messages.push(message);
        }

        let mut messages = vec![];
        let assistant_messages: Vec<_> = self
            .history
            .clone()
            .into_iter()
            .filter(|m| m.role == "assistant")
            .collect();
        for (i, user_message) in user_messages.into_iter().enumerate() {
            messages.push(user_message);
            messages.push(assistant_messages.get(i).unwrap().clone());
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
            content: user_prompt_template
                .replace("{{prompt}}", prompt)
                .replace("{{context}}", context.unwrap_or_default()),
        });

        let messages: Vec<_> = messages
            .into_iter()
            .filter(|m| !m.content.is_empty())
            .collect();

        let start_time = chrono::Utc::now().to_rfc3339();

        let response = self
            .client
            .llama_3_8b_instruct(TextGenerationRequest::Message(
                MessageRequest {
                    messages: messages.clone(),
                    model_parameters: self.model_parameters.clone(),
                    ..Default::default()
                },
            ))
            .await?;

        let generation = CreateGenerationBody {
            id: Some(Some(Uuid::new_v4().to_string())),
            name: Some(Some(self.name.clone())),
            model: Some(Some(LLAMA_3_8B_INSTRUCT.to_string())),
            model_parameters: Some(Some(
                serde_json::from_value(
                    serde_json::to_value(self.model_parameters).unwrap(),
                )
                .unwrap(),
            )),
            start_time: Some(Some(start_time)),
            end_time: Some(Some(chrono::Utc::now().to_rfc3339())),
            input: Some(Some(serde_json::Value::Array(
                messages
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "role": t.role,
                            "content": t.content,
                        })
                    })
                    .collect(),
            ))),
            output: Some(Some(serde_json::Value::String(
                serde_json::to_string_pretty(&response).unwrap(),
            ))),
            ..Default::default()
        };

        info!(
            "QuestionAnswerAgent response: {:?}, prompt:{}",
            response, prompt
        );

        Ok((vec![response.result], generation))
    }
}
