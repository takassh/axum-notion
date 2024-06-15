use std::collections::HashMap;

use cloudflare::models::text_generation::{
    Message, MessageRequest, TextGeneration, TextGenerationRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use super::Agent;

pub struct FunctionCallAgent {
    client: cloudflare::models::Models,
    system_prompt: String,
    history: Vec<Message>,
    pub temperature: Option<f32>, // from 0 to 5
    pub top_p: Option<f32>,       // from 0 to 2
    pub top_k: Option<f32>,       // from 1 to 50
}

impl FunctionCallAgent {
    pub fn new(
        client: cloudflare::models::Models,
        available_tools: Vec<Tool>,
        history: Vec<Message>,
        temperature: Option<f32>,
        top_p: Option<f32>,
        top_k: Option<f32>,
    ) -> Self {
        let system_prompt = format!(
            r#"# Instructions
            You will answer function calls for search about user and other assistant's conversation.
            Never forget your answer must be the calls. Only respond the calls.
            Use the following pydantic model json schema to answer: {}
            For each function call return a json object with function name and arguments within <tool_call></tool_call> XML tags as follows: <tool_call>{}</tool_call>
            # Available tools
            <tools> {} </tools>
        "#,
            json!(
                {
                    "title": "FunctionCall",
                    "type": "object",
                    "properties": {
                        "arguments": {
                            "title": "Arguments",
                            "type": "object"
                        },
                        "name": {
                            "title": "Name",
                            "type": "string"
                        }
                    },
                    "required": ["arguments", "name"]
                }
            ),
            json!(
                {
                    "arguments": "<args-dict>",
                    "name": "function-name"
                }
            ),
            serde_json::to_string(&available_tools).unwrap(),
        );
        Self {
            client,
            system_prompt,
            history,
            temperature,
            top_p,
            top_k,
        }
    }

    async fn call(&self, messages: Vec<Message>) -> anyhow::Result<String> {
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
        Ok(response.result.response)
    }
}

impl Agent for FunctionCallAgent {
    type Item = anyhow::Result<Vec<ToolCall>>;

    async fn prompt(self, prompt: &str, context: Option<&str>) -> Self::Item {
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
                "#Context\n{}\n# Prompt\n{}",
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

        let messages = messages
            .into_iter()
            .filter(|m| !m.content.is_empty())
            .collect();

        let response = self.call(messages).await?;

        info!(
            "FunctionCallAgent response: {}, prompt:{}",
            response, prompt
        );

        if !response.contains("<tool_call>") {
            return Ok(vec![]);
        }

        let re = regex::Regex::new(r"<tool_call>|</tool_call>").unwrap();
        let mut rpcs = vec![];
        for part in re.split(&response) {
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

        Ok(rpcs)
    }

    async fn prompt_with_stream(
        self,
        prompt: &str,
        context: Option<&str>,
    ) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = Self::Item> + Send>>
    {
        let _ = context;
        let _ = prompt;
        todo!();
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Option<HashMap<String, Option<String>>>,
}

#[derive(Serialize, Default)]
pub struct Tool {
    pub r#type: String,
    pub function: Function,
}

#[derive(Serialize, Default)]
pub struct Function {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Parameters>,
}

#[derive(Serialize, Default)]
pub struct Parameters {
    pub r#type: String,
    pub properties: HashMap<String, PropertyType>,
    pub required: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum PropertyType {
    String,
    Number,
}
