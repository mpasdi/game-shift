<script setup lang="ts">
  import { computed } from 'vue'

  interface DataTableColumn {
    key: string
    label: string
    width?: string
    align?: 'left' | 'center' | 'right'
  }

  const props = withDefaults(
    defineProps<{
      columns: DataTableColumn[]
      rows: unknown[]
      rowKey?: (row: unknown, index: number) => string | number
      density?: 'compact' | 'regular'
      ariaLabel?: string
    }>(),
    {
      rowKey: undefined,
      density: 'regular',
      ariaLabel: '数据表格'
    }
  )

  const gridTemplateColumns = computed(() => props.columns.map((column) => column.width ?? 'minmax(0, 1fr)').join(' '))

  function getRowKey(row: unknown, index: number) {
    return props.rowKey?.(row, index) ?? index
  }

  function getCellValue(row: unknown, key: string) {
    if (!row || typeof row !== 'object') return ''
    return (row as Record<string, unknown>)[key] ?? ''
  }

  function getAlignClass(column: DataTableColumn) {
    return `data-table__cell--${column.align ?? 'left'}`
  }
</script>

<template>
  <div class="data-table" :class="`data-table--${props.density}`" role="table" :aria-label="props.ariaLabel">
    <div class="data-table__header" role="row" :style="{ gridTemplateColumns }">
      <div
        v-for="column in props.columns"
        :key="column.key"
        class="data-table__heading"
        :class="getAlignClass(column)"
        role="columnheader"
      >
        {{ column.label }}
      </div>
    </div>

    <div class="data-table__body" role="rowgroup">
      <div
        v-for="(row, rowIndex) in props.rows"
        :key="getRowKey(row, rowIndex)"
        class="data-table__row"
        role="row"
        :style="{ gridTemplateColumns }"
      >
        <div
          v-for="column in props.columns"
          :key="column.key"
          class="data-table__cell"
          :class="getAlignClass(column)"
          role="cell"
        >
          <slot
            :name="`cell-${column.key}`"
            :row="row"
            :column="column"
            :value="getCellValue(row, column.key)"
            :index="rowIndex"
          >
            {{ getCellValue(row, column.key) }}
          </slot>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
  .data-table {
    min-width: 0;
  }

  .data-table__header,
  .data-table__row {
    display: grid;
    align-items: center;
  }

  .data-table__header {
    min-height: 34px;
    border-bottom: 1px solid var(--border);
    color: var(--text-subtle);
    font-size: var(--font-size-xs);
    font-weight: 700;
  }

  .data-table__row {
    min-height: 54px;
    border-bottom: 1px solid var(--border);
    transition: background 170ms ease;
  }

  .data-table__row:last-child {
    border-bottom: 0;
  }

  .data-table__row:hover {
    background: rgba(255, 255, 255, 0.055);
  }

  .data-table__heading,
  .data-table__cell {
    min-width: 0;
    padding: 0 12px;
  }

  .data-table__heading {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .data-table__cell--center {
    text-align: center;
  }

  .data-table__cell--right {
    text-align: right;
  }

  .data-table--compact .data-table__header {
    min-height: 32px;
  }

  .data-table--compact .data-table__row {
    min-height: 50px;
  }

  .data-table--compact .data-table__heading,
  .data-table--compact .data-table__cell {
    padding-right: 10px;
    padding-left: 10px;
  }
</style>
