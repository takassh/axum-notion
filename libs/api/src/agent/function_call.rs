use std::collections::HashMap;

use cloudflare::models::text_generation::{
    Message, MessageRequest, TextGeneration, TextGenerationRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

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
}

impl FunctionCallAgent {
    pub fn new(
        client: cloudflare::models::Models,
        available_tools: Vec<Tool>,
        history: Vec<Message>,
    ) -> Self {
        let system_prompt = format!(
            r#"
            You are a function calling AI model.
            You are provided three information:
            1. A list of available tools.
            2. Conversation history.
            3. A user prompt.
            Based on those 3 information, you must figure out context and generate function calls.
            Use the following pydantic model json schema to answer: {}
            For each function call return a json object with function name and arguments within <tool_call></tool_call> XML tags as follows: <tool_call>{}</tool_call> Always your response must starts with <tool_call>
            Available tools:
            <tools> {} </tools>
            Conversation history:
            {}
            user:
            Do you recommend any articles?
            response:
            I'd be happy to help you with that.As of the current date '28/05/2024 17:10', I can recommend some articles based on the ones available on this site. If you're interested in topics like Function calling using Hermes Fine-tuning LLM, you might find the articles 'Function calling using Hermes 2 pro mistral 7b' or 'Fine-tune LLM with lora' informative. If you're looking for articles on technical concepts like Image processing, Kafka, or Concurrent programming, 'Image processing: wasm vs dart', 'Kafka vs SQS', and 'Concurrency Models' might be worth checking out. Let me know if there's a specific topic you're interested in, and I can try to suggest relevant articles!
            user:
            I'd like to know the Function calling
            response:
            It looks like the article 'Function calling using Hermes 2 pro mistral 7b' is still a work in progress. According to the resource, it says 'In progress - stay tune 👌', so it's not available yet. However, I can suggest checking back later as the article is likely to be updated with new content. Alternatively, I can help you explore other related topics or articles on this site. Would you like me to do that?
            user:
            Yes please
            response:
            Let me see what else I can find that might be relevant to Function calling. As of the current date '28/05/2024 17:17', I noticed that there are several articles related to programming and technical concepts on this. If you're interested in learning more about concurrent programming, you might find the articles 'Concurrency Models' or 'Send vs' informative. Additionally, if you're curious about asynchronous programming, you might want to check out the articles 'Executing Futures and Tasks' or 'Async / Await'. Let me know if any of these articles catch your interest, or if you'd like me explore further!
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
}

impl Agent for FunctionCallAgent {
    async fn prompt(&self, prompt: &str) -> anyhow::Result<Vec<ToolCall>> {
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
            "FunctionCallAgent response: {}, ptompt:{}",
            response, prompt
        );

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
