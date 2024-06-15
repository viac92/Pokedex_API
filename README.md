# Pokedex Project

This project consists of two applications running with Docker Compose:

1. **API**: Built with [Warp](https://github.com/seanmonstar/warp) and Rust.
2. **Single-Page Application**: Built with Vue.js for testing the API.

## API Overview

The API provides two main endpoints:

- **GET /pokemon/{pokemon_name}**:
  - **Description**: Returns detailed information about a Pokémon.
  - **Response**: Includes the Pokémon's name, description, habitat, and whether it is legendary.

- **GET /translated/{pokemon_name}**:
  - **Description**: Returns a translated description of the Pokémon.
  - **Translation Rules**:
    - If the Pokémon's habitat is a cave or it is legendary, the description is translated to Yoda-speak.
    - Otherwise, the description is translated to Shakespearean English.

## Vue.js Application

The Vue.js application serves as a simple interface to interact with and test the API. 

### Features:

- **User Input**: Enter the name of a Pokémon to retrieve its information.
- **Display**: Shows the information returned by the API, including translated descriptions when applicable.

### Usage:

- Enter a Pokémon name in the provided input field.
- The application displays the Pokémon's name, description, habitat, and legendary status.
- It also shows the translated description based on the criteria defined in the API.

## Installation

1. **Clone the Repository**:
    ```sh
    git clone https://github.com/viac92/Pokedex_API.git
    ```

2. **Install Docker**:
    Ensure you have [Docker](https://docs.docker.com/engine/install/) installed on your machine to run the applications.

3. **Navigate to Project Root**:
    After cloning the repository, navigate to the root directory of the project:
    ```sh
    cd Pokedex_API
    ```

4. **Run Docker Compose**:
    Start the application by running:
    ```sh
    docker compose up
    ```

5. **Wait for Setup**:
    Allow Docker to pull and build the necessary containers. This may take a few minutes as it builds the applications.

## Usage

1. **Access the Application**:
    Open your browser and navigate to [http://localhost:5173](http://localhost:5173) to test the API.

2. **Run Unit Tests**:
    To run unit tests inside the Docker container of the API:

    - Access the Docker container:
      ```sh
      docker exec -it pokedex_web_app /bin/sh
      ```

    - Run the tests:
      ```sh
      cargo test
      ```
3. **Use Postman or curl**
    - Call the two end points:
    1. `http://localhost:3030/pokemon/{pokemon_name}`
    2. `http://loaclhost:3030/translated/{pokemon_name}`

## Possible Improvements

For this project, I aimed to keep things straightforward and avoid unnecessary complexity. Here are some improvements I would make if this were a real world application:

- Code Organization: I would use modules to better separate and organize the code, enhancing maintainability and readability.
- Caching: Currently, I implemented a basic cache to avoid calling the external API on every request. This cache does not have an expiration mechanism, which is acceptable for testing. In a real world scenario, I would use a dedicated caching library to manage cache expiration and improve performance.
- Rate Limiting: The current implementation lacks rate limiting. In a real world application, I would implement rate limiting to prevent denial-of-service (DoS) and distributed denial-of-service (DDoS) attacks.
- Logger: Implement logger for debugging and metrics.
- Testing: add integration and end-to-end tests

## Final Toughts 

I chose to use Rust for this project despite my limited experience with the language. Previously, I contributed to an [open-source project](https://github.com/Datagen-Project/Datagen-Substrate-Grant) using Rust for several months. Although I have much to learn, I am excited about the capabilities and potential of Rust.

This is also my first time using Warp. I want to challenge myself and take this opportunity to learn something new!
