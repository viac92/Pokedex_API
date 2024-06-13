use reqwest::Error;
use rustemon::{model::resource::FlavorText, Follow};
use serde_json::{json, Value};
use warp::Filter;

////////////
// Routes //
////////////

async fn get_pokemon(pokemon_name_to_search: String) -> Result<impl warp::Reply, warp::Rejection> {
    let pokemon = fetch_pokemon_from_api(pokemon_name_to_search).await;

    // Suppose the only error is the pokemon not found, we should handle all possible errors in real world.
    if pokemon.is_err() {
        let reply = warp::reply::json(&json!({
            "error": "Pokemon not found"
        }));
        return Ok(warp::reply::with_status(reply, warp::http::StatusCode::NOT_FOUND));
    };

    let reply = warp::reply::json(&pokemon.unwrap());
    Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
}

async fn get_translated_pokemon(pokemon_name_to_search: String) -> Result<impl warp::Reply, warp::Rejection> {
    let pokemon = fetch_pokemon_from_api(pokemon_name_to_search).await;

    // Suppose the only error is the pokemon not found, we should handle all possible errors in real world.
    if pokemon.is_err() {
        let reply = warp::reply::json(&json!({
            "error": "Pokemon not found"
        }));
        return Ok(warp::reply::with_status(reply, warp::http::StatusCode::NOT_FOUND));
    };

    let mut pokemon = pokemon.unwrap();
    let translated_pokemon_description = get_translation(
        pokemon["description"].as_str().unwrap(), 
        pokemon["habitat"].to_string(), 
        pokemon["is_legendary"].as_bool().unwrap()
    ).await;
    
    // Suppose the only error is the rate limit reached, return a 429 status code.
    // In real world, we should handle all possible errors.
    if translated_pokemon_description.is_err() {
        let reply = warp::reply::json(&json!({
            "error": "Translation failed"
        }));
        return Ok(warp::reply::with_status(reply, warp::http::StatusCode::TOO_MANY_REQUESTS));
    };

    let translated_pokemon_description = translated_pokemon_description.unwrap();

    if let Some(description) = pokemon.get_mut("description") {
        *description = json!(translated_pokemon_description);
    }

    let reply = warp::reply::json(&pokemon);
    Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
}

////////////////////////////////////
// Interaction with external APIs //
////////////////////////////////////

async fn fetch_pokemon_from_api(pokemon_name_to_search: String) -> Result<Value, rustemon::error::Error> {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemon = rustemon::pokemon::pokemon::get_by_name(&pokemon_name_to_search, &rustemon_client).await?;

    let species_resource = pokemon.species;
    let species = species_resource.follow(&rustemon_client).await.unwrap(); // Suppose to be safe to unwrap, in real world, we should handle the error

    let pokemon_description = get_english_description(species.flavor_text_entries);
    let pokemon_description = pokemon_description.replace("\n", " ");
    let pokemon_description = pokemon_description.replace("\x0C", " ");

    let res = json!({
        "name": &pokemon.name,
        "description": pokemon_description,
        "habitat": &species.habitat.unwrap().name, // Suppose to be safe to unwrap, in real world, we should handle the error
        "is_legendary": species.is_legendary
    });

    Ok(res)
}

async fn fetch_yoda_translation_from_api(pokemon_description: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let res = client.post("https://api.funtranslations.com/translate/yoda")
        .body(format!("{{\"text\": \"{}\"}}", pokemon_description))
        .send()
        .await?;

    println!("{:?}", res);
    
    // The https://api.funtranslations.com/translate/yoda API has a rate limit of 10 requests per hour and 60 requests per day. 
    // If the rate limit is reached, the API will return a 429 status code.
    // Return an error if the rate limit is reached.
    if res.status() == 429 {
        return Err(Error::without_url(res.error_for_status().err().unwrap()));
    }

    let data: serde_json::Value = res.json().await.unwrap();
    let translated_text = data["contents"]["translated"].as_str().unwrap().to_string();
    let translated_text = translated_text.replace("  ", " ");
    
    Ok(translated_text)
}

async fn fetch_shakespeare_translation_from_api(pokemon_description: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let res = client.post("https://api.funtranslations.com/translate/shakespeare")
        .body(format!("{{\"text\": \"{}\"}}", pokemon_description))
        .send()
        .await?;
    
    println!("{:?}", res);

    if res.status() == 429 {
        return Err(Error::without_url(res.error_for_status().err().unwrap()));
    }

    let data: serde_json::Value = res.json().await.unwrap();
    let translated_text = data["contents"]["translated"].as_str().unwrap().to_string();
    let translated_text = translated_text.replace("  ", " ");

    Ok(translated_text)
}

///////////////////////
// Utility functions //
///////////////////////

fn get_english_description(language_array: Vec<FlavorText>) -> String {
    let mut english_translation = String::new();
    for entry in language_array {
        if entry.language.name == "en" {
            english_translation = entry.flavor_text;
            break;
        }
    }
    english_translation
}

async fn get_translation(pokemon_description: &str, pokemon_habitat: String, pokemon_is_legendary: bool) -> Result<String, Error> {
    if pokemon_habitat == "cave" || pokemon_is_legendary == true {
        fetch_yoda_translation_from_api(pokemon_description).await
    } else {
        fetch_shakespeare_translation_from_api(pokemon_description).await
    }
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

    let routes = warp::get()
        .and(pokemon
        .or(translated_pokemon)
    );

    warp::serve(routes)
        // Set the IP address for docker to 0.0.0.0
        .run(([0, 0, 0, 0], 3030))
        .await;
}

///////////
// Tests //
///////////

#[tokio::test]
async fn test_fetch_pokemon_from_api_with_common_pokemon() {
    let pokemon = fetch_pokemon_from_api("pikachu".to_string()).await.unwrap();
    assert_eq!(pokemon["name"], "pikachu");
    assert_eq!(pokemon["habitat"], "forest");
    assert_eq!(pokemon["is_legendary"], false);
    assert_eq!(pokemon["description"], "When several of these POKéMON gather, their electricity could build and cause lightning storms.");
}

#[tokio::test]
async fn test_fetch_pokemon_from_api_with_legendary_pokemon() {
    let pokemon = fetch_pokemon_from_api("mewtwo".to_string()).await.unwrap();
    assert_eq!(pokemon["name"], "mewtwo");
    assert_eq!(pokemon["habitat"], "rare");
    assert_eq!(pokemon["is_legendary"], true);
    assert_eq!(pokemon["description"], "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.");
}

#[tokio::test]
async fn test_fetch_pokemon_from_api_with_cave_pokemon() {
    let pokemon = fetch_pokemon_from_api("zubat".to_string()).await.unwrap();
    assert_eq!(pokemon["name"], "zubat");
    assert_eq!(pokemon["habitat"], "cave");
    assert_eq!(pokemon["is_legendary"], false);
    assert_eq!(pokemon["description"], "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.");
}

#[tokio::test]
async fn test_fetch_yoda_translation_from_api_with_mewtwo_description() {
    let translation = fetch_yoda_translation_from_api(
        "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.").await.unwrap();

    // The translation lowercase the DNA to dna.
    assert_eq!(translation, "Created by a scientist after years of horrific gene splicing and dna engineering experiments, it was.");
}

#[tokio::test]
async fn test_fetch_yoda_translation_from_api_with_zubat_description() {
    let translation = fetch_yoda_translation_from_api(
        "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.").await.unwrap();

    assert_eq!(translation, "Forms colonies in perpetually dark places.Ultrasonic waves to identify and approach targets, uses.");
}

#[tokio::test]
async fn test_fetch_shakespeare_translation_from_api_with_pikachu_description() {
    let translation = fetch_shakespeare_translation_from_api(
        "When several of these POKéMON gather, their electricity could build and cause lightning storms.").await.unwrap();

    assert_eq!(translation, "At which hour several of these pokémon gather, their electricity couldst buildeth and cause lightning storms.");
}

#[tokio::test]
async fn test_get_english_description_with_flavor_text_entries() {
    use rustemon::model::resource::NamedApiResource;
    use rustemon::model::utility::Language;

    let mut en_language: NamedApiResource<Language> = rustemon::model::resource::NamedApiResource::default(); 
    en_language.name = "en".to_string();

    let mut zh_language: NamedApiResource<Language> = rustemon::model::resource::NamedApiResource::default();
    zh_language.name = "zh-Hant".to_string();

    let flavor_text_entries = vec![
        FlavorText {
            flavor_text: "因為沒有眼珠所以看不見東西。會從口中發出超音波 來探測周圍的狀況".to_string(),
            language: zh_language,
            version: None
        },
        FlavorText {
            flavor_text: "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.".to_string(),
            language: en_language,
            version: None
        }
    ];

    let english_description = get_english_description(flavor_text_entries);
    assert_eq!(english_description, "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.");
}

#[tokio::test]
async fn test_get_translation_with_cave_pokemon() {
    let translation = get_translation(
        "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.",
        "cave".to_string(),
        false
    ).await.unwrap();

    assert_eq!(translation, "Forms colonies in perpetually dark places.Ultrasonic waves to identify and approach targets, uses.");
}

#[tokio::test]
async fn test_get_translation_with_legendary_pokemon() {
    let translation = get_translation(
        "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.",
        "rare".to_string(),
        true
    ).await.unwrap();

    assert_eq!(translation, "Created by a scientist after years of horrific gene splicing and dna engineering experiments, it was.");
}

#[tokio::test]
async fn test_get_translation_with_common_pokemon() {
    let translation = get_translation(
        "When several of these POKéMON gather, their electricity could build and cause lightning storms.",
        "forest".to_string(),
        false
    ).await.unwrap();

    assert_eq!(translation, "At which hour several of these pokémon gather, their electricity couldst buildeth and cause lightning storms.");
}

#[tokio::test]
async fn test_get_pokemon() {
    let f = warp::path("pokemon")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_pokemon);

    let res = warp::test::request().path("/pokemon/pikachu").reply(&f).await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), 
        "{\"description\":\"When several of these POKéMON gather, their electricity could build and cause lightning storms.\",\"habitat\":\"forest\",\"is_legendary\":false,\"name\":\"pikachu\"}"
    );
}

#[tokio::test]
async fn test_get_pokemon_not_found() {
    let f = warp::path("pokemon")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_pokemon);

    let res = warp::test::request().path("/pokemon/NoPokemon").reply(&f).await;

    assert_eq!(res.status(), 404);
    assert_eq!(res.body(), "{\"error\":\"Pokemon not found\"}");
}

#[tokio::test]
async fn test_get_translated_pokemon() {
    let f = warp::path("translated")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_translated_pokemon);

    let res = warp::test::request().path("/translated/pikachu").reply(&f).await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), 
        "{\"description\":\"At which hour several of these pokémon gather, their electricity couldst buildeth and cause lightning storms.\",\"habitat\":\"forest\",\"is_legendary\":false,\"name\":\"pikachu\"}"
    );
}

#[tokio::test]
async fn test_get_translated_pokemon_not_found() {
    let f = warp::path("translated")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_translated_pokemon);

    let res = warp::test::request().path("/translated/NoPokemon").reply(&f).await;

    assert_eq!(res.status(), 404);
    assert_eq!(res.body(), "{\"error\":\"Pokemon not found\"}");
}