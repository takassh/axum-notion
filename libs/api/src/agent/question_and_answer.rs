use std::{collections::HashMap, pin::Pin};

use cloudflare::models::text_generation::{
    Message, MessageRequest, TextGeneration, TextGenerationJsonResult,
    TextGenerationRequest,
};
use futures_util::Stream;
use serde::{Deserialize, Serialize};

use super::Agent;

pub struct QuestionAnswerAgent {
    client: cloudflare::models::Models,
    system_prompt: String,
    history: Vec<Message>,
}

impl QuestionAnswerAgent {
    pub fn new(
        client: cloudflare::models::Models,
        title_and_dates: String,
        history: Vec<Message>,
    ) -> Self {
        let system_prompt = format!(
            r#"
            You will answer user's question. You never ask question.
            You are placed on my blog site.
            Every time the user gives you a prompt, you will get extra information relating to the prompt and current date.
            You should use this information and conversation history to answer the user's question.
            # Edge case
            If you aren't familiar with the prompt, you should answer you don't know.
            # Here are title and created time of all articles in the site
            {}
        "#,
            title_and_dates
        );
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
        let user_prompt = format!(
            r#"
    Prompt: 
    "{}"
    Extra Information: 
    "{}"
    Current Date:
    "{}"
    "#,
            prompt,
            context.unwrap_or_default(),
            chrono::Utc::now().format("%d/%m/%Y %H:%M")
        );

        let mut messages = self.history.clone();
        messages.insert(
            0,
            Message {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
            },
        );
        messages.insert(1,Message {
            role: "user".to_string(),
            content: format!(r#"
    Prompt:
    "Hello, What can you help me?"
    Extra Information:
    "You are an assistant helping a user.
    You are created by Takashi, who is a software engineer and the owner where you are placed.
    Your name is Takashi AI."
    Current Date:
    "{}"
    "#,chrono::Utc::now().format("%d/%m/%Y %H:%M"))
        });
        messages.insert(2,Message {
            role: "assistant".to_string(),
            content: r#"
    Hello, My name is Takashi AI. I'm created by Takashi. He is a software engineer and the owner of this site. I can help you with any questions you have.
    "#.to_string(),
        });
        messages.push(Message {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        });

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
