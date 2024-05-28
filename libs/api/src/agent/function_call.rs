use std::collections::HashMap;

use cloudflare::models::text_generation::{
    Message, MessageRequest, TextGeneration, TextGenerationRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub trait Agent {
    fn prompt(
        &self,
        prompt: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<ToolCall>>> + Send;
    fn call(
        &self,
        messages: Vec<Message>,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;
}

pub struct FunctionCallAgent {
    client: cloudflare::models::Models,
    system_prompt: String,
    example_tool_call: String,
    example_tool_response: String,
    history: Vec<Message>,
}

impl FunctionCallAgent {
    pub fn new(
        client: cloudflare::models::Models,
        available_tools: Vec<Tool>,
        additional_system_prompt: Option<String>,
        example_tool_call: String,
        example_tool_response: String,
        history: Vec<Message>,
    ) -> Self {
        let system_prompt = format!(
            r#"
        Principal:
        You are a function calling AI model. You are provided with function signatures within <tools></tools> XML tags.
        Your response is whether to call one or more functions or not to get more information and context by user's prompt.
        Don't make assumptions about what values to plug into functions.
        Here are the available tools:
        <tools> {} </tools> 
        Use the following pydantic model json schema for each tool call you will make: {}
        For each function call return a json object with function name and arguments within <tool_call></tool_call> XML tags as follows:
        <tool_call> {} </tool_call>

        Additional:
        {}
        "#,
            serde_json::to_string(&available_tools).unwrap(),
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
            additional_system_prompt.unwrap_or_default()
        );
        Self {
            client,
            system_prompt,
            example_tool_call,
            example_tool_response,
            history,
        }
    }
}

impl Agent for FunctionCallAgent {
    async fn prompt(&self, prompt: &str) -> anyhow::Result<Vec<ToolCall>> {
        let messages = [
            Message {
                role: "system".to_string(),
                content: self.system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "What is this site?".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: self.example_tool_call.to_string(),
            },
            Message {
                role: "tool".to_string(),
                content: self.example_tool_response.to_string(),
            },
        ];

        let mut messages: Vec<_> =
            messages.iter().chain(&self.history).cloned().collect();

        messages.push(Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let response = self.call(messages).await?;

        if !response.contains("<tool_call>") {
            return Ok(vec![]);
        }

        let re = regex::Regex::new(r"<tool_call>|</tool_call>").unwrap();
        let mut rpcs = vec![];
        for part in re.split(&response) {
            let result = serde_json::from_str(part);
            let Ok(result) = result else {
                println!("Error parsing tool call: {:?}", result);
                continue;
            };
            rpcs.push(result);
        }

        Ok(rpcs)
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

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ToolCall {
    pub name: String,
    pub arguments: HashMap<String, String>,
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
}
