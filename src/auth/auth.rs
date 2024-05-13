use crate::secretary::Secretary;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::StatusCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use warp::Filter;
use webbrowser;

pub async fn initialize(secretary: &mut Secretary) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OAuth2 client
    let client = Arc::new(
        BasicClient::new(
            ClientId::new(secretary.config.dropbox_client_id.clone()),
            Some(ClientSecret::new(
                secretary.config.dropbox_client_secret.clone(),
            )),
            AuthUrl::new("https://www.dropbox.com/oauth2/authorize".to_string())?,
            Some(TokenUrl::new(
                "https://api.dropboxapi.com/oauth2/token".to_string(),
            )?),
        )
        .set_redirect_uri(RedirectUrl::new(
            "http://localhost:8080/callback".to_string(),
        )?),
    );

    // Check if we need to run OAuth2 flow
    if !is_token_valid(&secretary.dropbox_client.access_token).await {
        // Create oneshot channel to wait for the authorization code
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));

        // Clone the client and oneshot sender for the server
        let client_clone = Arc::clone(&client);
        let tx_clone = Arc::clone(&tx);

        // Run the server to handle OAuth callback
        tokio::spawn(async move {
            run_server(client_clone, tx_clone).await;
        });

        // Generate the authorization URL and open it in the browser
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("files.content.read".to_string()))
            .url();

        println!("Opening browser to: {}", auth_url);
        webbrowser::open(auth_url.as_str())?;

        // Wait for the authorization code
        let auth_code = rx.await?;
        println!("Authorization code received: {}", auth_code);

        // Exchange the authorization code for an access token
        let token = client
            .exchange_code(AuthorizationCode::new(auth_code))
            .request_async(async_http_client)
            .await?;

        println!("Access token: {}", token.access_token().secret());

        // Update the Dropbox client with the access token
        secretary.dropbox_client.access_token = token.access_token().secret().clone();
    } else {
        println!("Using existing access token.");
    }

    Ok(())
}

async fn run_server(client: Arc<BasicClient>, tx: Arc<Mutex<Option<oneshot::Sender<String>>>>) {
    let routes = warp::path("callback")
        .and(warp::query::<HashMap<String, String>>())
        .map(move |params: HashMap<String, String>| {
            if let Some(code) = params.get("code") {
                println!("Received code: {}", code);
                if let Some(tx) = tx.lock().unwrap().take() {
                    tx.send(code.clone()).expect("Failed to send auth code");
                }
            }
            warp::reply::html("Authorization code received. You can close this window.")
        });

    println!("Server running on http://localhost:8080/callback");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

// Function to check if the token is valid
async fn is_token_valid(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.dropboxapi.com/2/check/user")
        .bearer_auth(token)
        .send()
        .await;

    match res {
        Ok(response) => response.status() == StatusCode::OK,
        Err(_) => false,
    }
}
