<template>
  <div class="admin-users">
    <h2>用户管理</h2>
    <el-card>
      <el-form :inline="true" :model="query">
        <el-form-item>
          <el-input v-model="query.keyword" placeholder="搜索邮箱/昵称" clearable />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="searchUsers">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="users" v-loading="loading">
        <el-table-column prop="nickname" label="昵称" width="120" />
        <el-table-column prop="email" label="邮箱" width="250" />
        <el-table-column label="等级" width="80">
          <template #default="{ row }">Lv{{ getLevel(row.experience) }}</template>
        </el-table-column>
        <el-table-column prop="experience" label="经验值" width="80" />
        <el-table-column prop="status" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.status === 'active' ? 'success' : 'danger'">
              {{ row.status === 'active' ? '正常' : '禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button v-if="row.role !== 'admin'" size="small" :type="row.status === 'active' ? 'danger' : 'success'" @click="toggleStatus(row)">
              {{ row.status === 'active' ? '禁用' : '恢复' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-wrap" v-if="total > 0">
        <el-pagination
          v-model:current-page="query.page"
          v-model:page-size="query.page_size"
          :page-sizes="[2, 5, 10, 20, 50]"
          :total="total"
          layout="total, sizes, prev, pager, next"
          @current-change="fetchUsers"
          @size-change="changePageSize"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getUsers, updateUserStatus } from '@/api/admin'
import type { UserInfo } from '@/types'

const users = ref<UserInfo[]>([])
const total = ref(0)
const loading = ref(false)
const query = reactive({ keyword: '', page: 1, page_size: 10 })

function getLevel(exp: number) {
  if (exp < 10) return 1
  if (exp < 30) return 2
  if (exp < 60) return 3
  if (exp < 100) return 4
  return 5
}

async function fetchUsers() {
  loading.value = true
  try {
    const res = await getUsers(query)
    users.value = res.data.users
    total.value = res.data.total
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

function searchUsers() {
  query.page = 1
  fetchUsers()
}

function changePageSize() {
  query.page = 1
  fetchUsers()
}

async function toggleStatus(row: UserInfo) {
  const newStatus = row.status === 'active' ? 'disabled' : 'active'
  try {
    await updateUserStatus(row.id, newStatus)
    ElMessage.success(`用户已${newStatus === 'active' ? '恢复' : '禁用'}`)
    fetchUsers()
  } catch {
    // handled
  }
}

onMounted(fetchUsers)
</script>

<style scoped>
.admin-users {
  max-width: 1200px;
}
.pagination-wrap {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}
</style>
