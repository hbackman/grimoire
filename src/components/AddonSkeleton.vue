<template>
  <div class="flex items-start gap-3 rounded-xl bg-white p-3 shadow ring-1 ring-black/5
             dark:bg-zinc-900 dark:ring-white/10
             transition-all duration-200 ease-in-out animate-pulse">
    <!-- Image skeleton -->
    <div class="h-14 w-14 flex-shrink-0 rounded-md bg-zinc-200 dark:bg-zinc-700"></div>

    <div class="flex flex-col flex-1">
      <!-- Name skeleton -->
      <div class="h-4 bg-zinc-200 dark:bg-zinc-700 rounded w-32 mb-2"></div>

      <!-- Description skeleton -->
      <div class="space-y-1">
        <div class="h-3 bg-zinc-200 dark:bg-zinc-700 rounded w-full"></div>
        <div class="h-3 bg-zinc-200 dark:bg-zinc-700 rounded w-3/4"></div>
      </div>

      <!-- Author skeleton -->
      <div class="mt-2 flex items-center gap-1">
        <div class="h-3 w-3 bg-zinc-200 dark:bg-zinc-700 rounded"></div>
        <div class="h-3 bg-zinc-200 dark:bg-zinc-700 rounded w-20"></div>
      </div>
    </div>
  </div>
</template>

<script setup>
// No props needed for skeleton
</script>