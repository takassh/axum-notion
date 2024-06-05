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
}

impl FunctionCallAgent {
    pub fn new(
        client: cloudflare::models::Models,
        available_tools: Vec<Tool>,
        history: Vec<Message>,
    ) -> Self {
        let system_prompt = format!(
            r#"
            You will return function calls.
            You will get three information:
            1. A list of available tools.
            2. Conversation history.
            3. A user prompt.
            Based on those 3 information, you must generate function calls to get related information.
            Use the following pydantic model json schema to answer: {}
            For each function call return a json object with function name and arguments within <tool_call></tool_call> XML tags as follows: <tool_call>{}</tool_call> Always your response must start with <tool_call>
            Available tools:
            <tools> {} </tools>
            Conversation history:
            {}
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
            history
                .iter()
                .map(|x| {
                    if x.role == "user" {
                        format!("user:{}", x.content)
                    } else {
                        format!("response:{}", x.content)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        );
        Self {
            client,
            system_prompt,
        }
    }

    async fn call(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let response = self
            .client
            .hermes_2_pro_mistral_7b(TextGenerationRequest::Message(
                MessageRequest {
                    messages,
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
        let _ = context;
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: self.system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];

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
            let part = part.replace('\'', "\"");
            let extra = regex::Regex::new(r"^.*?\{").unwrap();
            let part: std::borrow::Cow<str> = extra.replace(&part, "{");
            let extra = regex::Regex::new(r"\}[^\}]*?$").unwrap();
            let part: std::borrow::Cow<str> = extra.replace(&part, "}");
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let result = serde_json::from_str(part);
            let Ok(result) = result else {
                println!("Error parsing tool call: {:?}", result);
                continue;
            };
            rpcs.push(result);
        }

        Ok(rpcs)
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
