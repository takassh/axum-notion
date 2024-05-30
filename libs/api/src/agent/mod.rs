pub mod function_call;
pub mod question_and_answer;

pub trait Agent {
    type Item;
    fn prompt(
        self,
        prompt: &str,
        context: Option<&str>,
    ) -> impl std::future::Future<Output = Self::Item> + Send;
}
