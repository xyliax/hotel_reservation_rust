use config::*;
use mongodb::bson::document::ValueAccessError;
use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, Client, Collection};
use profile::pb::profile_api::Comment;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use user::pb::user_api::UserProfile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
    mongo_client_options.app_name = Some("test_worker".to_owned());
    let user_coll = Client::with_options(mongo_client_options)?
        .database(mongo_svc::DB)
        .collection::<Document>(mongo_svc::coll::USER_PROFILE);
    initialize_database(&user_coll).await;
    Ok(())
}

/**
 * Initialize user_profile collection with SAMPLES documents
 */
async fn initialize_database(coll: &Collection<Document>) {
    const SAMPLES: i32 = 10_000;
    let mut checked_name = HashSet::<String>::new();
    let mut docs = Vec::<Document>::new();
    let mut count = SAMPLES;
    loop {
        let username = format!("user_{}", generate_string(10));
        let password = generate_string(20);
        if checked_name.contains(&username) {
            continue;
        }
        checked_name.insert(username.clone());
        println!("{}", username.clone());
        docs.push(doc! {
            mongo_svc::user::USERNAME: username,
            mongo_svc::user::PASSWORD: password,
            mongo_svc::user::LOCATION: "Hong Kong",
            mongo_svc::user::FAVORITE: "Unknown",
        });
        count -= 1;
        if count <= 0 {
            break;
        }
    }
    match coll.insert_many(docs, None).await {
        Ok(res) => println!("{} users written to database!", res.inserted_ids.len()),
        Err(err) => eprintln!("{err}"),
    }
}

fn generate_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}
