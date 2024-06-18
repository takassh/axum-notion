use std::pin::Pin;

use cloudflare::models::text_generation::Message;
use futures_util::Stream;
use langfuse::{
    apis::{configuration, prompts_api::prompts_get},
    models::CreateGenerationBody,
};

pub mod function_call;
pub mod question_and_answer;

pub trait Agent {
    type Item;
    fn prompt_with_stream(
        self,
        user_prompt_template: &str,
        prompt: &str,
        context: Option<&str>,
    ) -> impl std::future::Future<
        Output = (
            Vec<Message>,
            Pin<Box<dyn Stream<Item = anyhow::Result<Self::Item>> + Send>>,
        ),
    > + Send;

    fn prompt(
        self,
        user_prompt_template: &str,
        prompt: &str,
        context: Option<&str>,
    ) -> impl std::future::Future<
        Output = anyhow::Result<(Self::Item, CreateGenerationBody)>,
    > + Send;
}

pub async fn get_template(
    configuration: &configuration::Configuration,
    name: &str,
) -> anyhow::Result<String> {
    let result = prompts_get(configuration, name, None, None).await?;

    match result {
        langfuse::models::Prompt::PromptOneOf1(prompt) => Ok(prompt.prompt),
        _ => Err(anyhow::anyhow!("Prompt is not of type PromptOneOf1")),
    }
}
