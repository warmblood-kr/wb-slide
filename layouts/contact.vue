<script setup>
</script>

<template>
  <div class="slidev-layout contact-layout">
    <div class="watermark">Monocle AI</div>

    <div class="contact-content">
      <slot />
    </div>

    <div class="footer-logo"><i>W</i><span>armblood</span></div>
  </div>
</template>

<style scoped>
.contact-layout {
  padding: 40px;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
}

.contact-content {
  margin-top: 1rem;
}
</style>
