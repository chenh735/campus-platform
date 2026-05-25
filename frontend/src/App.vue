<template>
  <DefaultLayout v-if="useDefaultLayout">
    <router-view />
  </DefaultLayout>
  <AdminLayout v-else-if="useAdminLayout">
    <router-view />
  </AdminLayout>
  <router-view v-else />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import DefaultLayout from '@/layouts/DefaultLayout.vue'
import AdminLayout from '@/layouts/AdminLayout.vue'

const route = useRoute()

const useDefaultLayout = computed(() => {
  const path = route.path
  return !path.startsWith('/login') && !path.startsWith('/register') && !path.startsWith('/admin')
})

const useAdminLayout = computed(() => {
  return route.path.startsWith('/admin')
})
</script>

<style>
body {
  margin: 0;
  padding: 0;
  font-family: 'Helvetica Neue', Helvetica, 'PingFang SC', 'Hiragino Sans GB',
    'Microsoft YaHei', Arial, sans-serif;
}
</style>
