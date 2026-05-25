<template>
  <div class="course-list-page">
    <el-card>
      <el-form :inline="true" :model="query" class="search-form">
        <el-form-item label="搜索">
          <el-input v-model="query.keyword" placeholder="课程名称/代码/教师" clearable />
        </el-form-item>
        <el-form-item label="类别">
          <el-select v-model="query.category" placeholder="全部" style="width: 110px;">
            <el-option label="全部" value="" />
            <el-option label="专业必修" value="专业必修" />
            <el-option label="专业选修" value="专业选修" />
            <el-option label="公共必修" value="公共必修" />
            <el-option label="公共选修" value="公共选修" />
          </el-select>
        </el-form-item>
        <el-form-item label="排序">
          <el-select v-model="query.sort" placeholder="默认" style="width: 110px;">
            <el-option label="默认" value="" />
            <el-option label="评分最高" value="rating_desc" />
            <el-option label="评分最低" value="rating_asc" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="search">搜索</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card style="margin-top: 16px;">
      <el-table v-loading="loading" :data="courses" stripe style="width: 100%">
        <el-table-column prop="code" label="课程代码" width="110" />
        <el-table-column prop="name" label="课程名称" min-width="160" />
        <el-table-column prop="teacher" label="教师" width="120" />
        <el-table-column prop="credit" label="学分" width="70">
          <template #default="{ row }">
            {{ row.credit ?? '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="category" label="类别" width="100">
          <template #default="{ row }">
            <el-tag size="small">{{ row.category }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="评分" width="100">
          <template #default="{ row }">
            <span v-if="row.rating_avg">{{ row.rating_avg.toFixed(1) }} / 5</span>
            <span v-else style="color: #909399;">暂无</span>
          </template>
        </el-table-column>
        <el-table-column prop="review_count" label="评价" width="70" />
        <el-table-column prop="material_count" label="资料" width="70" />
        <el-table-column label="操作" width="100" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" link @click="$router.push(`/courses/${row.id}`)">
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
          @current-change="fetchCourses"
          @size-change="changePageSize"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { getCourses } from '@/api/courses'
import type { Course } from '@/types'

const courses = ref<Course[]>([])
const total = ref(0)
const loading = ref(false)

const query = reactive({
  keyword: '',
  category: '',
  sort: '',
  page: 1,
  page_size: 10,
})

async function fetchCourses() {
  loading.value = true
  try {
    const res = await getCourses(query)
    courses.value = res.data.courses
    total.value = res.data.total
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

function search() {
  query.page = 1
  fetchCourses()
}

function changePageSize() {
  query.page = 1
  fetchCourses()
}

onMounted(fetchCourses)
</script>

<style scoped>
.course-list-page {
  max-width: 1200px;
  margin: 0 auto;
}
</style>
