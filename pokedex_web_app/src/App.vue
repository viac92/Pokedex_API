<script setup>
import axios from 'axios';
import { ref } from 'vue';

let pokemon = ref(null);
let pokemon_name = ref('');

let translated_pokemon = ref(null);
let translated_pokemon_name = ref('');

const fetchPokemon = async (pokemon_name) => {
  try {
    const response = await axios.get(`http://127.0.0.1:3030/pokemon/${pokemon_name}`);
    console.log(response.data);
    pokemon.value = response.data;
  } catch(error) {
    console.error(error);
    pokemon.value = error.response.data.error;
  }
}

const fetchTranslatedPokemon = async (translated_pokemon_name) => {
  try {
    const response = await axios.get(`http://127.0.0.1:3030/translated/${translated_pokemon_name}`);
    console.log(response.data);
    translated_pokemon.value = response.data;
  } catch(error) {
    console.error(error);
    translated_pokemon.value  = error.response.data.error;
  }
}

</script>

<template>
  <main>
    <h1>Pokedex</h1>
    <p>Get Your Pokemon!</p>
    <input type="text" v-model="pokemon_name" />
    <button class="button-margin" @click="fetchPokemon(pokemon_name)" :disabled="pokemon_name === ''">Get Pokemon Information</button>
    <p>name: {{ pokemon?.name }}</p>
    <p>description: {{ pokemon?.description }}</p>
    <p>habitat: {{ pokemon?.habitat }}</p>
    <p>is_legendary: {{ pokemon?.is_legendary }}</p>

    <p>Get your translated Pokemon!</p>
    <input type="text" v-model="translated_pokemon_name" />
    <button class="button-margin" @click="fetchTranslatedPokemon(translated_pokemon_name)" :disabled="translated_pokemon_name === ''">Get Translated Pokemon Information</button>
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
</style>