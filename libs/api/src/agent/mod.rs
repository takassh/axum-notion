use std::pin::Pin;

use futures_util::Stream;

pub mod function_call;
pub mod question_and_answer;

pub trait Agent {
    type Item;
    fn prompt_with_stream(
        self,
        prompt: &str,
        context: Option<&str>,
    ) -> impl std::future::Future<
        Output = Pin<Box<dyn Stream<Item = Self::Item> + Send>>,
    > + Send;
    fn prompt(
        self,
        prompt: &str,
        context: Option<&str>,
    ) -> impl std::future::Future<Output = Self::Item> + Send;
}
