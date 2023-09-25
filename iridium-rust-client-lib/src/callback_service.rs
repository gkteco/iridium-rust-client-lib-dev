pub mod callback_service {
    use crate::callback_service::{exchange_headers, exchange_url};
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Deserialize, Serialize, Debug)]
    struct TokenResponse {
        access_token: String,
    }
    pub async fn handle_callback(
        params: HashMap<String, String>,
        verifier: String,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        if let (Some(code), Some(state)) = (params.get("code"), params.get("state")) {
            let client = reqwest::Client::new();
            let exchange_url = exchange_url::generate(code, state, &verifier);
            println!("exchange_url: {}", exchange_url);
            let headers = exchange_headers::generate();

            match client.post(&exchange_url).headers(headers).send().await {
                Ok(response) if response.status() == StatusCode::OK => {
                    let json = response.json::<TokenResponse>().await.unwrap();

                    Ok(warp::reply::json(&json))
                }
                Ok(response) => {
                    eprintln!("Unexpected response status: {}", response.status());
                    Err(warp::reject())
                }
                Err(e) => {
                    eprintln!("Error making request: {}", e);
                    Err(warp::reject())
                }
            }
        } else {
            Err(warp::reject::not_found())
        }
    }
}

pub mod exchange_url {
    use dotenvy::dotenv;
    use std::env;

    pub fn generate(code: &str, state: &str, verifier: &str) -> String {
        dotenv().ok();
        let url = env::var("RUST_IRIDIUM_BASE_URL").expect("RUST_IRIDIUM_BASE_URL must be set");
        let redirect_uri = env::var("RUST_PUBLIC_IRIDIUM_REDIRECT_URI")
            .expect("RUST_PUBLIC_IRIDIUM_REDIRECT_URI must be set");
        let client_id = env::var("RUST_PUBLIC_IRIDIUM_CLIENT_ID")
            .expect("RUST_PUBLIC_IRIDIUM_CLIENT_ID must be set");

        format!("{}oauth/token?grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}&state={}", url, code, redirect_uri, client_id, verifier, state)
    }
}

pub mod exchange_headers {
    pub fn generate() -> reqwest::header::HeaderMap {
        // ... your logic to generate the headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert("Content-Length", "24".parse().unwrap());
        headers
    }
}
