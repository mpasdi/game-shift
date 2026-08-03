<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import {
    CheckSquare,
    ChevronDown,
    ChevronRight,
    CircleCheck,
    CircleHelp,
    Info,
    ListChecks,
    MinusSquare,
    Sparkles,
    Square
  } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import type { ScanCandidate } from '../types/game'

  interface ScanRow extends ScanCandidate {
    selected: boolean
  }

  interface RecognitionTooltip {
    exePath: string
    confidence: number
    reasons: string[]
    top: number
    left: number
    width: number
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
  const recommendedExpanded = ref(true)
  const otherExpanded = ref(false)
  const existingExpanded = ref(false)
  const recognitionTooltip = ref<RecognitionTooltip | null>(null)

  const selectedRows = computed(() => rows.value.filter((row) => !row.exists && row.selected))
  const selectedCount = computed(() => selectedRows.value.length)
  const recommendedRows = computed(() => rows.value.filter((row) => !row.exists && row.recommended))
  const otherRows = computed(() => rows.value.filter((row) => !row.exists && !row.recommended))
  const existingRows = computed(() => rows.value.filter((row) => row.exists))
  const candidateSections = computed(() => [
    {
      key: 'recommended',
      title: '推荐游戏',
      description: '综合证据最强，已默认选择',
      rows: recommendedRows.value,
      expanded: recommendedExpanded.value,
      collapsible: true
    },
    {
      key: 'other',
      title: '其他可执行文件',
      description: '证据不足或存在多个相近候选，请手动确认',
      rows: otherRows.value,
      expanded: otherExpanded.value,
      collapsible: true
    },
    {
      key: 'existing',
      title: '已存在',
      description: '启动路径已经在游戏库中',
      rows: existingRows.value,
      expanded: existingExpanded.value,
      collapsible: true
    }
  ])

  watch(
    () => [props.open, props.candidates] as const,
    ([open, candidates]) => {
      rows.value = open
        ? candidates.map((candidate) => ({
            ...candidate,
            selected: candidate.recommended && !candidate.exists
          }))
        : []
      recommendedExpanded.value = true
      otherExpanded.value = false
      existingExpanded.value = false
      recognitionTooltip.value = null
    },
    { immediate: true }
  )

  function toggleRow(row: ScanRow) {
    if (row.exists) return
    row.selected = !row.selected
  }

  function sectionSelectionState(sectionRows: ScanRow[]) {
    const selectableRows = sectionRows.filter((row) => !row.exists)
    const selectedRows = selectableRows.filter((row) => row.selected)
    if (selectableRows.length > 0 && selectedRows.length === selectableRows.length) return 'all'
    if (selectedRows.length > 0) return 'some'
    return 'none'
  }

  function toggleSectionSelection(sectionRows: ScanRow[]) {
    const selectableRows = sectionRows.filter((row) => !row.exists)
    const shouldSelect = sectionSelectionState(selectableRows) !== 'all'
    for (const row of selectableRows) row.selected = shouldSelect
  }

  function toggleSection(section: string) {
    if (section === 'recommended') {
      recommendedExpanded.value = !recommendedExpanded.value
      return
    }
    if (section === 'other') {
      otherExpanded.value = !otherExpanded.value
      return
    }
    if (section === 'existing') existingExpanded.value = !existingExpanded.value
  }

  function showRecognitionTooltip(row: ScanRow, event: MouseEvent | FocusEvent) {
    const trigger = event.currentTarget as HTMLElement
    const rect = trigger.getBoundingClientRect()
    const width = Math.min(280, window.innerWidth - 24)
    const preferredLeft = rect.left - width - 8
    const left = preferredLeft >= 12 ? preferredLeft : Math.min(rect.right + 8, window.innerWidth - width - 12)

    recognitionTooltip.value = {
      exePath: row.exePath,
      confidence: row.confidence,
      reasons: row.reasons,
      top: Math.min(Math.max(rect.top + rect.height / 2, 72), window.innerHeight - 72),
      left,
      width
    }
  }

  function hideRecognitionTooltip() {
    recognitionTooltip.value = null
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
  <BaseModal
    :open="props.open"
    title="扫描结果"
    size="lg"
    :body-scrollable="false"
    :close-on-backdrop="false"
    :close-disabled="props.importing"
    @close="emit('close')"
  >
    <div class="scan-dialog">
      <div class="scan-summary">
        <div class="scan-summary-item">
          <span class="scan-summary-icon scan-summary-icon--recommended" aria-hidden="true">
            <Sparkles :size="16" />
          </span>
          <span>推荐游戏</span>

          <strong>{{ recommendedRows.length }}</strong>
        </div>

        <div class="scan-summary-item">
          <span class="scan-summary-icon scan-summary-icon--other" aria-hidden="true">
            <CircleHelp :size="16" />
          </span>
          <span>其他程序</span>

          <strong>{{ otherRows.length }}</strong>
        </div>

        <div class="scan-summary-item">
          <span class="scan-summary-icon scan-summary-icon--existing" aria-hidden="true">
            <CircleCheck :size="16" />
          </span>
          <span>已存在</span>

          <strong>{{ existingRows.length }}</strong>
        </div>

        <div class="scan-summary-item scan-summary-item--selected">
          <span class="scan-summary-icon scan-summary-icon--selected" aria-hidden="true">
            <ListChecks :size="16" />
          </span>
          <span>已选程序</span>

          <strong>{{ selectedCount }}</strong>
        </div>
      </div>

      <p v-if="props.errorMessage" class="scan-error">{{ props.errorMessage }}</p>

      <div v-if="rows.length === 0" class="scan-empty">没有扫描到可导入的 .exe 文件。</div>
      <div v-else class="scan-table" role="table" aria-label="扫描候选列表" @scroll="hideRecognitionTooltip">
        <div class="scan-row scan-row--head" role="row">
          <span></span>
          <span>游戏名称</span>
          <span>启动程序</span>
        </div>

        <template v-for="section in candidateSections" :key="section.key">
          <div v-if="section.rows.length > 0" class="scan-section-header" role="row">
            <div class="scan-section-controls">
              <button
                class="section-toggle-button"
                type="button"
                :aria-label="section.expanded ? `收起${section.title}` : `展开${section.title}`"
                :aria-expanded="section.expanded"
                @click="toggleSection(section.key)"
              >
                <ChevronDown v-if="section.expanded" :size="16" />
                <ChevronRight v-else :size="16" />
              </button>
              <button
                v-if="section.key !== 'existing'"
                class="check-button"
                type="button"
                :aria-label="
                  sectionSelectionState(section.rows) === 'all' ? `取消全选${section.title}` : `全选${section.title}`
                "
                @click="toggleSectionSelection(section.rows)"
              >
                <CheckSquare v-if="sectionSelectionState(section.rows) === 'all'" :size="16" />
                <MinusSquare v-else-if="sectionSelectionState(section.rows) === 'some'" :size="16" />
                <Square v-else :size="16" />
              </button>
            </div>
            <button class="scan-section-title" type="button" @click="toggleSection(section.key)">
              <strong>{{ section.title }}</strong>
              <small>{{ section.description }}</small>
            </button>
          </div>

          <template v-if="section.expanded">
            <div
              v-for="row in section.rows"
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
                <div class="path-heading">
                  <strong>{{ row.exeFileName }}</strong>
                  <button
                    v-if="row.reasons.length"
                    class="recognition-trigger"
                    type="button"
                    aria-label="查看识别依据"
                    :aria-describedby="recognitionTooltip?.exePath === row.exePath ? 'recognition-tooltip' : undefined"
                    @mouseenter="showRecognitionTooltip(row, $event)"
                    @mouseleave="hideRecognitionTooltip"
                    @focus="showRecognitionTooltip(row, $event)"
                    @blur="hideRecognitionTooltip"
                  >
                    <Info :size="15" />
                  </button>
                </div>
                <span :title="row.folderPath">{{ row.folderPath }}</span>
              </div>
            </div>
          </template>
        </template>
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

  <Teleport to="body">
    <div
      v-if="recognitionTooltip"
      id="recognition-tooltip"
      class="recognition-tooltip"
      role="tooltip"
      :style="{
        top: `${recognitionTooltip.top}px`,
        left: `${recognitionTooltip.left}px`,
        width: `${recognitionTooltip.width}px`
      }"
    >
      <div class="recognition-tooltip-heading">
        <strong>识别依据</strong>
        <span>识别分 {{ recognitionTooltip.confidence }} / 100</span>
      </div>
      <ul>
        <li v-for="reason in recognitionTooltip.reasons" :key="reason">{{ reason }}</li>
      </ul>
    </div>
  </Teleport>
</template>

<style scoped>
  .scan-dialog {
    display: grid;
    gap: 14px;
    min-height: 0;
  }

  .scan-summary {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    width: 100%;
    gap: 8px;
  }

  .scan-summary-item {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.04);
    padding: 6px 8px;
  }

  .scan-summary-icon {
    display: grid;
    width: 28px;
    height: 28px;
    margin-right: 1px;
    border-radius: 999px;
    place-items: center;
  }

  .scan-summary-icon--recommended {
    background: rgba(139, 92, 246, 0.16);
    color: #c4b5fd;
  }

  .scan-summary-icon--other {
    background: rgba(245, 158, 11, 0.13);
    color: #fcd34d;
  }

  .scan-summary-icon--existing {
    background: rgba(34, 197, 94, 0.13);
    color: #86efac;
  }

  .scan-summary-icon--selected {
    background: rgba(139, 92, 246, 0.2);
    color: var(--accent-strong);
  }

  .scan-summary strong {
    margin-left: auto;
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

  .scan-summary span:not(.scan-summary-icon),
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
    grid-template-columns: 64px minmax(130px, 190px) minmax(0, 1fr);
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

  .scan-section-header {
    display: grid;
    grid-template-columns: 64px minmax(0, 1fr);
    width: 100%;
    gap: 10px;
    align-items: center;
    border: 0;
    border-bottom: 1px solid var(--border);
    background: rgba(124, 92, 255, 0.08);
    color: var(--text);
    padding: 9px 10px;
    text-align: left;
  }

  .scan-section-header:hover {
    background: rgba(124, 92, 255, 0.13);
  }

  .scan-section-controls {
    display: flex;
    align-items: center;
  }

  .section-toggle-button {
    display: grid;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    padding: 0;
    place-items: center;
  }

  .section-toggle-button:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .scan-section-title {
    display: grid;
    width: 100%;
    gap: 2px;
    border: 0;
    background: transparent;
    color: inherit;
    padding: 0;
    text-align: left;
  }

  .scan-section-title small {
    color: var(--text-muted);
    font-size: var(--font-size-xs);
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

  .scan-row > .check-button {
    justify-self: start;
    margin-left: 28px;
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

  .path-heading {
    display: flex;
    min-width: 0;
    gap: 5px;
    align-items: center;
  }

  .path-cell strong,
  .path-cell span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .path-cell strong {
    min-width: 0;
    color: rgba(245, 242, 255, 0.9);
    font-size: var(--font-size-sm);
  }

  .path-cell span {
    color: var(--text-muted);
    font-size: var(--font-size-xs);
  }

  .recognition-trigger {
    display: grid;
    width: 24px;
    min-width: 24px;
    height: 24px;
    border: 0;
    border-radius: 999px;
    outline: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 0;
    place-items: center;
  }

  .recognition-trigger:hover,
  .recognition-trigger:focus-visible {
    background: transparent;
    color: var(--accent-strong);
  }

  .recognition-trigger:focus-visible {
    outline: 1px solid var(--accent-strong);
    outline-offset: 2px;
  }

  .recognition-tooltip {
    position: fixed;
    z-index: 3000;
    max-height: calc(100vh - 24px);
    overflow: auto;
    transform: translateY(-50%);
    border: 1px solid rgba(139, 92, 246, 0.32);
    border-radius: 9px;
    background: rgba(24, 21, 32, 0.98);
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.38);
    color: var(--text);
    padding: 10px 12px;
    pointer-events: none;
  }

  .recognition-tooltip-heading {
    display: flex;
    gap: 10px;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .recognition-tooltip-heading strong {
    color: var(--accent-strong);
    font-size: var(--font-size-sm);
  }

  .recognition-tooltip-heading span {
    color: var(--text-subtle);
    font-size: var(--font-size-xs);
    white-space: nowrap;
  }

  .recognition-tooltip ul {
    display: grid;
    gap: 4px;
    margin: 0;
    padding-left: 17px;
  }

  .recognition-tooltip li {
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    line-height: 1.45;
  }

  @media (max-width: 720px) {
    .scan-summary {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .scan-row {
      grid-template-columns: 64px minmax(0, 1fr);
    }

    .scan-section-header {
      grid-template-columns: 64px minmax(0, 1fr);
    }

    .scan-row--head span:nth-of-type(2),
    .scan-row--head span:nth-of-type(3),
    .path-cell {
      grid-column: 2;
    }
  }
</style>
