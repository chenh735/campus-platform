<template>
  <div class="admin-layout">
    <el-container>
      <el-header class="admin-header">
        <div class="header-left">
          <span class="logo" @click="$router.push('/admin')">管理后台</span>
        </div>
        <div class="header-center">
          <el-menu mode="horizontal" :default-active="activeMenu" @select="handleSelect" :ellipsis="false">
            <el-menu-item index="/admin">首页</el-menu-item>
            <el-menu-item index="/admin/courses">课程管理</el-menu-item>
            <el-menu-item index="/admin/reviews">内容管理</el-menu-item>
            <el-menu-item index="/admin/users">用户管理</el-menu-item>
            <el-menu-item index="/admin/projects">项目管理</el-menu-item>
          </el-menu>
        </div>
        <div class="header-right">
          <span>{{ auth.user?.nickname }}</span>
          <el-button text @click="$router.push('/courses')">返回用户端</el-button>
          <el-button type="danger" text @click="logout">退出</el-button>
        </div>
      </el-header>
      <el-main>
        <slot />
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getMe } from '@/api/auth'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const activeMenu = computed(() => {
  const path = route.path
  if (path === '/admin') return '/admin'
  if (path.startsWith('/admin/courses')) return '/admin/courses'
  if (path.startsWith('/admin/reviews')) return '/admin/reviews'
  if (path.startsWith('/admin/users')) return '/admin/users'
  if (path.startsWith('/admin/projects')) return '/admin/projects'
  return '/admin'
})

function handleSelect(index: string) {
  router.push(index)
}

function logout() {
  auth.clearAuth()
  router.push('/courses')
}

onMounted(async () => {
  if (auth.token && !auth.user) {
    try {
      const res = await getMe()
      auth.user = res.data
    } catch {
      auth.clearAuth()
    }
  }
})
</script>

<style scoped>
.admin-layout {
  min-height: 100vh;
  background: #f5f7fa;
}
.admin-header {
  background: #304156;
  display: flex;
  align-items: center;
  padding: 0 20px;
  height: 60px;
}
.header-left {
  flex-shrink: 0;
}
.logo {
  font-size: 18px;
  font-weight: bold;
  color: #fff;
  cursor: pointer;
  margin-right: 20px;
}
.header-center {
  flex: 1;
  display: flex;
}
.header-center :deep(.el-menu) {
  background: transparent;
  border-bottom: none;
}
.header-center :deep(.el-menu-item) {
  color: #bfcbd9;
}
.header-center :deep(.el-menu-item.is-active) {
  color: #409eff;
  border-bottom-color: #409eff;
}
.header-right {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  color: #bfcbd9;
}
</style>
