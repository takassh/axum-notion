use cloudflare::models::text_generation::{
    Message, MessageRequest, TextGeneration, TextGenerationJsonResult,
    TextGenerationRequest, Tool, HERMES_2_PRO_MISTRAL_7B,
};
use langfuse::{apis::configuration, models::CreateGenerationBody};
use regex::Regex;
use tracing::info;
use uuid::Uuid;

use super::{get_template, Agent};

pub struct FunctionCallAgent {
    client: cloudflare::models::Models,
    name: String,
    system_prompt: String,
    history: Vec<Message>,
    pub temperature: Option<f32>, // from 0 to 5
    pub top_p: Option<f32>,       // from 0 to 2
    pub top_k: Option<f32>,       // from 1 to 50
}

impl FunctionCallAgent {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        client: cloudflare::models::Models,
        configuration: &configuration::Configuration,
        name: String,
        available_tools: Vec<Tool>,
        history: Vec<Message>,
        temperature: Option<f32>,
        top_p: Option<f32>,
        top_k: Option<f32>,
    ) -> anyhow::Result<Self> {
        let mut system_prompt =
            get_template(configuration, "function-calls-system").await?;

        let re = Regex::new(r"\s+").unwrap();
        system_prompt = system_prompt.replace(
            "{{schema}}",
            &re.replace_all(
                "{
                    'title': 'FunctionCall',
                    'type': 'object',
                    'properties': {
                        'arguments': {
                            'title': 'Arguments',
                            'type': 'object'
                        },
                        'name': {
                            'title': 'Name',
                            'type': 'string'
                        }
                    },
                    'required': ['arguments', 'name']
                }",
                "",
            ),
        );
        system_prompt = system_prompt.replace(
            "{{tool_call_response}}",
            &re.replace_all(
                "{
                    'arguments': '<args-dict>',
                    'name': 'function-name'
                }",
                "",
            ),
        );
        system_prompt = system_prompt.replace(
            "{{tools}}",
            &serde_json::to_string(&available_tools)?.replace('"', "'"),
        );

        Ok(Self {
            client,
            name,
            system_prompt,
            history,
            temperature,
            top_p,
            top_k,
        })
    }

    async fn call(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<TextGenerationJsonResult> {
        let response = self
            .client
            .hermes_2_pro_mistral_7b(TextGenerationRequest::Message(
                MessageRequest {
                    messages,
                    temperature: self.temperature,
                    top_p: self.top_p,
                    top_k: self.top_k,
                    ..Default::default()
                },
            ))
            .await?;
        Ok(response.result)
    }
}

impl Agent for FunctionCallAgent {
    type Item = TextGenerationJsonResult;

    async fn prompt(
        self,
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
            message.content = format!(
                "# Context\n{}\n# Prompt\n{}",
                contexts.get(i).unwrap_or(&"".to_string()),
                message.content,
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
            content: format!(
                "# Context\n{}\n# Prompt\n{}",
                context.unwrap_or_default(),
                prompt
            ),
        });

        let messages: Vec<_> = messages
            .into_iter()
            .filter(|m| !m.content.is_empty())
            .collect();

        let start_time = chrono::Utc::now().to_rfc3339();

        let response = self.call(messages.clone()).await?;

        let generation = CreateGenerationBody {
            id: Some(Some(Uuid::new_v4().to_string())),
            name: Some(Some(self.name.clone())),
            model: Some(Some(HERMES_2_PRO_MISTRAL_7B.to_string())),
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
            "FunctionCallAgent response: {:?}, prompt:{}",
            response, prompt
        );

        Ok((response, generation))
    }

    async fn prompt_with_stream(
        self,
        prompt: &str,
        context: Option<&str>,
    ) -> (
        Vec<Message>,
        std::pin::Pin<
            Box<
                dyn futures_util::Stream<Item = anyhow::Result<Self::Item>>
                    + Send,
            >,
        >,
    ) {
        let _ = context;
        let _ = prompt;
        todo!()
    }
}
