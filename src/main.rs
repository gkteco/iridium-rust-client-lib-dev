use std::collections::HashMap;
use iridium_rust_client_lib::{authenticate_with_external_redirect, get_identity};
use warp::Filter;

use iridium_rust_client_lib::callback_service;
use iridium_rust_client_lib::User;

#[tokio::main]
async fn main() {
    let verifier = iridium_rust_client_lib::generate_random_string();
    let auth_verifier = verifier.clone();
    let callback_verifier = verifier.clone();
    //authenticate
    let auth = warp::path!("auth").map( move || {
        let uri = authenticate_with_external_redirect(auth_verifier.clone()).unwrap();
        warp::redirect(uri)
    });
    //call back
    let callback = warp::path!("callback")
        .and(warp::query::<HashMap<String, String>>())
        .and_then(move |params: HashMap<String, String>| {
            let response = callback_service::callback_service::handle_callback(params, callback_verifier.clone());
            async {
                if let Ok(res) = response.await {
                    if let Ok(user) = get_identity(&res.token).await {
                        println!("user id: {}, username: {}", user.data.id, user.data.username);
                        Ok(user)
                    } else {
                        eprintln!("Error getting identity");
                        Err(warp::reject())
                    }

                } else {
                    eprintln!("Error handling callback");
                    Err(warp::reject())
                }
            }
        }
        );

    let routes = auth.or(callback);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
