use reqwest::Error;
use warp::Filter;
use rustemon::Follow;

// use serde::{Deserialize, Serialize};
use serde_json::{json, Value};


async fn get_pokemon(pokemon_name_to_search: String) -> Result<impl warp::Reply, warp::Rejection> {
    let pokemon = fetch_pokemon_from_api(pokemon_name_to_search).await.unwrap();
    Ok(warp::reply::json(&pokemon))
}

async fn get_translated_pokemon(pokemon_name_to_search: String) -> Result<impl warp::Reply, warp::Rejection> {
    let pokemon = fetch_pokemon_from_api(pokemon_name_to_search).await.unwrap();

    let pokemon_description = pokemon["description"].as_str().unwrap(); 

    if pokemon["habitat"] == "cave" || pokemon["is_legendary"] == true {
        // Translate the description to Yoda

        let translated_pokemon_description = fetch_yoda_translation_from_api(pokemon_description.to_string()).await.unwrap();
        let translated_pokemon_description = translated_pokemon_description.replace("\"", "");
        let translated_pokemon_description = translated_pokemon_description.replace("  ", " ");

        let res = json!({
            "name": pokemon["name"],
            "description": translated_pokemon_description,
            "habitat": pokemon["habitat"],
            "is_legendary": pokemon["is_legendary"]
        });

        Ok(warp::reply::json(&res))
    } else {
        // Translate the description to Shakespeare

        let translated_pokemon_description = fetch_shakespeare_translation_from_api(pokemon_description.to_string()).await.unwrap();
        let translated_pokemon_description = translated_pokemon_description.replace("\"", "");
        let translated_pokemon_description = translated_pokemon_description.replace("  ", " ");

        let res = json!({
            "name": pokemon["name"],
            "description": translated_pokemon_description,
            "habitat": pokemon["habitat"],
            "is_legendary": pokemon["is_legendary"]
        });

        Ok(warp::reply::json(&res))
    }
}

async fn fetch_pokemon_from_api(pokemon_name_to_search: String) -> Result<Value, Error> {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemon = rustemon::pokemon::pokemon::get_by_name(&pokemon_name_to_search, &rustemon_client).await;

    let pokemon = pokemon.unwrap();
    let pokemon_name = &pokemon.name;

    let species_resource = pokemon.species;
    let species = species_resource.follow(&rustemon_client).await;

    let species = species.unwrap();

    let pokemon_description = &species.flavor_text_entries[0].flavor_text;
    let pokemon_description = pokemon_description.replace("\n", " ");
    let pokemon_description = pokemon_description.replace("\x0C", " ");

    let pokemon_habitat = &species.habitat.unwrap().name;
    let pokemon_is_legendary = species.is_legendary;

    let res = json!({
        "name": pokemon_name,
        "description": pokemon_description,
        "habitat": pokemon_habitat,
        "is_legendary": pokemon_is_legendary
    });

    Ok(res)
}

async fn fetch_yoda_translation_from_api(pokemon_description: String) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let res = client.post("https://api.funtranslations.com/translate/yoda")
        .body(format!("{{\"text\": \"{}\"}}", pokemon_description))
        .send()
        .await?;

    println!("{:?}", res);

    let data: serde_json::Value = res.json().await.unwrap();
    let translated_text = data["contents"]["translated"].to_string();
    
    Ok(translated_text)
}

async fn fetch_shakespeare_translation_from_api(pokemon_description: String) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let res = client.post("https://api.funtranslations.com/translate/shakespeare")
        .body(format!("{{\"text\": \"{}\"}}", pokemon_description))
        .send()
        .await?;

    println!("{:?}", res);

    let data: serde_json::Value = res.json().await.unwrap();
    let translated_text = data["contents"]["translated"].to_string();
    
    Ok(translated_text)
}

#[tokio::main]
async fn main() {
    let pokemon = warp::get()
        .and(warp::path("pokemon"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_pokemon);

    let translated_pokemon = warp::get()
        .and(warp::path("translated"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_translated_pokemon);

    let routes = warp::get().and(
        pokemon
        .or(translated_pokemon)
    );

    warp::serve(routes)
        // Set the IP address for docker to 0.0.0.0
        .run(([0, 0, 0, 0], 3030))
        .await;
}