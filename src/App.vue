<template>
  <main>
    hello
  </main>
</template>

<script setup>
import {onMounted} from "vue";
import {scrape} from "@/lib/curseforge.js";

onMounted(() => {
  scrape("https://www.curseforge.com/wow/search?page=1&pageSize=20&sortBy=relevancy&class=addons")
    .then(r => {
      console.log("scrape", r);
    })
    .catch(() => {
      console.error("error");
    })
});
</script>
