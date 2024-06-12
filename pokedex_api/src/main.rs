use reqwest::Error;
use rustemon::{model::resource::FlavorText, Follow};
use serde_json::{json, Value};
use warp::Filter;

async fn get_pokemon(pokemon_name_to_search: String) -> Result<impl warp::Reply, warp::Rejection> {
    let pokemon = fetch_pokemon_from_api(pokemon_name_to_search).await.unwrap();
    Ok(warp::reply::json(&pokemon))
}

async fn get_translated_pokemon(pokemon_name_to_search: String) -> Result<impl warp::Reply, warp::Rejection> {
    let pokemon = fetch_pokemon_from_api(pokemon_name_to_search).await.unwrap();

    let translated_pokemon_description = get_translation(
        pokemon["description"].to_string(), 
        pokemon["habitat"].to_string(), 
        pokemon["is_legendary"].as_bool().unwrap()
    ).await;

    let res = json!({
        "name": pokemon["name"],
        "description": translated_pokemon_description,
        "habitat": pokemon["habitat"],
        "is_legendary": pokemon["is_legendary"]
    });

    Ok(warp::reply::json(&res))
}

async fn fetch_pokemon_from_api(pokemon_name_to_search: String) -> Result<Value, Error> {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let pokemon = rustemon::pokemon::pokemon::get_by_name(&pokemon_name_to_search, &rustemon_client).await;

    let pokemon = pokemon.unwrap();
    let pokemon_name = &pokemon.name;

    let species_resource = pokemon.species;
    let species = species_resource.follow(&rustemon_client).await;

    let species = species.unwrap();
    let pokemon_description = get_english_description(species.flavor_text_entries);
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
    let translated_text = translated_text.replace("\"", "");
    let translated_text = translated_text.replace("  ", " ");
    
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
    let translated_text = translated_text.replace("\"", "");
    let translated_text = translated_text.replace("  ", " ");

    Ok(translated_text)
}

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

async fn get_translation(pokemon_description: String, pokemon_habitat: String, pokemon_is_legendary: bool) -> String {
    if pokemon_habitat == "cave" || pokemon_is_legendary == true {
        fetch_yoda_translation_from_api(pokemon_description).await.unwrap()
    } else {
        fetch_shakespeare_translation_from_api(pokemon_description).await.unwrap()
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

    let routes = warp::get().and(
        pokemon
        .or(translated_pokemon)
    );

    warp::serve(routes)
        // Set the IP address for docker to 0.0.0.0
        .run(([0, 0, 0, 0], 3030))
        .await;
}

// Tests

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
        "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments."
        .to_string()).await.unwrap();

    // The translation lowercase the DNA to dna.
    assert_eq!(translation, "Created by a scientist after years of horrific gene splicing and dna engineering experiments, it was.");
}

#[tokio::test]
async fn test_fetch_yoda_translation_from_api_with_zubat_description() {
    let translation = fetch_yoda_translation_from_api(
        "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets."
        .to_string()).await.unwrap();

    assert_eq!(translation, "Forms colonies in perpetually dark places.Ultrasonic waves to identify and approach targets, uses.");
}

#[tokio::test]
async fn test_fetch_shakespeare_translation_from_api_with_pikachu_description() {
    let translation = fetch_shakespeare_translation_from_api(
        "When several of these POKéMON gather, their electricity could build and cause lightning storms."
        .to_string()).await.unwrap();

    assert_eq!(translation, "At which hour several of these pokémon gather, their electricity couldst buildeth and cause lightning storms.");
}

#[tokio::test]
async fn test_get_english_description_with_flavor_text_entries() {
    use rustemon::model::resource::NamedApiResource;
    use rustemon::model::utility::Language;

    let mut eng_language: NamedApiResource<Language> = rustemon::model::resource::NamedApiResource::default(); 
    eng_language.name = "en".to_string();

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
            language: eng_language,
            version: None
        }
    ];

    let english_description = get_english_description(flavor_text_entries);
    assert_eq!(english_description, "Forms colonies in perpetually dark places. Uses ultrasonic waves to identify and approach targets.");
}
