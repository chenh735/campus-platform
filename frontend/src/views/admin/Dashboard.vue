<template>
  <div class="admin-dashboard" v-loading="loading">
    <h2>后台首页</h2>
    <el-row :gutter="20" class="stats-row">
      <el-col :xs="12" :sm="8" :md="4" v-for="s in stats" :key="s.label">
        <el-card class="stat-card">
          <div class="stat-value">{{ s.value }}</div>
          <div class="stat-label">{{ s.label }}</div>
        </el-card>
      </el-col>
    </el-row>

    <el-card class="activity-panel">
      <template #header><h3 style="margin:0;">近 30 天每日活跃人数</h3></template>
      <el-table :data="dailyActiveCounts" height="360">
        <el-table-column prop="date" label="日期" width="160" />
        <el-table-column prop="count" label="活跃人数" width="120" />
        <el-table-column label="趋势">
          <template #default="{ row }">
            <div class="activity-bar-track">
              <div class="activity-bar" :style="{ width: activityBarWidth(row.count) }"></div>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-card>
      <template #header><h3 style="margin:0;">最近操作记录</h3></template>
      <el-table :data="logs" v-loading="logsLoading">
        <el-table-column prop="action" label="操作" width="150" />
        <el-table-column prop="target_type" label="对象类型" width="120" />
        <el-table-column prop="detail" label="详情" />
        <el-table-column prop="created_at" label="时间" width="180" />
      </el-table>
      <div class="pagination-wrap" v-if="logsTotal > 0">
        <el-pagination
          v-model:current-page="logsPage"
          v-model:page-size="logPageSize"
          :page-sizes="[2, 5, 10, 20, 50]"
          :total="logsTotal"
          layout="total, sizes, prev, pager, next"
          @current-change="fetchLogs"
          @size-change="changeLogPageSize"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { getDashboard, getAdminLogs } from '@/api/admin'
import type { DailyActiveCount, AdminLog } from '@/types'

const stats = ref([
  { label: '用户数', value: 0 },
  { label: '课程数', value: 0 },
  { label: '资料数', value: 0 },
  { label: '项目招募', value: 0 },
  { label: '待审核', value: 0 },
  { label: '今日活跃', value: 0 },
])
const dailyActiveCounts = ref<DailyActiveCount[]>([])
const maxActiveCount = computed(() => Math.max(1, ...dailyActiveCounts.value.map(item => item.count)))
const loading = ref(false)
const logs = ref<AdminLog[]>([])
const logsTotal = ref(0)
const logsPage = ref(1)
const logPageSize = ref(10)
const logsLoading = ref(false)

async function fetchData() {
  loading.value = true
  try {
    const dashRes = await getDashboard()
    const d = dashRes.data
    stats.value[0].value = d.user_count
    stats.value[1].value = d.course_count
    stats.value[2].value = d.material_count
    stats.value[3].value = d.project_count
    stats.value[4].value = d.pending_audit_count
    stats.value[5].value = d.today_active_count
    dailyActiveCounts.value = [...d.daily_active_counts].reverse()
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

function activityBarWidth(count: number) {
  return `${Math.max(count === 0 ? 0 : 4, (count / maxActiveCount.value) * 100)}%`
}

async function fetchLogs() {
  logsLoading.value = true
  try {
    const res = await getAdminLogs({ page: logsPage.value, page_size: logPageSize.value })
    logs.value = res.data.logs
    logsTotal.value = res.data.total
  } catch {
    // handled
  } finally {
    logsLoading.value = false
  }
}

function changeLogPageSize() {
  logsPage.value = 1
  fetchLogs()
}

onMounted(() => {
  fetchData()
  fetchLogs()
})
</script>

<style scoped>
.admin-dashboard {
  max-width: 1200px;
}
.stats-row {
  margin-bottom: 20px;
}
.stat-card {
  text-align: center;
  margin-bottom: 20px;
}
.stat-value {
  font-size: 28px;
  font-weight: bold;
  color: #409eff;
}
.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 8px;
}
.activity-panel {
  margin-bottom: 20px;
}
.activity-bar-track {
  height: 12px;
  width: 100%;
  max-width: 520px;
  border-radius: 6px;
  background: #edf1f7;
  overflow: hidden;
}
.activity-bar {
  height: 100%;
  border-radius: 6px;
  background: #409eff;
}
.pagination-wrap {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}
</style>
