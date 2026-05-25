<template>
  <div class="course-detail-page" v-loading="loading">
    <el-button @click="$router.back()" style="margin-bottom: 16px;">返回课程列表</el-button>

    <el-row :gutter="20" v-if="course">
      <el-col :span="8">
        <el-card>
          <template #header>
            <h3 style="margin: 0;">课程信息</h3>
          </template>
          <div class="info-item"><strong>课程名称：</strong>{{ course.name }}</div>
          <div class="info-item"><strong>课程代码：</strong>{{ course.code }}</div>
          <div class="info-item"><strong>任课教师：</strong>{{ course.teacher }}</div>
          <div class="info-item"><strong>学分：</strong>{{ course.credit ?? '-' }}</div>
          <div class="info-item"><strong>类别：</strong>{{ course.category }}</div>
          <div class="info-item"><strong>开课学期：</strong>{{ course.semester }}</div>
          <div class="info-item"><strong>课程简介：</strong>{{ course.description || '暂无' }}</div>
          <div class="info-item" v-if="ratingAvg !== null"><strong>平均评分：</strong>{{ ratingAvg }} / 5</div>
        </el-card>
      </el-col>
      <el-col :span="16">
        <el-tabs v-model="activeTab">
          <el-tab-pane label="评价" name="reviews">
            <div class="section-header">
              <span v-if="ratingAvg !== null">当前评分：{{ ratingAvg }} / 5</span>
              <el-button type="primary" size="small" @click="showReviewDialog = true" v-if="auth.isLoggedIn()">我要评价</el-button>
            </div>
            <el-empty v-if="reviews.length === 0" description="暂无评价" />
            <div v-for="review in reviews" :key="review.id" class="review-item">
              <div class="review-header">
                <span class="reviewer">{{ review.nickname }}</span>
                <el-rate :model-value="review.rating" disabled show-score size="small" />
                <span class="like-count">赞 {{ review.like_count }}</span>
              </div>
              <div class="review-meta" v-if="review.difficulty || review.workload">
                难度：{{ review.difficulty || '-' }} | 工作量：{{ review.workload || '-' }}
              </div>
              <div class="review-content">{{ review.content }}</div>
              <el-button size="small" @click="handleLikeReview(review)" :type="review.liked ? 'primary' : 'default'" :disabled="!auth.isLoggedIn()">
                {{ review.liked ? '取消点赞' : '点赞' }} ({{ review.like_count }})
              </el-button>
            </div>
            <div class="pagination-wrap" v-if="reviewsTotal > 0">
              <el-pagination
                v-model:current-page="reviewQuery.page"
                v-model:page-size="reviewQuery.page_size"
                :page-sizes="[2, 5, 10, 20, 50]"
                :total="reviewsTotal"
                layout="total, sizes, prev, pager, next"
                @current-change="fetchReviews"
                @size-change="changeReviewPageSize"
              />
            </div>
          </el-tab-pane>
          <el-tab-pane label="学习资料" name="materials">
            <div class="section-header">
              <el-button type="primary" size="small" @click="showUploadDialog = true" v-if="auth.isLoggedIn()">上传资料</el-button>
            </div>
            <el-empty v-if="materials.length === 0" description="暂无资料" />
            <div v-for="mat in materials" :key="mat.id" class="material-item">
              <div class="material-info">
                <span class="material-title">{{ mat.title }}</span>
                <span class="material-meta">上传者：{{ mat.nickname }} | {{ mat.resource_type === 'link' ? '链接资料' : `下载：${mat.download_count}` }} | 赞：{{ mat.like_count }}</span>
                <el-tag size="small" v-if="mat.tag">{{ mat.tag }}</el-tag>
              </div>
              <div class="material-actions">
                <el-button v-if="mat.resource_type === 'link'" size="small" @click="openLink(mat.link_url)">打开链接</el-button>
                <el-button v-else size="small" @click="handleDownload(mat.id)" :disabled="!auth.isLoggedIn()">下载</el-button>
                <el-button size="small" @click="handleLikeMaterial(mat)" :type="mat.liked ? 'primary' : 'default'" :disabled="!auth.isLoggedIn()">
                  {{ mat.liked ? '取消点赞' : '点赞' }} ({{ mat.like_count }})
                </el-button>
              </div>
            </div>
            <div class="pagination-wrap" v-if="materialsTotal > 0">
              <el-pagination
                v-model:current-page="materialQuery.page"
                v-model:page-size="materialQuery.page_size"
                :page-sizes="[2, 5, 10, 20, 50]"
                :total="materialsTotal"
                layout="total, sizes, prev, pager, next"
                @current-change="fetchMaterials"
                @size-change="changeMaterialPageSize"
              />
            </div>
          </el-tab-pane>
        </el-tabs>
      </el-col>
    </el-row>

    <!-- Review dialog -->
    <el-dialog v-model="showReviewDialog" title="发表课程评价" width="500px">
      <el-form :model="reviewForm" label-width="80px">
        <el-form-item label="评分">
          <el-rate v-model="reviewForm.rating" />
        </el-form-item>
        <el-form-item label="难度">
          <el-select v-model="reviewForm.difficulty" placeholder="请选择">
            <el-option label="低" value="low" />
            <el-option label="中" value="medium" />
            <el-option label="高" value="high" />
          </el-select>
        </el-form-item>
        <el-form-item label="工作量">
          <el-select v-model="reviewForm.workload" placeholder="请选择">
            <el-option label="低" value="low" />
            <el-option label="中" value="medium" />
            <el-option label="高" value="high" />
          </el-select>
        </el-form-item>
        <el-form-item label="学习建议">
          <el-input v-model="reviewForm.content" type="textarea" :rows="4" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showReviewDialog = false">取消</el-button>
        <el-button type="primary" @click="submitReview" :loading="submitting">提交</el-button>
      </template>
    </el-dialog>

    <!-- Upload dialog -->
    <el-dialog v-model="showUploadDialog" title="上传学习资料" width="500px">
      <el-form :model="uploadForm" label-width="80px">
        <el-form-item label="标题">
          <el-input v-model="uploadForm.title" />
        </el-form-item>
        <el-form-item label="标签">
          <el-select v-model="uploadForm.tag" placeholder="请选择">
            <el-option label="课件" value="课件" />
            <el-option label="笔记" value="笔记" />
            <el-option label="作业" value="作业" />
            <el-option label="复习资料" value="复习资料" />
            <el-option label="往年题" value="往年题" />
          </el-select>
        </el-form-item>
        <el-form-item label="简介">
          <el-input v-model="uploadForm.description" type="textarea" :rows="2" />
        </el-form-item>
        <el-form-item label="资料类型">
          <el-radio-group v-model="uploadForm.resource_type">
            <el-radio-button value="file">上传文件</el-radio-button>
            <el-radio-button value="link">分享链接</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="uploadForm.resource_type === 'file'" label="文件">
          <input type="file" @change="onFileChange" ref="fileInput" />
        </el-form-item>
        <el-form-item v-else label="链接">
          <el-input v-model="uploadForm.link_url" placeholder="https://..." />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showUploadDialog = false">取消</el-button>
        <el-button type="primary" @click="submitUpload" :loading="uploading">立即上传</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { getCourse } from '@/api/courses'
import { getReviews, getCourseLikeStatus, createReview, likeReview, unlikeReview } from '@/api/reviews'
import { getMaterials, uploadMaterial, downloadMaterial, likeMaterial, unlikeMaterial } from '@/api/materials'
import { useAuthStore } from '@/stores/auth'
import type { CourseDetail, Review, Material } from '@/types'

const route = useRoute()
const auth = useAuthStore()
const course = ref<CourseDetail | null>(null)
type LikedReview = Review & { liked: boolean }
type LikedMaterial = Material & { liked: boolean }

const reviews = ref<LikedReview[]>([])
const materials = ref<LikedMaterial[]>([])
const likedReviewIds = ref(new Set<number>())
const likedMaterialIds = ref(new Set<number>())
const reviewsTotal = ref(0)
const materialsTotal = ref(0)
const loading = ref(false)
const activeTab = ref('reviews')
const showReviewDialog = ref(false)
const showUploadDialog = ref(false)
const submitting = ref(false)
const uploading = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)

const reviewForm = reactive({ rating: 5, difficulty: '', workload: '', content: '' })
const uploadForm = reactive({ title: '', description: '', tag: '', resource_type: 'file', link_url: '' })
const reviewQuery = reactive({ page: 1, page_size: 10 })
const materialQuery = reactive({ page: 1, page_size: 10 })

const courseId = computed(() => Number(route.params.id))
const ratingAvg = computed(() => {
  return course.value?.rating_avg == null ? null : course.value.rating_avg.toFixed(1)
})

async function fetchData() {
  loading.value = true
  try {
    await Promise.all([fetchCourse(), fetchLikeStatus(), fetchReviews(), fetchMaterials()])
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

async function fetchCourse() {
  const res = await getCourse(courseId.value)
  course.value = res.data
}

async function fetchReviews() {
  const res = await getReviews(courseId.value, reviewQuery)
  reviews.value = res.data.items.map(r => ({ ...r, liked: likedReviewIds.value.has(r.id) }))
  reviewsTotal.value = res.data.total
}

async function fetchMaterials() {
  const res = await getMaterials(courseId.value, materialQuery)
  materials.value = res.data.items.map(mat => ({ ...mat, liked: likedMaterialIds.value.has(mat.id) }))
  materialsTotal.value = res.data.total
}

async function fetchLikeStatus() {
  if (!auth.isLoggedIn()) {
    likedReviewIds.value = new Set()
    likedMaterialIds.value = new Set()
    return
  }
  const res = await getCourseLikeStatus(courseId.value)
  likedReviewIds.value = new Set(res.data.review_ids)
  likedMaterialIds.value = new Set(res.data.material_ids)
  reviews.value = reviews.value.map(review => ({ ...review, liked: likedReviewIds.value.has(review.id) }))
  materials.value = materials.value.map(mat => ({ ...mat, liked: likedMaterialIds.value.has(mat.id) }))
}

function changeReviewPageSize() {
  reviewQuery.page = 1
  fetchReviews()
}

function changeMaterialPageSize() {
  materialQuery.page = 1
  fetchMaterials()
}

async function submitReview() {
  if (!reviewForm.content.trim()) {
    ElMessage.warning('请输入评价内容')
    return
  }
  submitting.value = true
  try {
    await createReview(courseId.value, { ...reviewForm })
    ElMessage.success('评价发表成功')
    showReviewDialog.value = false
    reviewForm.content = ''
    reviewQuery.page = 1
    fetchReviews()
  } catch {
    // handled
  } finally {
    submitting.value = false
  }
}

function onFileChange(e: Event) {
  const target = e.target as HTMLInputElement
  if (target.files) selectedFile.value = target.files[0]
}

async function submitUpload() {
  if (!uploadForm.title.trim()) {
    ElMessage.warning('请填写标题')
    return
  }
  if (uploadForm.resource_type === 'file' && !selectedFile.value) {
    ElMessage.warning('请选择文件')
    return
  }
  if (uploadForm.resource_type === 'link' && !/^https?:\/\/\S+$/i.test(uploadForm.link_url.trim())) {
    ElMessage.warning('请输入有效的 http 或 https 链接')
    return
  }
  uploading.value = true
  try {
    const fd = new FormData()
    fd.append('title', uploadForm.title)
    fd.append('description', uploadForm.description)
    fd.append('tag', uploadForm.tag)
    fd.append('resource_type', uploadForm.resource_type)
    if (uploadForm.resource_type === 'file' && selectedFile.value) {
      fd.append('file', selectedFile.value)
    } else {
      fd.append('link_url', uploadForm.link_url.trim())
    }
    await uploadMaterial(courseId.value, fd)
    ElMessage.success('资料上传成功')
    showUploadDialog.value = false
    uploadForm.title = ''
    uploadForm.description = ''
    uploadForm.tag = ''
    uploadForm.resource_type = 'file'
    uploadForm.link_url = ''
    selectedFile.value = null
    if (fileInput.value) fileInput.value.value = ''
    materialQuery.page = 1
    fetchMaterials()
  } catch {
    // handled
  } finally {
    uploading.value = false
  }
}

function openLink(link: string | null) {
  if (link) window.open(link, '_blank', 'noopener,noreferrer')
}

async function handleDownload(id: number) {
  try {
    const res = await downloadMaterial(id)
    const url = URL.createObjectURL(res.data)
    const a = document.createElement('a')
    a.href = url
    a.download = ''
    a.click()
    URL.revokeObjectURL(url)
  } catch {
    // handled
  }
}

async function handleLikeReview(review: LikedReview) {
  try {
    if (review.liked) {
      await unlikeReview(review.id)
      review.like_count = Math.max(review.like_count - 1, 0)
      review.liked = false
      likedReviewIds.value.delete(review.id)
      ElMessage.success('已取消点赞')
    } else {
      await likeReview(review.id)
      review.like_count++
      review.liked = true
      likedReviewIds.value.add(review.id)
      ElMessage.success('点赞成功')
    }
  } catch {
    // handled
  }
}

async function handleLikeMaterial(mat: LikedMaterial) {
  try {
    if (mat.liked) {
      await unlikeMaterial(mat.id)
      mat.like_count = Math.max(mat.like_count - 1, 0)
      mat.liked = false
      likedMaterialIds.value.delete(mat.id)
      ElMessage.success('已取消点赞')
    } else {
      await likeMaterial(mat.id)
      mat.like_count++
      mat.liked = true
      likedMaterialIds.value.add(mat.id)
      ElMessage.success('点赞成功')
    }
  } catch {
    // handled
  }
}

onMounted(fetchData)
</script>

<style scoped>
.course-detail-page {
  max-width: 1200px;
  margin: 0 auto;
}
.info-item {
  margin-bottom: 8px;
  font-size: 14px;
  color: #303133;
}
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
.review-item {
  border-bottom: 1px solid #ebeef5;
  padding: 12px 0;
}
.review-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}
.reviewer {
  font-weight: bold;
  color: #303133;
}
.like-count {
  color: #909399;
  font-size: 12px;
}
.review-meta {
  font-size: 12px;
  color: #909399;
  margin-bottom: 4px;
}
.review-content {
  margin: 8px 0;
  color: #606266;
  line-height: 1.6;
}
.material-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #ebeef5;
}
.material-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.material-title {
  font-weight: bold;
}
.material-meta {
  font-size: 12px;
  color: #909399;
}
.material-actions {
  display: flex;
  gap: 8px;
}
.pagination-wrap {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}
</style>
