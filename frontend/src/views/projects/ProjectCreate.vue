<template>
  <div class="project-create-page">
    <el-card class="form-card">
      <template #header>
        <h3 style="margin: 0;">发布项目/比赛招募</h3>
      </template>
      <el-form :model="form" :rules="rules" ref="formRef" label-width="100px">
        <el-form-item label="标题" prop="title">
          <el-input v-model="form.title" placeholder="请输入招募标题" />
        </el-form-item>
        <el-form-item label="类型" prop="type">
          <el-select v-model="form.type" placeholder="请选择">
            <el-option label="课程项目" value="course_project" />
            <el-option label="比赛组队" value="competition" />
            <el-option label="科研项目" value="research" />
            <el-option label="个人项目" value="personal" />
          </el-select>
        </el-form-item>
        <el-form-item label="技术方向" prop="tech_stack">
          <el-input v-model="form.tech_stack" placeholder="如：Vue, Rust, MySQL" />
        </el-form-item>
        <el-form-item label="需要人数" prop="required_members">
          <el-input-number v-model="form.required_members" :min="1" :max="20" />
        </el-form-item>
        <el-form-item label="截止时间" prop="deadline">
          <el-date-picker v-model="form.deadline" type="date" placeholder="选择日期" value-format="YYYY-MM-DD" />
        </el-form-item>
        <el-form-item label="联系方式" prop="contact">
          <el-input v-model="form.contact" placeholder="微信/QQ/邮箱等" />
        </el-form-item>
        <el-form-item label="项目简介" prop="description">
          <el-input v-model="form.description" type="textarea" :rows="4" placeholder="描述项目目标、内容等" />
        </el-form-item>
        <el-form-item label="队友要求" prop="requirements">
          <el-input v-model="form.requirements" type="textarea" :rows="3" placeholder="对队友技能、经验等方面的要求" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSubmit" :loading="submitting">发布</el-button>
          <el-button @click="$router.back()">取消</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { createProject } from '@/api/projects'

const router = useRouter()
const formRef = ref()
const submitting = ref(false)

const form = reactive({
  title: '',
  type: 'course_project',
  tech_stack: '',
  required_members: 2,
  deadline: '',
  contact: '',
  description: '',
  requirements: '',
})

const rules = {
  title: [{ required: true, message: '请输入标题', trigger: 'blur' }],
  description: [{ required: true, message: '请输入项目简介', trigger: 'blur' }],
  contact: [{ required: true, message: '请输入联系方式', trigger: 'blur' }],
}

async function handleSubmit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    await createProject({ ...form })
    ElMessage.success('项目发布成功')
    router.push('/projects')
  } catch {
    // handled
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.project-create-page {
  max-width: 700px;
  margin: 0 auto;
}
</style>
