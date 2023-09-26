mod callback_service;
mod pkce_service;
mod state_generator_service;
mod token_service;
mod url_generator_service;

use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use tokio;
use warp::http::{HeaderMap, Uri};
use warp::{Filter};
use warp::Reply;





#[tokio::main]
pub async fn authenticate_with_external_redirect() -> () {
    let state = state_generator_service::state_generator::generate();
    let verifier = generate_random_string();
    let pkce_code = pkce_service::pkce_service::generate_code_challenge(&verifier);

    if let Ok(code_challenge) = pkce_code {
        let auth = warp::path!("auth").map(move || {
            let uri = Uri::from_str(
                &url_generator_service::url_generator_service::get_iridium_auth_url(
                    &state,
                    &code_challenge,
                ),
            )
            .unwrap();
            warp::redirect(uri)
        });

        //call back
        let callback = warp::path!("callback")
            .and(warp::query::<HashMap<String, String>>())
            .and_then(move |params: HashMap<String, String>| {
                callback_service::callback_service::handle_callback(params, verifier.clone())
            }
            );

        let routes = auth.or(callback);

        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    } else {
        eprintln!("Error generating code challenge")
    }
}

pub async fn get_identity(token: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let base_url = env::var("RUST_IRIDIUM_BASE_URL").expect("RUST_IRIDIUM_BASE_URL must be set");
    let identities_url = format!("{}identities", base_url);
    let mut headers = HeaderMap::new();
    let bearer = format!("Bearer {}", token);
    headers.insert(
        "Accept",
        "application/vnd.iridium.id.identity-response.1+json"
            .parse()
            .unwrap(),
    );

    headers.insert("Authorization", bearer.parse().unwrap());

    client.get(&identities_url).headers(headers).send().await
}
fn generate_random_string() -> String {
    let random_string: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    random_string
}
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
