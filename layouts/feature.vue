<script setup>
defineProps({
  heading: String,
  subtitle: String,
})
</script>

<template>
  <div class="slidev-layout feature-layout">
    <div class="watermark">Monocle AI</div>

    <div class="content-area">
      <h1 class="slide-title">{{ heading }}</h1>
      <p v-if="subtitle" class="slide-subtitle">{{ subtitle }}</p>
      <div class="slot-area">
        <slot />
      </div>
    </div>

    <div class="footer-logo"><i>W</i><span>armblood</span></div>
  </div>
</template>

<style scoped>
.feature-layout {
  padding: 40px;
  display: flex;
  flex-direction: column;
}

.content-area {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.slot-area {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
  overflow: hidden;
}
</style>
