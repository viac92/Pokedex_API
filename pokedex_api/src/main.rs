use warp::Filter;
use rustemon::Follow;

// use serde::{Deserialize, Serialize};
use serde_json::json;


async fn get_pokemon(param: String) -> Result<impl warp::Reply, warp::Rejection> {

    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemon = rustemon::pokemon::pokemon::get_by_name(&param, &rustemon_client).await;

    let pokemon = pokemon.unwrap();
    let pokemon_name = &pokemon.name;

    let species_resource = pokemon.species;
    let species = species_resource.follow(&rustemon_client).await;

    let species = species.unwrap();

    let pokemon_description = &species.flavor_text_entries[0].flavor_text;
    let pokemon_habitat = &species.habitat.unwrap().name;
    let pokemon_is_legendary = species.is_legendary;

    let res = json!({
        "name": pokemon_name,
        "description": pokemon_description,
        "habitat": pokemon_habitat,
        "is_legendary": pokemon_is_legendary
    });

    Ok(res.to_string())
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let pokemon = warp::get()
        .and(warp::path("pokemon"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_pokemon);

    warp::serve(pokemon)
        // Set the IP address for docker to 0.0.0.0
        .run(([0, 0, 0, 0], 3030))
        .await;
}