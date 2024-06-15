use repository::Repository;
use rpc::serve;
use rpc_router::{CallResponse, Request};
use serde_json::json;
use util::load_env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secrets = load_env()?;

    let config_name =
        &format!("Config{}", secrets.get("CONFIG").unwrap().as_str().unwrap());
    let conn_string =
        secrets.get("LOCAL_DATABASE_URL").unwrap().as_str().unwrap();
    let cloudflare_token =
        secrets.get("CLOUDFLARE_TOKEN").unwrap().as_str().unwrap();
    let cloudflare_account_id = secrets
        .get("CLOUDFLARE_ACCOUNT_ID")
        .unwrap()
        .as_str()
        .unwrap();

    let config = util::load_config(config_name)?;

    let repository = Repository::new(conn_string).await?;

    let cloudflare = cloudflare::models::Models::new(
        &cloudflare_account_id,
        &cloudflare_token,
    );

    let rpc_router = serve(
        config_name,
        repository,
        qdrant_client::client::QdrantClient::from_url(
            config
                .get("qdrant")
                .unwrap()
                .get("base_url")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .with_api_key(secrets.get("QDRANT_API_KEY").unwrap().as_str().unwrap())
        .build()
        .unwrap(),
        cloudflare,
    )?;

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
