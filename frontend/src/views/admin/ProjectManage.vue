<template>
  <div class="admin-projects">
    <h2>项目管理</h2>
    <el-card>
      <el-form :inline="true" :model="query" class="filter-form">
        <el-form-item label="类型">
          <el-select v-model="query.type" style="width: 120px;">
            <el-option label="全部" value="" />
            <el-option label="课程项目" value="course_project" />
            <el-option label="比赛组队" value="competition" />
            <el-option label="科研项目" value="research" />
            <el-option label="个人项目" value="personal" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="query.status" style="width: 110px;">
            <el-option label="全部" value="" />
            <el-option label="招募中" value="recruiting" />
            <el-option label="已满员" value="full" />
            <el-option label="已关闭" value="closed" />
            <el-option label="已隐藏" value="hidden" />
          </el-select>
        </el-form-item>
        <el-form-item label="搜索">
          <el-input v-model="query.keyword" placeholder="标题/技术方向/发布者" clearable />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="search">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="projects" v-loading="loading">
        <el-table-column prop="title" label="标题" />
        <el-table-column label="类型" width="100">
          <template #default="{ row }">{{ typeLabel(row.type) }}</template>
        </el-table-column>
        <el-table-column prop="nickname" label="发布者" width="120" />
        <el-table-column label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'recruiting' ? 'success' : 'info'">{{ statusLabel(row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200">
          <template #default="{ row }">
            <el-button size="small" type="warning" @click="handleStatus(row.id, 'hidden')">隐藏</el-button>
            <el-button size="small" type="danger" @click="handleStatus(row.id, 'closed')">关闭</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="pagination-wrap" v-if="total > 0">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="pageSize"
          :page-sizes="[2, 5, 10, 20, 50]"
          :total="total"
          layout="total, sizes, prev, pager, next"
          @current-change="fetchProjects"
          @size-change="changePageSize"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getAdminProjects, updateProjectStatus } from '@/api/admin'
import type { Project } from '@/types'

const projects = ref<Project[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(10)
const loading = ref(false)
const query = reactive({ keyword: '', type: '', status: '' })

function typeLabel(t: string) {
  const map: Record<string, string> = { course_project: '课程项目', competition: '比赛组队', research: '科研项目', personal: '个人项目' }
  return map[t] || t
}

function statusLabel(s: string) {
  const map: Record<string, string> = { recruiting: '招募中', full: '已满员', closed: '已关闭', hidden: '已隐藏' }
  return map[s] || s
}

async function fetchProjects() {
  loading.value = true
  try {
    const res = await getAdminProjects({ ...query, page: page.value, page_size: pageSize.value })
    projects.value = res.data.projects
    total.value = res.data.total
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

function search() {
  page.value = 1
  fetchProjects()
}

function changePageSize() {
  page.value = 1
  fetchProjects()
}

async function handleStatus(id: number, status: string) {
  try {
    await updateProjectStatus(id, status)
    ElMessage.success('项目状态已更新')
    fetchProjects()
  } catch {
    // handled
  }
}

onMounted(fetchProjects)
</script>

<style scoped>
.admin-projects {
  max-width: 1200px;
}
.filter-form {
  margin-bottom: 8px;
}
.pagination-wrap {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}
</style>
