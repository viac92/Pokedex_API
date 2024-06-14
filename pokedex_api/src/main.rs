use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use reqwest::Error;
use rustemon::{model::resource::FlavorText, Follow};
use serde_json::{json, Value};
use warp::Filter;

////////////
// Routes //
////////////

/// Get the data for the pokemon/pokemon_name endpoint.
/// 
/// The endpoint will return the pokemon data as a JSON object.
/// - name: String
/// - description: String
/// - habitat: String
/// - is_legendary: bool
/// 
/// The endpoint will cache the pokemon data.
async fn get_pokemon(pokemon_name_to_search: String, cache: Arc<Mutex<HashMap<String, Value>>>) -> Result<impl warp::Reply, warp::Rejection> { 
    {
        let cache_guard = cache.lock().unwrap();
        if cache_guard.contains_key(&pokemon_name_to_search) {
            let reply = warp::reply::json(&cache_guard[&pokemon_name_to_search]);
            return Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK));
        }
    } // MutexGuard is dropped here

    let pokemon = fetch_pokemon_from_api(pokemon_name_to_search.clone()).await;

    // Suppose the only error is the pokemon not found, we should handle all possible errors in real world.
    if pokemon.is_err() {
        let reply = warp::reply::json(&json!({
            "error": "Pokemon not found"
        }));
        return Ok(warp::reply::with_status(reply, warp::http::StatusCode::NOT_FOUND));
    }

    let pokemon = pokemon.unwrap();

    cache.lock().unwrap().insert(pokemon_name_to_search, pokemon.clone());

    let reply = warp::reply::json(&pokemon);
    Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
}

/// Get the data for the translated/pokemon_name endpoint.
/// 
/// The endpoint will return the pokemon data with the description translated as a JSON object.
/// - name: String
/// - description: String
/// - habitat: String
/// - is_legendary: bool
/// 
/// The endpoint will cache the pokemon data.
async fn get_translated_pokemon(pokemon_name_to_search: String, cache_pokemon: Arc<Mutex<HashMap<String, Value>>>, cache_translation: Arc<Mutex<HashMap<String, String>>>) -> Result<impl warp::Reply, warp::Rejection> {
    // Get the pokemon data from the cache or fetch from the API
    let pokemon_data = get_pokemon_from_cache(pokemon_name_to_search.clone(), cache_pokemon.clone());
    let mut pokemon;
    if pokemon_data.is_some() {
        pokemon = pokemon_data.unwrap();
    } else {
        let pokemon_result = fetch_pokemon_from_api(pokemon_name_to_search.clone()).await;

        // Suppose the only error is the pokemon not found, we should handle all possible errors in real world.
        if pokemon_result.is_err() {
            let reply = warp::reply::json(&json!({
                "error": "Pokemon not found"
            }));
            return Ok(warp::reply::with_status(reply, warp::http::StatusCode::NOT_FOUND));
        };

        let pokemon_result = Some(pokemon_result.unwrap());
        cache_pokemon.lock().unwrap().insert(pokemon_name_to_search.clone(), pokemon_result.clone().unwrap());
        pokemon = pokemon_result.unwrap();        
    }

    let translation_in_cache: Option<String> = get_translation_from_cache(pokemon_name_to_search.clone(), cache_translation.clone());

    // Get the translation from the cache or fetch from the API
    if translation_in_cache.is_some() {
        let translated_pokemon_description = translation_in_cache.unwrap();

        if let Some(description) = pokemon.get_mut("description") {
            *description = json!(translated_pokemon_description);
        }

        let reply = warp::reply::json(&pokemon);
        return Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK));
    }

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

    cache_translation.lock().unwrap().insert(pokemon_name_to_search.clone(), translated_pokemon_description.clone());

    let reply = warp::reply::json(&pokemon);
    Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
}

////////////////////////////////////
// Interaction with external APIs //
////////////////////////////////////

/// Fetch the pokemon data from the PokeAPI.
/// 
/// In real world application, I should handle all possible errors, here I just return an error if the pokemon is not found.
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

/// Fetch the Yoda translation from the Fun Translations API.
/// 
/// The Yoda API has a rate limit of 10 requests per hour and 60 requests per day.
/// Be careful with the rate limit!
/// 
/// The API will return a 429 status code if the rate limit is reached.
/// 
/// In real world application, I should handle all possible errors, here I just return an error if the rate limit is reached.
/// Also I will consider using API keys to increase the rate limit.
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

/// Fetch the translation from the Shakespeare API.
/// 
/// The Shakespeare API has a rate limit of 10 requests per hour and 60 requests per day.
/// Be careful with the rate limit!
/// 
/// The API will return a 429 status code if the rate limit is reached.
/// 
/// In real world application, I should handle all possible errors, here I just return an error if the rate limit is reached.
/// Also I will consider using API keys to increase the rate limit.
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

/// Get the first english description from the flavor text entries.
/// 
/// Some pokemon have multiple descriptions in different languages, this function will return the first english description.
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

/// Get the correct translation based on the pokemon habitat and if the pokemon is legendary.
/// 
/// if the pokemon habitat is cave or the pokemon is legendary, the translation will be in Yoda.
/// Otherwise, the translation will be in Shakespeare.
async fn get_translation(pokemon_description: &str, pokemon_habitat: String, pokemon_is_legendary: bool) -> Result<String, Error> {
    if pokemon_habitat == "cave" || pokemon_is_legendary == true {
        fetch_yoda_translation_from_api(pokemon_description).await
    } else {
        fetch_shakespeare_translation_from_api(pokemon_description).await
    }
}

/// Cache the pokemon in a HashMap with the pokemon name as the key.
/// 
/// In real world application I should use a cache library like Redis.
/// Actually I cache the pokemon for unlimited time, in real world I should set a TTL.
fn get_pokemon_from_cache(pokemon_name: String, cache: Arc<Mutex<HashMap<String, Value>>>) -> Option<Value> {
    let cache_guard = cache.lock().unwrap();
    if cache_guard.contains_key(&pokemon_name) {
        return Some(cache_guard[&pokemon_name].clone());
    }
    None
}

/// Cache the translation in a HashMap with the pokemon name as the key.
/// 
/// In real world application I should use a cache library like Redis.
/// Actually I cache the translation for unlimited time, in real world I should set a TTL.
fn get_translation_from_cache(pokemon_name: String, cache: Arc<Mutex<HashMap<String, String>>>) -> Option<String> {
    let cache_guard = cache.lock().unwrap();
    if cache_guard.contains_key(&pokemon_name) {
        return Some(cache_guard[&pokemon_name].clone());
    }
    None
}

#[tokio::main]
async fn main() {
    let pokemon_cache: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    let translation_cache: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new())); 

    let pokemon_cache_clone = Arc::clone(&pokemon_cache);

    let pokemon = warp::get()
        .and(warp::path("pokemon"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::any().map(move || pokemon_cache.clone()))
        .and_then(get_pokemon);

    let translated_pokemon = warp::get()
        .and(warp::path("translated"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::any().map(move || pokemon_cache_clone.clone()))
        .and(warp::any().map(move || translation_cache.clone()))
        .and_then(get_translated_pokemon);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET"]);

    let routes = warp::get()
        .and(pokemon
        .or(translated_pokemon)
        .with(cors)
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
async fn test_get_pokemon_from_cache() {
    let pokemon_cache: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));

    let pokemon = fetch_pokemon_from_api("pikachu".to_string()).await.unwrap();
    pokemon_cache.lock().unwrap().insert("pikachu".to_string(), pokemon.clone());

    let pokemon_from_cache = get_pokemon_from_cache("pikachu".to_string(), pokemon_cache.clone());
    assert_eq!(pokemon_from_cache.unwrap(), pokemon);
}

#[tokio::test]
async fn test_get_translation_from_cache() {
    let translation_cache: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let translation = fetch_shakespeare_translation_from_api(
        "When several of these POKéMON gather, their electricity could build and cause lightning storms.").await.unwrap();
    translation_cache.lock().unwrap().insert("pikachu".to_string(), translation.clone());

    let translation_from_cache = get_translation_from_cache("pikachu".to_string(), translation_cache.clone());
    assert_eq!(translation_from_cache.unwrap(), translation);
}

#[tokio::test]
async fn test_get_pokemon() {
    let pokemon_cache: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));

    let f = warp::path("pokemon")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::any().map(move || pokemon_cache.clone()))
        .and_then(get_pokemon);

    let res = warp::test::request().path("/pokemon/pikachu").reply(&f).await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), 
        "{\"description\":\"When several of these POKéMON gather, their electricity could build and cause lightning storms.\",\"habitat\":\"forest\",\"is_legendary\":false,\"name\":\"pikachu\"}"
    );
}

#[tokio::test]
async fn test_get_pokemon_not_found() {
    let pokemon_cache: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));

    let f = warp::path("pokemon")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::any().map(move || pokemon_cache.clone()))
        .and_then(get_pokemon);

    let res = warp::test::request().path("/pokemon/NoPokemon").reply(&f).await;

    assert_eq!(res.status(), 404);
    assert_eq!(res.body(), "{\"error\":\"Pokemon not found\"}");
}

#[tokio::test]
async fn test_get_translated_pokemon() {
    let pokemon_cache: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    let translation_cache: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let f = warp::path("translated")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::any().map(move || pokemon_cache.clone()))
        .and(warp::any().map(move || translation_cache.clone()))
        .and_then(get_translated_pokemon);

    let res = warp::test::request().path("/translated/pikachu").reply(&f).await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), 
        "{\"description\":\"At which hour several of these pokémon gather, their electricity couldst buildeth and cause lightning storms.\",\"habitat\":\"forest\",\"is_legendary\":false,\"name\":\"pikachu\"}"
    );
}

#[tokio::test]
async fn test_get_translated_pokemon_not_found() {
    let pokemon_cache: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    let translation_cache: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let f = warp::path("translated")
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::any().map(move || pokemon_cache.clone()))
        .and(warp::any().map(move || translation_cache.clone()))
        .and_then(get_translated_pokemon);

    let res = warp::test::request().path("/translated/NoPokemon").reply(&f).await;

    assert_eq!(res.status(), 404);
    assert_eq!(res.body(), "{\"error\":\"Pokemon not found\"}");
}