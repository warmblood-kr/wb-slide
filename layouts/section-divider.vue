<script setup>
</script>

<template>
  <div class="slidev-layout section-divider-layout">
    <div class="watermark">Monocle AI</div>

    <div class="center-content">
      <slot />
    </div>

    <div class="footer-logo"><i>W</i><span>armblood</span></div>
  </div>
</template>

<style scoped>
.section-divider-layout {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}

.center-content {
  text-align: center;
}
</style>
