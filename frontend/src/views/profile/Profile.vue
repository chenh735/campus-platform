<template>
  <div class="profile-page" v-loading="userLoading">
    <el-card class="user-card" v-if="profile">
      <div class="user-header">
        <div>
          <h2>{{ profile.user.nickname }}</h2>
          <p>邮箱：{{ profile.user.email }}</p>
          <p>等级：Lv{{ profile.level }} | 经验值：{{ profile.user.experience }}</p>
          <el-progress :percentage="expProgress" :stroke-width="12" style="width: 200px;" />
        </div>
        <div>
          <el-button @click="showNicknameDialog = true">修改昵称</el-button>
          <el-button @click="showPasswordDialog = true">修改密码</el-button>
        </div>
      </div>
    </el-card>

    <el-tabs v-model="activeTab" style="margin-top: 16px;" @tab-change="onTabChange">
      <el-tab-pane label="我的资料" name="materials">
        <el-empty v-if="!materialsLoading && materials.length === 0" description="暂未上传资料" />
        <template v-else>
          <el-table :data="materials" stripe v-loading="materialsLoading">
            <el-table-column label="课程编号" width="100">
              <template #default="{ row }">{{ row.course_code ?? '-' }}</template>
            </el-table-column>
            <el-table-column label="课程名称" min-width="140">
              <template #default="{ row }">{{ row.course_name ?? '-' }}</template>
            </el-table-column>
            <el-table-column prop="title" label="标题" min-width="140" />
            <el-table-column prop="tag" label="标签" width="80">
              <template #default="{ row }">{{ row.tag || '-' }}</template>
            </el-table-column>
            <el-table-column label="类型" width="80">
              <template #default="{ row }">{{ row.resource_type === 'link' ? '链接' : '文件' }}</template>
            </el-table-column>
            <el-table-column prop="like_count" label="点赞" width="70" />
            <el-table-column prop="download_count" label="下载" width="70" />
            <el-table-column label="状态" width="90">
              <template #default="{ row }">
                <el-tag :type="row.status === 'approved' ? 'success' : row.status === 'pending' ? 'warning' : 'info'">
                  {{ statusLabel(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="120" fixed="right">
              <template #default="{ row }">
                <el-button v-if="row.resource_type === 'link'" type="primary" size="small" link @click="openMaterialLink(row.link_url)">打开</el-button>
                <el-popconfirm title="确定删除这份资料吗？" @confirm="handleDeleteMaterial(row.id)">
                  <template #reference>
                    <el-button type="danger" size="small" link>删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination-wrap" v-if="materialsTotal > 0">
            <el-pagination v-model:current-page="materialsPage" v-model:page-size="materialsPageSize" :page-sizes="[2, 5, 10, 20, 50]" :total="materialsTotal" layout="total, sizes, prev, pager, next" @current-change="fetchMaterials" @size-change="changeMaterialsPageSize" />
          </div>
        </template>
      </el-tab-pane>

      <el-tab-pane label="我的评价" name="reviews">
        <el-empty v-if="!reviewsLoading && reviews.length === 0" description="暂未发表评价" />
        <template v-else>
          <el-table :data="reviews" stripe v-loading="reviewsLoading">
            <el-table-column label="课程编号" width="100">
              <template #default="{ row }">{{ row.course_code ?? '-' }}</template>
            </el-table-column>
            <el-table-column label="课程名称" min-width="140">
              <template #default="{ row }">{{ row.course_name ?? '-' }}</template>
            </el-table-column>
            <el-table-column label="评分" width="70">
              <template #default="{ row }">{{ row.rating }} / 5</template>
            </el-table-column>
            <el-table-column label="内容" min-width="180" show-overflow-tooltip>
              <template #default="{ row }">{{ row.content }}</template>
            </el-table-column>
            <el-table-column prop="like_count" label="点赞" width="70" />
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="row.status === 'visible' ? 'success' : 'info'">{{ statusLabel(row.status) }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="80" fixed="right">
              <template #default="{ row }">
                <el-popconfirm title="确定删除这条评价吗？" @confirm="handleDeleteReview(row.id)">
                  <template #reference>
                    <el-button type="danger" size="small" link>删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination-wrap" v-if="reviewsTotal > 0">
            <el-pagination v-model:current-page="reviewsPage" v-model:page-size="reviewsPageSize" :page-sizes="[2, 5, 10, 20, 50]" :total="reviewsTotal" layout="total, sizes, prev, pager, next" @current-change="fetchReviews" @size-change="changeReviewsPageSize" />
          </div>
        </template>
      </el-tab-pane>

      <el-tab-pane label="我的招募" name="projects">
        <el-empty v-if="!projectsLoading && myProjects.length === 0" description="暂未发布招募" />
        <template v-else>
          <el-table :data="myProjects" stripe v-loading="projectsLoading">
            <el-table-column prop="title" label="标题" min-width="160" />
            <el-table-column label="类型" width="100">
              <template #default="{ row }">
                <el-tag :type="typeColor(row.type)">{{ typeLabel(row.type) }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="人数" width="100">
              <template #default="{ row }">{{ row.current_members }} / {{ row.required_members }}</template>
            </el-table-column>
            <el-table-column label="截止日期" width="110">
              <template #default="{ row }">{{ row.deadline ?? '-' }}</template>
            </el-table-column>
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="row.status === 'recruiting' ? 'success' : 'info'">{{ statusLabel(row.status) }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="130" fixed="right">
              <template #default="{ row }">
                <el-popconfirm v-if="row.status !== 'hidden'" title="确定隐藏这条招募吗？" @confirm="handleProjectStatus(row.id, 'hidden')">
                  <template #reference>
                    <el-button type="warning" size="small" link>隐藏</el-button>
                  </template>
                </el-popconfirm>
                <el-popconfirm v-if="row.status !== 'closed'" title="确定关闭这条招募吗？" @confirm="handleProjectStatus(row.id, 'closed')">
                  <template #reference>
                    <el-button type="danger" size="small" link>关闭</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination-wrap" v-if="projectsTotal > 0">
            <el-pagination v-model:current-page="projectsPage" v-model:page-size="projectsPageSize" :page-sizes="[2, 5, 10, 20, 50]" :total="projectsTotal" layout="total, sizes, prev, pager, next" @current-change="fetchMyProjects" @size-change="changeProjectsPageSize" />
          </div>
        </template>
      </el-tab-pane>

      <el-tab-pane label="我的申请" name="applications">
        <el-empty v-if="!appsLoading && applications.length === 0" description="暂未申请项目" />
        <template v-else>
          <el-table :data="applications" stripe v-loading="appsLoading">
            <el-table-column prop="project_title" label="项目标题" min-width="160" />
            <el-table-column label="申请说明" min-width="160" show-overflow-tooltip>
              <template #default="{ row }">{{ row.introduction || '-' }}</template>
            </el-table-column>
            <el-table-column prop="contact" label="联系方式" width="140" />
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="row.status === 'accepted' ? 'success' : row.status === 'pending' ? 'warning' : 'info'">
                  {{ appStatusLabel(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="80" fixed="right">
              <template #default="{ row }">
                <el-popconfirm v-if="row.status !== 'accepted'" title="确定删除这条申请吗？" @confirm="handleDeleteApplication(row.id)">
                  <template #reference>
                    <el-button type="danger" size="small" link>删除</el-button>
                  </template>
                </el-popconfirm>
                <span v-else>-</span>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination-wrap" v-if="appsTotal > 0">
            <el-pagination v-model:current-page="appsPage" v-model:page-size="appsPageSize" :page-sizes="[2, 5, 10, 20, 50]" :total="appsTotal" layout="total, sizes, prev, pager, next" @current-change="fetchApplications" @size-change="changeAppsPageSize" />
          </div>
        </template>
      </el-tab-pane>

      <el-tab-pane label="收到的申请" name="received">
        <el-empty v-if="!receivedLoading && receivedApplications.length === 0" description="暂未收到申请" />
        <template v-else>
          <el-table :data="receivedApplications" stripe v-loading="receivedLoading">
            <el-table-column prop="project_title" label="项目标题" min-width="150" />
            <el-table-column prop="applicant_nickname" label="申请人" width="110" />
            <el-table-column label="申请说明" min-width="180" show-overflow-tooltip>
              <template #default="{ row }">{{ row.introduction || '-' }}</template>
            </el-table-column>
            <el-table-column prop="contact" label="联系方式" width="160" />
            <el-table-column label="状态" width="90">
              <template #default="{ row }">
                <el-tag :type="row.status === 'accepted' ? 'success' : row.status === 'pending' ? 'warning' : 'info'">
                  {{ appStatusLabel(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <template v-if="row.status === 'pending'">
                  <el-button type="success" size="small" link @click="handleReceivedApplication(row.id, 'accepted')">同意</el-button>
                  <el-button type="danger" size="small" link @click="handleReceivedApplication(row.id, 'rejected')">拒绝</el-button>
                </template>
                <span v-else>-</span>
              </template>
            </el-table-column>
          </el-table>
          <div class="pagination-wrap" v-if="receivedTotal > 0">
            <el-pagination v-model:current-page="receivedPage" v-model:page-size="receivedPageSize" :page-sizes="[2, 5, 10, 20, 50]" :total="receivedTotal" layout="total, sizes, prev, pager, next" @current-change="fetchReceivedApplications" @size-change="changeReceivedPageSize" />
          </div>
        </template>
      </el-tab-pane>
    </el-tabs>

    <!-- Edit nickname dialog -->
    <el-dialog v-model="showNicknameDialog" title="修改昵称" width="400px">
      <el-input v-model="newNickname" placeholder="请输入新昵称" />
      <template #footer>
        <el-button @click="showNicknameDialog = false">取消</el-button>
        <el-button type="primary" @click="updateNickname">保存</el-button>
      </template>
    </el-dialog>

    <!-- Change password dialog -->
    <el-dialog v-model="showPasswordDialog" title="修改密码" width="400px">
      <el-form :model="passwordForm" label-width="80px">
        <el-form-item label="原密码">
          <el-input v-model="passwordForm.old_password" type="password" show-password />
        </el-form-item>
        <el-form-item label="新密码">
          <el-input v-model="passwordForm.new_password" type="password" show-password />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showPasswordDialog = false">取消</el-button>
        <el-button type="primary" @click="updatePassword">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import {
  getProfile, updateProfile, changePassword,
  getMyMaterials, getMyReviews, getMyProjects, getMyApplications, getReceivedApplications,
  handleProjectApplication,
  deleteMaterial, deleteReview, updateOwnProjectStatus, deleteApplication,
} from '@/api/profile'
import type { ProfileData, Material, Review, Project, ProjectApplication } from '@/types'

const profile = ref<ProfileData | null>(null)
const userLoading = ref(false)
const activeTab = ref('materials')
const showNicknameDialog = ref(false)
const showPasswordDialog = ref(false)
const newNickname = ref('')
const passwordForm = reactive({ old_password: '', new_password: '' })

const materials = ref<Material[]>([])
const materialsTotal = ref(0)
const materialsPage = ref(1)
const materialsPageSize = ref(10)
const materialsLoading = ref(false)

const reviews = ref<Review[]>([])
const reviewsTotal = ref(0)
const reviewsPage = ref(1)
const reviewsPageSize = ref(10)
const reviewsLoading = ref(false)

const myProjects = ref<Project[]>([])
const projectsTotal = ref(0)
const projectsPage = ref(1)
const projectsPageSize = ref(10)
const projectsLoading = ref(false)

const applications = ref<ProjectApplication[]>([])
const appsTotal = ref(0)
const appsPage = ref(1)
const appsPageSize = ref(10)
const appsLoading = ref(false)

const receivedApplications = ref<ProjectApplication[]>([])
const receivedTotal = ref(0)
const receivedPage = ref(1)
const receivedPageSize = ref(10)
const receivedLoading = ref(false)

const expProgress = computed(() => {
  if (!profile.value) return 0
  const exp = profile.value.user.experience
  const thresholds = [0, 0, 10, 30, 60, 100]
  const level = profile.value.level
  const base = thresholds[level] ?? 0
  const next = thresholds[level + 1]
  if (next === undefined) return 100
  return Math.min(Math.round(((exp - base) / (next - base)) * 100), 100)
})

function statusLabel(s: string) {
  const map: Record<string, string> = { pending: '待审核', approved: '已发布', rejected: '已驳回', hidden: '已隐藏', visible: '可见', deleted: '已删除', recruiting: '招募中', full: '已满员', closed: '已关闭' }
  return map[s] || s
}

function appStatusLabel(s: string) {
  const map: Record<string, string> = { pending: '待处理', accepted: '已通过', rejected: '已拒绝' }
  return map[s] || s
}

function typeLabel(t: string) {
  const map: Record<string, string> = { course_project: '课程项目', competition: '比赛组队', research: '科研项目', personal: '个人项目' }
  return map[t] || t
}

function typeColor(t: string) {
  const map: Record<string, string> = { course_project: 'primary', competition: 'warning', research: 'success', personal: 'info' }
  return map[t] || ''
}

async function fetchUser() {
  userLoading.value = true
  try {
    const res = await getProfile()
    profile.value = res.data
  } finally {
    userLoading.value = false
  }
}

async function fetchMaterials() {
  materialsLoading.value = true
  try {
    const res = await getMyMaterials({ page: materialsPage.value, page_size: materialsPageSize.value })
    materials.value = res.data.items
    materialsTotal.value = res.data.total
  } finally {
    materialsLoading.value = false
  }
}

async function fetchReviews() {
  reviewsLoading.value = true
  try {
    const res = await getMyReviews({ page: reviewsPage.value, page_size: reviewsPageSize.value })
    reviews.value = res.data.items
    reviewsTotal.value = res.data.total
  } finally {
    reviewsLoading.value = false
  }
}

async function fetchMyProjects() {
  projectsLoading.value = true
  try {
    const res = await getMyProjects({ page: projectsPage.value, page_size: projectsPageSize.value })
    myProjects.value = res.data.items
    projectsTotal.value = res.data.total
  } finally {
    projectsLoading.value = false
  }
}

async function fetchApplications() {
  appsLoading.value = true
  try {
    const res = await getMyApplications({ page: appsPage.value, page_size: appsPageSize.value })
    applications.value = res.data.items
    appsTotal.value = res.data.total
  } finally {
    appsLoading.value = false
  }
}

async function fetchReceivedApplications() {
  receivedLoading.value = true
  try {
    const res = await getReceivedApplications({ page: receivedPage.value, page_size: receivedPageSize.value })
    receivedApplications.value = res.data.items
    receivedTotal.value = res.data.total
  } finally {
    receivedLoading.value = false
  }
}

function onTabChange(tab: string) {
  switch (tab) {
    case 'materials': if (materials.value.length === 0) fetchMaterials(); break
    case 'reviews': if (reviews.value.length === 0) fetchReviews(); break
    case 'projects': if (myProjects.value.length === 0) fetchMyProjects(); break
    case 'applications': if (applications.value.length === 0) fetchApplications(); break
    case 'received': if (receivedApplications.value.length === 0) fetchReceivedApplications(); break
  }
}

function changeMaterialsPageSize() {
  materialsPage.value = 1
  fetchMaterials()
}

function changeReviewsPageSize() {
  reviewsPage.value = 1
  fetchReviews()
}

function changeProjectsPageSize() {
  projectsPage.value = 1
  fetchMyProjects()
}

function changeAppsPageSize() {
  appsPage.value = 1
  fetchApplications()
}

function changeReceivedPageSize() {
  receivedPage.value = 1
  fetchReceivedApplications()
}

function openMaterialLink(link: string | null) {
  if (link) window.open(link, '_blank', 'noopener,noreferrer')
}

async function updateNickname() {
  if (!newNickname.value.trim()) return
  try {
    await updateProfile({ nickname: newNickname.value })
    ElMessage.success('昵称已更新')
    showNicknameDialog.value = false
    fetchUser()
  } catch { /* handled */ }
}

async function updatePassword() {
  if (!passwordForm.old_password || !passwordForm.new_password) {
    ElMessage.warning('请填写完整')
    return
  }
  try {
    await changePassword({ ...passwordForm })
    ElMessage.success('密码已修改')
    showPasswordDialog.value = false
  } catch { /* handled */ }
}

async function handleDeleteMaterial(id: number) {
  try {
    await deleteMaterial(id)
    ElMessage.success('资料已删除')
    fetchMaterials()
  } catch { /* handled */ }
}

async function handleDeleteReview(id: number) {
  try {
    await deleteReview(id)
    ElMessage.success('评价已删除')
    fetchReviews()
  } catch { /* handled */ }
}

async function handleProjectStatus(id: number, status: 'closed' | 'hidden') {
  try {
    await updateOwnProjectStatus(id, status)
    ElMessage.success(status === 'closed' ? '招募已关闭' : '招募已隐藏')
    fetchMyProjects()
  } catch { /* handled */ }
}

async function handleDeleteApplication(id: number) {
  try {
    await deleteApplication(id)
    ElMessage.success('申请已删除')
    fetchApplications()
  } catch { /* handled */ }
}

async function handleReceivedApplication(id: number, status: 'accepted' | 'rejected') {
  try {
    await handleProjectApplication(id, status)
    ElMessage.success(status === 'accepted' ? '已同意申请' : '已拒绝申请')
    fetchReceivedApplications()
    fetchMyProjects()
  } catch { /* handled */ }
}

onMounted(() => {
  fetchUser()
  fetchMaterials()
})
</script>

<style scoped>
.profile-page {
  max-width: 1200px;
  margin: 0 auto;
}
.user-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}
.user-header h2 {
  margin: 0 0 8px;
}
.user-header p {
  color: #606266;
  margin: 4px 0;
  font-size: 14px;
}
.pagination-wrap {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}
</style>
