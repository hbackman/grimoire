<template>
  <div
    class="flex w-full items-center gap-2 rounded-xl bg-white p-2 shadow ring-1 ring-black/5
          dark:bg-zinc-900 dark:ring-white/10
          focus-within:ring-2 focus-within:ring-blue-500 focus-within:shadow-lg focus-within:scale-105
          transition-all duration-200 ease-in-out">
    <!-- Search icon -->
    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-zinc-400 dark:text-zinc-500" viewBox="0 0 24 24" fill="currentColor">
      <path
        d="M10 2a8 8 0 015.292 13.708l4 4a1 1 0 01-1.414 1.414l-4-4A8 8 0 1110 2zm0 2a6 6 0 100 12A6 6 0 0010 4z"
      />
    </svg>
    <!-- Input -->
    <input
      ref="searchInput"
      :value="modelValue"
      @input="$emit('update:modelValue', $event.target.value)"
      @keydown.enter="$event.target.blur()"
      @keydown.escape="$event.target.blur()"
      type="text"
      placeholder="Search addons..."
      class="w-full bg-transparent text-sm text-zinc-900 placeholder-zinc-400
            focus:outline-none dark:text-zinc-100 dark:placeholder-zinc-500"
    />
  </div>
</template>

<script setup>
import {defineProps, defineEmits, ref, onMounted, onUnmounted} from "vue";

const searchInput = ref(null);

defineProps({
  modelValue: String,
});

defineEmits([
  "update:modelValue",
]);

const handleKeydown = (event) => {
  // Focus search when "/" is pressed, but not when typing in an input
  if (event.key === '/' && event.target.tagName !== 'INPUT') {
    event.preventDefault();
    searchInput.value?.focus();
  }
};

onMounted(() => {
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
});
</script>
