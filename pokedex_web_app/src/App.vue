<script setup>
import axios from 'axios';
import { ref } from 'vue';

let pokemon = ref(null);
let pokemon_name = ref('');
let pokemon_error = ref('');

let translated_pokemon = ref(null);
let translated_pokemon_name = ref('');
let translated_pokemon_error = ref('');

const fetchPokemon = async (pokemon_name) => {
  try {
    const response = await axios.get(`http://127.0.0.1:3030/pokemon/${pokemon_name}`);
    console.log(response.data);
    pokemon.value = response.data;
    pokemon_error.value = '';
  } catch(error) {
    console.error(error);
    pokemon_error.value = error.response.data.error;
  }
}

const fetchTranslatedPokemon = async (translated_pokemon_name) => {
  try {
    const response = await axios.get(`http://127.0.0.1:3030/translated/${translated_pokemon_name}`);
    console.log(response.data);
    translated_pokemon.value = response.data;
    translated_pokemon_error.value = '';
  } catch(error) {
    console.error(error);
    translated_pokemon_error.value  = error.response.data.error;
  }
}

</script>

<template>
  <main>
    <h1>Pokedex</h1>
    <h3>Get Your Pokemon!</h3>
    <input type="text" v-model="pokemon_name" />
    <button class="button-margin" @click="fetchPokemon(pokemon_name)" :disabled="pokemon_name === ''">Get Pokemon Information</button>
    <div v-if="pokemon_error">
      <p class="error"> error: {{ pokemon_error }} </p>
    </div>
    <p>name: {{ pokemon?.name }}</p>
    <p>description: {{ pokemon?.description }}</p>
    <p>habitat: {{ pokemon?.habitat }}</p>
    <p>is_legendary: {{ pokemon?.is_legendary }}</p>


    <h3>Get your translated Pokemon!</h3>
    <input type="text" v-model="translated_pokemon_name" />
    <button class="button-margin" @click="fetchTranslatedPokemon(translated_pokemon_name)" :disabled="translated_pokemon_name === ''">Get Translated Pokemon Information</button>
    <div v-if="translated_pokemon_error">
      <p class="error"> error: {{ translated_pokemon_error }} </p>
    </div>
    <p>name: {{ translated_pokemon?.name }}</p>
    <p>translated description: {{ translated_pokemon?.description }}</p>
    <p>habitat: {{ translated_pokemon?.habitat }}</p>
    <p>is_legendary: {{ translated_pokemon?.is_legendary }}</p>
  </main>
</template>

<style scoped>
.button-margin {
  margin-left: 10px;
}
.error {
  color: red;
}
</style>