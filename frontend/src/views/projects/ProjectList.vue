<template>
  <div class="project-list-page">
    <div class="page-header">
      <h2>项目/比赛招募</h2>
      <el-button type="primary" @click="$router.push('/projects/new')" v-if="auth.isLoggedIn()">发布招募</el-button>
    </div>

    <el-card>
      <el-form :inline="true" :model="query">
        <el-form-item label="类型">
          <el-select v-model="query.type" placeholder="全部" style="width: 120px;">
            <el-option label="全部" value="" />
            <el-option label="课程项目" value="course_project" />
            <el-option label="比赛组队" value="competition" />
            <el-option label="科研项目" value="research" />
            <el-option label="个人项目" value="personal" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="query.status" placeholder="全部" style="width: 110px;">
            <el-option label="全部" value="" />
            <el-option label="招募中" value="recruiting" />
            <el-option label="已满员" value="full" />
            <el-option label="已关闭" value="closed" />
          </el-select>
        </el-form-item>
        <el-form-item label="关键词">
          <el-input v-model="query.keyword" placeholder="标题/技术方向" clearable />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="search">搜索</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card style="margin-top: 16px;">
      <el-table v-loading="loading" :data="projects" stripe style="width: 100%">
        <el-table-column prop="title" label="标题" min-width="180">
          <template #default="{ row }">
            <el-button type="primary" link @click="$router.push(`/projects/${row.id}`)">
              {{ row.title }}
            </el-button>
          </template>
        </el-table-column>
        <el-table-column label="类型" width="100">
          <template #default="{ row }">
            <el-tag :type="typeColor(row.type)">{{ typeLabel(row.type) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="tech_stack" label="技术方向" width="140">
          <template #default="{ row }">
            {{ row.tech_stack || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="描述" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.description?.substring(0, 80) }}{{ row.description?.length > 80 ? '...' : '' }}
          </template>
        </el-table-column>
        <el-table-column label="人数" width="120">
          <template #default="{ row }">
            {{ row.current_members }} / {{ row.required_members }}
          </template>
        </el-table-column>
        <el-table-column label="截止日期" width="120">
          <template #default="{ row }">
            {{ row.deadline ?? '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="nickname" label="发布者" width="100" />
        <el-table-column label="状态" width="90">
          <template #default="{ row }">
            <el-tag :type="row.status === 'recruiting' ? 'success' : 'info'">{{ statusLabel(row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" @click="$router.push(`/projects/${row.id}`)">
              查看详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div style="display: flex; justify-content: center; margin-top: 20px;" v-if="total > 0">
        <el-pagination
          v-model:current-page="query.page"
          v-model:page-size="query.page_size"
          :page-sizes="[2, 5, 10, 20, 50]"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @current-change="fetchProjects"
          @size-change="changePageSize"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { getProjects } from '@/api/projects'
import { useAuthStore } from '@/stores/auth'
import type { Project } from '@/types'

const auth = useAuthStore()
const projects = ref<Project[]>([])
const total = ref(0)
const loading = ref(false)

const query = reactive({
  type: '',
  status: '',
  keyword: '',
  page: 1,
  page_size: 10,
})

function typeLabel(t: string) {
  const map: Record<string, string> = { course_project: '课程项目', competition: '比赛组队', research: '科研项目', personal: '个人项目' }
  return map[t] || t
}

function typeColor(t: string) {
  const map: Record<string, string> = { course_project: 'primary', competition: 'warning', research: 'success', personal: 'info' }
  return map[t] || ''
}

function statusLabel(s: string) {
  const map: Record<string, string> = { recruiting: '招募中', full: '已满员', closed: '已关闭' }
  return map[s] || s
}

async function fetchProjects() {
  loading.value = true
  try {
    const res = await getProjects(query)
    projects.value = res.data.projects
    total.value = res.data.total
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

function search() {
  query.page = 1
  fetchProjects()
}

function changePageSize() {
  query.page = 1
  fetchProjects()
}

onMounted(fetchProjects)
</script>

<style scoped>
.project-list-page {
  max-width: 1200px;
  margin: 0 auto;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
</style>
