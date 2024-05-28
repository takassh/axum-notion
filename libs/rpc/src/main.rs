use repository::Repository;
use rpc::serve;
use rpc_router::{CallResponse, Request};
use serde_json::json;
use util::load_env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secrets = load_env()?;
    let conn_string =
        secrets.get("LOCAL_DATABASE_URL").unwrap().as_str().unwrap();

    let repository = Repository::new(conn_string).await?;

    let rpc_router = serve(repository)?;

    // Create and parse rpc request example.
    let rpc_request: Request = json!({
        "jsonrpc": "2.0",
        "id": "some-client-req-id", // the json rpc id, that will get echoed back, can be null
        "method": "find_article_by_word",
        "params": {
            "word": "about"
        }
    })
    .try_into()?;

    // Async Execute the RPC Request with the router common resources
    let call_response = rpc_router.call(rpc_request).await?;

    // Or `call_with_resources` for  additional per-call Resources that override router common resources.
    // e.g., rpc_router.call_with_resources(rpc_request, additional_resources)

    // Display the response.
    let CallResponse { id, method, value } = call_response;

    println!(
        r#"RPC call response:

    id:  {id:?},
method:  {method},
 value:  {value:?},
"#
    );

    Ok(())
}
