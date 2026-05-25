<template>
  <div class="admin-audit">
    <h2>内容管理</h2>
    <el-card>
      <el-form :inline="true" :model="query" class="filter-form">
        <el-form-item label="状态">
          <el-select v-model="query.status" style="width: 120px;">
            <el-option label="待审核" value="pending" />
            <el-option label="已通过" value="approved" />
            <el-option label="已驳回" value="rejected" />
            <el-option label="已隐藏" value="hidden" />
            <el-option label="已删除" value="deleted" />
          </el-select>
        </el-form-item>
        <el-form-item label="搜索">
          <el-input v-model="query.keyword" placeholder="标题/作者/课程" clearable />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="search">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="items" v-loading="loading">
        <el-table-column prop="title" label="标题" />
        <el-table-column prop="item_type" label="类型" width="100">
          <template #default="{ row }">
            <el-tag>{{ row.item_type === 'material' ? '学习资料' : row.item_type }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="author" label="作者" width="120" />
        <el-table-column prop="related" label="关联课程" width="150" />
        <el-table-column label="资源" width="90">
          <template #default="{ row }">
            <el-button v-if="row.resource_type === 'link'" type="primary" link @click="openLink(row.link_url)">查看链接</el-button>
            <span v-else>文件</span>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="提交时间" width="180" />
        <el-table-column label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="statusTagType(row.status)">{{ statusText(row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="260">
          <template #default="{ row }">
            <el-button v-if="row.status !== 'approved' && row.status !== 'deleted'" size="small" type="success" @click="handleAudit(row.id, 'approved')">通过</el-button>
            <el-button v-if="row.status !== 'rejected' && row.status !== 'deleted'" size="small" type="danger" @click="handleAudit(row.id, 'rejected')">驳回</el-button>
            <el-button v-if="row.status !== 'hidden' && row.status !== 'deleted'" size="small" type="warning" @click="handleAudit(row.id, 'hidden')">隐藏</el-button>
            <el-popconfirm v-if="row.status !== 'deleted'" title="确定删除这条资料吗？" @confirm="handleDelete(row.id)">
              <template #reference>
                <el-button size="small" type="danger">删除</el-button>
              </template>
            </el-popconfirm>
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
          @current-change="fetchItems"
          @size-change="changePageSize"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { getAuditItems, auditMaterial, deleteMaterialByAdmin } from '@/api/admin'
import type { AuditItem } from '@/types'

const items = ref<AuditItem[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(10)
const loading = ref(false)
const query = reactive({ keyword: '', status: 'approved' })

async function fetchItems() {
  loading.value = true
  try {
    const res = await getAuditItems({ ...query, page: page.value, page_size: pageSize.value })
    items.value = res.data.items
    total.value = res.data.total
  } catch {
    // handled
  } finally {
    loading.value = false
  }
}

function search() {
  page.value = 1
  fetchItems()
}

function openLink(link: string | null) {
  if (link) window.open(link, '_blank', 'noopener,noreferrer')
}

function changePageSize() {
  page.value = 1
  fetchItems()
}

async function handleAudit(id: number, status: string) {
  try {
    await auditMaterial(id, status)
    ElMessage.success(statusMessage(status))
    fetchItems()
  } catch {
    // handled
  }
}

async function handleDelete(id: number) {
  try {
    await deleteMaterialByAdmin(id)
    ElMessage.success('资料已删除')
    fetchItems()
  } catch {
    // handled
  }
}

function statusText(status: string) {
  return ({ pending: '待审核', approved: '已发布', rejected: '已驳回', hidden: '已隐藏', deleted: '已删除' } as Record<string, string>)[status] || status
}

function statusMessage(status: string) {
  return ({ approved: '已发布', rejected: '已驳回', hidden: '已隐藏' } as Record<string, string>)[status] || '状态已更新'
}

function statusTagType(status: string) {
  return ({ approved: 'success', pending: 'warning', rejected: 'danger', hidden: 'info', deleted: 'info' } as Record<string, string>)[status] || 'info'
}

onMounted(fetchItems)
</script>

<style scoped>
.admin-audit {
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
