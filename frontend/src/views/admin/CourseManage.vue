<template>
  <div class="admin-courses">
    <div class="page-header">
      <h2>课程管理</h2>
      <el-button type="primary" @click="showCreateDialog = true">新增课程</el-button>
    </div>

    <el-card>
      <el-table :data="courses" v-loading="loading">
        <el-table-column prop="code" label="课程代码" width="120" />
        <el-table-column prop="name" label="课程名称" width="150" />
        <el-table-column prop="teacher" label="教师" width="120" />
        <el-table-column prop="credit" label="学分" width="80" />
        <el-table-column prop="category" label="类别" width="100" />
        <el-table-column prop="status" label="状态" width="80" />
        <el-table-column label="操作" width="150">
          <template #default="{ row }">
            <el-button size="small" @click="editCourse(row)">编辑</el-button>
            <el-button size="small" type="danger" @click="handleDelete(row.id)">下架</el-button>
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
          @current-change="fetchCourses"
          @size-change="changePageSize"
        />
      </div>
    </el-card>

    <el-dialog v-model="showCreateDialog" :title="editingCourse ? '编辑课程' : '新增课程'" width="500px">
      <el-form :model="form" label-width="80px">
        <el-form-item label="课程代码">
          <el-input v-model="form.code" />
        </el-form-item>
        <el-form-item label="课程名称">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="任课教师">
          <el-input v-model="form.teacher" />
        </el-form-item>
        <el-form-item label="学分">
          <el-input-number v-model="form.credit" :min="0" :max="10" :step="0.5" />
        </el-form-item>
        <el-form-item label="类别">
          <el-select v-model="form.category">
            <el-option label="专业必修" value="专业必修" />
            <el-option label="专业选修" value="专业选修" />
            <el-option label="公共必修" value="公共必修" />
            <el-option label="公共选修" value="公共选修" />
          </el-select>
        </el-form-item>
        <el-form-item label="开课学期">
          <el-input v-model="form.semester" />
        </el-form-item>
        <el-form-item label="课程简介">
          <el-input v-model="form.description" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="handleSave" :loading="saving">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { getCourses } from '@/api/courses'
import { createCourse, updateCourse, deleteCourse } from '@/api/admin'
import type { Course } from '@/types'

const courses = ref<Course[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(10)
const loading = ref(false)
const showCreateDialog = ref(false)
const editingCourse = ref<Course | null>(null)
const saving = ref(false)
const form = reactive({
  code: '', name: '', teacher: '', credit: 0, category: '', semester: '', description: '',
})

async function fetchCourses() {
  loading.value = true
  try {
    const res = await getCourses({ page: page.value, page_size: pageSize.value })
    courses.value = res.data.courses
    total.value = res.data.total
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

function changePageSize() {
  page.value = 1
  fetchCourses()
}

function editCourse(row: Course) {
  editingCourse.value = row
  Object.assign(form, {
    code: row.code, name: row.name, teacher: row.teacher, credit: row.credit ?? 0,
    category: row.category, semester: '', description: row.description,
  })
  showCreateDialog.value = true
}

function resetForm() {
  editingCourse.value = null
  Object.assign(form, { code: '', name: '', teacher: '', credit: 0, category: '', semester: '', description: '' })
}

async function handleSave() {
  saving.value = true
  try {
    if (editingCourse.value) {
      await updateCourse(editingCourse.value.id, { ...form })
      ElMessage.success('课程已更新')
    } else {
      await createCourse({ ...form })
      ElMessage.success('课程已添加')
    }
    showCreateDialog.value = false
    resetForm()
    fetchCourses()
  } catch {
    // handled
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: number) {
  try {
    await ElMessageBox.confirm('确定要下架该课程吗？', '确认', { type: 'warning' })
    await deleteCourse(id)
    ElMessage.success('课程已下架')
    fetchCourses()
  } catch {
    // cancelled
  }
}

onMounted(fetchCourses)
</script>

<style scoped>
.admin-courses {
  max-width: 1200px;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
.pagination-wrap {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}
</style>
