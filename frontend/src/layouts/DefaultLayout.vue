<template>
  <div class="layout">
    <el-container>
      <el-header class="header">
        <div class="header-left">
          <span class="logo" @click="$router.push('/')">软件工程学院学习协作平台</span>
        </div>
        <div class="header-center">
          <el-menu mode="horizontal" :default-active="activeMenu" @select="handleSelect" :ellipsis="false">
            <el-menu-item index="/courses">课程</el-menu-item>
            <el-menu-item index="/projects">项目招募</el-menu-item>
            <el-menu-item v-if="auth.isLoggedIn()" index="/profile">个人中心</el-menu-item>
            <el-menu-item v-if="auth.isAdmin()" index="/admin">管理后台</el-menu-item>
          </el-menu>
        </div>
        <div class="header-right">
          <template v-if="auth.isLoggedIn()">
            <span class="user-info">{{ auth.user?.nickname }} Lv{{ level }}</span>
            <el-button type="danger" text @click="logout">退出</el-button>
          </template>
          <template v-else>
            <el-button type="primary" @click="$router.push('/login')">登录</el-button>
            <el-button @click="$router.push('/register')">注册</el-button>
          </template>
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
  if (path.startsWith('/courses')) return '/courses'
  if (path.startsWith('/projects')) return '/projects'
  if (path.startsWith('/profile')) return '/profile'
  if (path.startsWith('/admin')) return '/admin'
  return '/courses'
})

const level = computed(() => {
  const exp = auth.user?.experience || 0
  if (exp < 10) return 1
  if (exp < 30) return 2
  if (exp < 60) return 3
  if (exp < 100) return 4
  return 5
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
.layout {
  min-height: 100vh;
  background: #f5f7fa;
}
.header {
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
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
  color: #409eff;
  cursor: pointer;
  margin-right: 20px;
}
.header-center {
  flex: 1;
  display: flex;
  justify-content: center;
}
.header-center :deep(.el-menu) {
  border-bottom: none;
}
.header-right {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}
.user-info {
  color: #606266;
}
</style>
