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
  }
}

const fetchTranslatedPokemon = async (translated_pokemon_name) => {
  try {
    const response = await axios.get(`http://127.0.0.1:3030/translated/${translated_pokemon_name}`);
    console.log(response.data);
    translated_pokemon.value = response.data;
  } catch(error) {
    console.error(error);
  }
}

</script>

<template>
  <main>
    <h1>Pokedex</h1>
    <!-- Add text box to choose a pokemon store it in variable-->

    <p>Get Your Pokemon!</p>
    <input type="text" v-model="pokemon_name" />
    <button @click="fetchPokemon(pokemon_name)">Get Pokemon Information</button>
    {{ pokemon }}

    <p>Get your translated Pokemon!</p>
    <input type="text" v-model="translated_pokemon_name" />
    <button @click="fetchTranslatedPokemon(translated_pokemon_name)">Get Translated Pokemon Information</button>
    {{ translated_pokemon }}
  </main>
</template>

<style scoped>

</style>