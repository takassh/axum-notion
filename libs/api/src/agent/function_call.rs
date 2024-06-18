use cloudflare::models::text_generation::{
    Message, MessageRequest, ModelParameters, TextGeneration,
    TextGenerationJsonResult, TextGenerationRequest, Tool,
    HERMES_2_PRO_MISTRAL_7B,
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
    model_parameters: Option<ModelParameters>,
}

impl FunctionCallAgent {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        client: cloudflare::models::Models,
        configuration: &configuration::Configuration,
        name: String,
        available_tools: Vec<Tool>,
        history: Vec<Message>,
        model_parameters: Option<ModelParameters>,
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
            model_parameters,
        })
    }

    async fn call(
        &self,
        messages: Vec<Message>,
        model_parameters: Option<ModelParameters>,
    ) -> anyhow::Result<TextGenerationJsonResult> {
        let response = self
            .client
            .hermes_2_pro_mistral_7b(TextGenerationRequest::Message(
                MessageRequest {
                    messages,
                    model_parameters,
                    stream: Some(false),
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

        let mut response = self
            .call(messages.clone(), self.model_parameters.clone())
            .await?;

        if response.tool_calls.is_none() {
            let re = regex::Regex::new(r"<tool_call>|</tool_call>").unwrap();
            let mut rpcs = vec![];
            for part in re.split(&response.response.clone().unwrap_or_default())
            {
                let part = part.trim().replace('\n', "");
                if part.is_empty() {
                    continue;
                }
                let part = part.replace('\'', "\"");
                let mut part = part.replace('\\', "");
                if part.find("name") < part.find("arguments") {
                    let start = regex::Regex::new(r".*\{.*?name").unwrap();
                    let end = regex::Regex::new(r"arguments.*}.*}").unwrap();
                    if !start.is_match(&part) {
                        part.insert(0, '{');
                    }
                    if !end.is_match(&part) {
                        part.push('}');
                    }
                } else {
                    let start = regex::Regex::new(r".*\{.*?arguments").unwrap();
                    let end = regex::Regex::new(r"name.*}").unwrap();
                    if !start.is_match(&part) {
                        part.insert(0, '{');
                    }
                    if !end.is_match(&part) {
                        part.push('}');
                    }
                }
                let extra = regex::Regex::new(r"^.*?\{").unwrap();
                let part: std::borrow::Cow<str> = extra.replace(&part, "{");
                let extra = regex::Regex::new(r"\}[^\}]*?$").unwrap();
                let part: std::borrow::Cow<str> = extra.replace(&part, "}");
                let result = serde_json::from_str(&part);
                let Ok(result) = result else {
                    println!(
                        "Error parsing tool call: {:?}, part: {}",
                        result, part
                    );
                    continue;
                };
                rpcs.push(result);
            }

            response.tool_calls = Some(rpcs);
        }

        let generation = CreateGenerationBody {
            id: Some(Some(Uuid::new_v4().to_string())),
            name: Some(Some(self.name.clone())),
            model: Some(Some(HERMES_2_PRO_MISTRAL_7B.to_string())),
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
            "FunctionCallAgent response: {:?}, prompt:{}",
            response, prompt
        );

        Ok((response, generation))
    }

    async fn prompt_with_stream(
        self,
        user_prompt_template: &str,
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
        let _ = user_prompt_template;
        let _ = context;
        let _ = prompt;
        todo!()
    }
}
