<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import { CheckSquare, Square } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import type { ScanCandidate } from '../types/game'

  interface ScanRow extends ScanCandidate {
    selected: boolean
  }

  const props = withDefaults(
    defineProps<{
      open: boolean
      candidates: ScanCandidate[]
      importing?: boolean
      errorMessage?: string | null
    }>(),
    {
      importing: false,
      errorMessage: null
    }
  )

  const emit = defineEmits<{
    close: []
    import: [candidates: ScanCandidate[]]
  }>()

  const rows = ref<ScanRow[]>([])

  const importableRows = computed(() => rows.value.filter((row) => !row.exists))
  const selectedRows = computed(() => importableRows.value.filter((row) => row.selected))
  const selectedCount = computed(() => selectedRows.value.length)
  const allSelected = computed(
    () => importableRows.value.length > 0 && selectedCount.value === importableRows.value.length
  )

  watch(
    () => [props.open, props.candidates] as const,
    ([open, candidates]) => {
      rows.value = open ? candidates.map((candidate) => ({ ...candidate, selected: !candidate.exists })) : []
    },
    { immediate: true }
  )

  function toggleRow(row: ScanRow) {
    if (row.exists) return
    row.selected = !row.selected
  }

  function toggleAll() {
    const nextSelected = !allSelected.value
    rows.value = rows.value.map((row) => (row.exists ? row : { ...row, selected: nextSelected }))
  }

  function importSelected() {
    emit(
      'import',
      selectedRows.value.map(({ selected: _selected, ...candidate }) => ({
        ...candidate,
        name: candidate.name.trim() || candidate.exeFileName.replace(/\.exe$/i, '')
      }))
    )
  }
</script>

<template>
  <BaseModal :open="props.open" title="扫描结果" size="lg" :body-scrollable="false" @close="emit('close')">
    <div class="scan-dialog">
      <div class="scan-summary">
        <div class="scan-summary-item">
          <span>全部程序</span>

          <strong>{{ props.candidates.length }}</strong>
        </div>

        <div class="scan-summary-item scan-summary-item--selected">
          <span>已选程序</span>

          <strong>{{ selectedCount }}</strong>
        </div>
      </div>

      <p v-if="props.errorMessage" class="scan-error">{{ props.errorMessage }}</p>

      <div v-if="rows.length === 0" class="scan-empty">没有扫描到可导入的 .exe 文件。</div>
      <div v-else class="scan-table" role="table" aria-label="扫描候选列表">
        <div class="scan-row scan-row--head" role="row">
          <button class="check-button" type="button" :disabled="importableRows.length === 0" @click="toggleAll">
            <CheckSquare v-if="allSelected" :size="16" />
            <Square v-else :size="16" />
          </button>
          <span>游戏名称</span>
          <span>启动程序</span>
          <span>状态</span>
        </div>

        <div
          v-for="row in rows"
          :key="row.exePath"
          class="scan-row"
          :class="{ 'scan-row--disabled': row.exists }"
          role="row"
        >
          <button class="check-button" type="button" :disabled="row.exists" @click="toggleRow(row)">
            <CheckSquare v-if="row.selected" :size="16" />
            <Square v-else :size="16" />
          </button>
          <input v-model="row.name" class="name-input" :disabled="row.exists" />
          <div class="path-cell">
            <strong>{{ row.exeFileName }}</strong>
            <span :title="row.exePath">{{ row.exePath }}</span>
          </div>
          <span class="status-pill" :class="{ 'status-pill--exists': row.exists }">
            {{ row.exists ? '已存在' : '新发现' }}
          </span>
        </div>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="ghost" type="button" :disabled="props.importing" @click="emit('close')">取消</BaseButton>
      <BaseButton
        variant="primary"
        type="button"
        :loading="props.importing"
        :disabled="selectedCount === 0"
        @click="importSelected"
      >
        导入
      </BaseButton>
    </template>
  </BaseModal>
</template>

<style scoped>
  .scan-dialog {
    display: grid;
    gap: 14px;
    min-height: 0;
  }

  .scan-summary {
    display: inline-flex;
    width: fit-content;
    gap: 6px;
    align-items: center;
  }

  .scan-summary-item {
    display: inline-flex;
    gap: 6px;
    align-items: baseline;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.04);
    padding: 6px 8px;
  }

  .scan-summary strong {
    color: var(--text);
    font-size: var(--font-size-lg);
  }

  .scan-summary-item--selected {
    background: rgba(139, 92, 246, 0.16);
  }

  .scan-summary-item--selected strong,
  .scan-summary-item--selected span {
    color: var(--accent-strong);
  }

  .scan-summary span,
  .scan-empty {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .scan-error {
    margin: 0;
    color: #fecdd3;
    font-size: var(--font-size-md);
  }

  .scan-table {
    max-height: min(520px, calc(100vh - 320px));
    min-height: 0;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
  }

  .scan-row {
    display: grid;
    grid-template-columns: 34px minmax(130px, 190px) minmax(0, 1fr) 76px;
    gap: 10px;
    align-items: center;
    min-height: 52px;
    border-bottom: 1px solid var(--border);
    padding: 8px 10px;
  }

  .scan-row:last-child {
    border-bottom: 0;
  }

  .scan-table .scan-row:not(.scan-row--head):last-child {
    border-bottom: 0;
  }

  .scan-row--head {
    position: sticky;
    top: 0;
    z-index: 1;
    min-height: 38px;
    background: rgba(32, 29, 42, 0.98);
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    font-weight: 700;
  }

  .scan-row--disabled {
    opacity: 0.58;
  }

  .check-button {
    display: grid;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--accent-strong);
    padding: 0;
    place-items: center;
  }

  .check-button:hover:not(:disabled) {
    background: var(--accent-soft);
  }

  .check-button:disabled {
    color: var(--text-subtle);
  }

  .name-input {
    width: 100%;
    min-width: 0;
    border: 1px solid transparent;
    border-radius: 7px;
    outline: 0;
    background: transparent;
    color: var(--text);
    font-weight: 700;
    padding: 7px 8px;
  }

  .name-input:hover:not(:disabled),
  .name-input:focus {
    border-color: var(--border);
    background: var(--surface);
  }

  .path-cell {
    display: grid;
    gap: 4px;
    min-width: 0;
  }

  .path-cell strong,
  .path-cell span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .path-cell strong {
    color: rgba(245, 242, 255, 0.9);
    font-size: var(--font-size-sm);
  }

  .path-cell span {
    color: var(--text-muted);
    font-size: var(--font-size-xs);
  }

  .status-pill {
    justify-self: start;
    border: 1px solid rgba(124, 92, 255, 0.34);
    border-radius: 999px;
    background: rgba(124, 92, 255, 0.13);
    color: var(--accent-strong);
    font-size: var(--font-size-sm);
    padding: 4px 8px;
  }

  .status-pill--exists {
    border-color: var(--border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-muted);
  }

  @media (max-width: 720px) {
    .scan-row {
      grid-template-columns: 34px minmax(0, 1fr);
    }

    .scan-row--head span:nth-of-type(2),
    .scan-row--head span:nth-of-type(3),
    .path-cell,
    .status-pill {
      grid-column: 2;
    }
  }
</style>
