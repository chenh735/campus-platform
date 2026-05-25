<template>
  <div class="project-detail-page" v-loading="loading">
    <el-button @click="$router.back()" style="margin-bottom: 16px;">返回列表</el-button>

    <el-card v-if="project">
      <template #header>
        <div class="detail-header">
          <h2>{{ project.title }}</h2>
          <el-tag>{{ typeLabel(project.type) }}</el-tag>
          <el-tag :type="project.status === 'recruiting' ? 'success' : 'info'">{{ statusLabel(project.status) }}</el-tag>
        </div>
      </template>
      <el-descriptions :column="2" border>
        <el-descriptions-item label="发布者">{{ project.nickname }}</el-descriptions-item>
        <el-descriptions-item label="技术方向">{{ project.tech_stack || '-' }}</el-descriptions-item>
        <el-descriptions-item label="人数">已有 {{ project.current_members }} / 需要 {{ project.required_members }}</el-descriptions-item>
        <el-descriptions-item label="截止时间">{{ project.deadline || '未设置' }}</el-descriptions-item>
      </el-descriptions>

      <h4>项目简介</h4>
      <p style="white-space: pre-wrap; color: #606266;">{{ project.description }}</p>

      <h4 v-if="project.requirements">队友要求</h4>
      <p v-if="project.requirements" style="white-space: pre-wrap; color: #606266;">{{ project.requirements }}</p>

      <div style="margin-top: 20px;">
        <el-button type="primary" @click="showApplyDialog = true" v-if="auth.isLoggedIn() && project.status === 'recruiting'" :disabled="project.user_id === auth.user?.id">
          申请加入
        </el-button>
        <el-tag v-if="project.user_id === auth.user?.id" type="warning">这是您发布的项目</el-tag>
      </div>
    </el-card>

    <el-dialog v-model="showApplyDialog" title="申请加入" width="450px">
      <el-form :model="applyForm" label-width="80px">
        <el-form-item label="自我介绍">
          <el-input v-model="applyForm.introduction" type="textarea" :rows="3" />
        </el-form-item>
        <el-form-item label="联系方式">
          <el-input v-model="applyForm.contact" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showApplyDialog = false">取消</el-button>
        <el-button type="primary" @click="handleApply" :loading="applying">提交申请</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { getProject, applyProject } from '@/api/projects'
import { useAuthStore } from '@/stores/auth'
import type { Project } from '@/types'

const route = useRoute()
const auth = useAuthStore()
const project = ref<Project | null>(null)
const loading = ref(false)
const showApplyDialog = ref(false)
const applying = ref(false)
const applyForm = reactive({ introduction: '', contact: '' })

const projectId = computed(() => Number(route.params.id))

function typeLabel(t: string) {
  const map: Record<string, string> = { course_project: '课程项目', competition: '比赛组队', research: '科研项目', personal: '个人项目' }
  return map[t] || t
}

function statusLabel(s: string) {
  const map: Record<string, string> = { recruiting: '招募中', full: '已满员', closed: '已关闭' }
  return map[s] || s
}

async function fetchProject() {
  loading.value = true
  try {
    const res = await getProject(projectId.value)
    project.value = res.data
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

async function handleApply() {
  if (!applyForm.introduction || !applyForm.contact) {
    ElMessage.warning('请填写完整信息')
    return
  }
  applying.value = true
  try {
    await applyProject(projectId.value, { ...applyForm })
    ElMessage.success('申请已提交')
    showApplyDialog.value = false
  } catch {
    // handled
  } finally {
    applying.value = false
  }
}

onMounted(fetchProject)
</script>

<style scoped>
.project-detail-page {
  max-width: 900px;
  margin: 0 auto;
}
.detail-header {
  display: flex;
  align-items: center;
  gap: 12px;
}
.detail-header h2 {
  margin: 0;
  flex: 1;
}
h4 {
  margin: 16px 0 8px;
}
</style>
