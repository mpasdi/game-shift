import { describe, expect, it, afterEach } from 'vitest'
import { mount, enableAutoUnmount, flushPromises } from '@vue/test-utils'
import { nextTick } from 'vue'
import ScanResultsDialog from './ScanResultsDialog.vue'
// type
import type { ScanCandidate } from '../types/game.ts'

enableAutoUnmount(afterEach)

const candidates: ScanCandidate[] = [
  {
    name: 'games',
    exePath: 'D:\\Games\\test.exe',
    folderPath: 'D:\\Games',
    exeFileName: 'Games/test.exe',
    exists: false,
    recommended: true,
    confidence: 1,
    reasons: ['test.exe']
  },
  {
    name: 'games',
    exePath: 'D:\\Games\\test1.exe',
    folderPath: 'D:\\Games',
    exeFileName: 'Games/test1.exe',
    exists: false,
    recommended: false,
    confidence: 1,
    reasons: ['test1.exe']
  },
  {
    name: 'games',
    exePath: 'D:\\Games\\test2.exe',
    folderPath: 'D:\\Games',
    exeFileName: 'Games/test2.exe',
    exists: true,
    recommended: true,
    confidence: 1,
    reasons: ['test2.exe']
  }
]

describe('ScanResults Dialog', () => {
  it('renders scan results and summary counts', async () => {
    mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: candidates
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    const scanSummary = document.body.querySelectorAll('.scan-summary-item')
    expect(scanSummary.length).toBe(4)
    expect(scanSummary[0].textContent).include('1')
    expect(scanSummary[1].textContent).include('1')
    expect(scanSummary[2].textContent).include('1')
    expect(scanSummary[3].textContent).toContain('1')
  })

  it('emits import when import button is clicked', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: candidates
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toEqual([[[candidates[0]]]])
  })

  it('existing games cannot be selected', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: candidates
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 已经存在的游戏不可选中
    const expandButton = document.querySelector<HTMLButtonElement>('[aria-label="展开已存在"]')
    expect(expandButton).not.toBeNull()
    expandButton!.click()
    await nextTick()
    const existingRow = document.body.querySelector('.scan-row--disabled')
    const existingCheckButton = existingRow?.querySelector<HTMLButtonElement>('.check-button')
    expect(existingCheckButton).not.toBeNull()
    expect(existingCheckButton!.disabled).toBe(true)

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toEqual([[[candidates[0]]]])
  })

  it('imports other exes when other exe is selected', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: candidates
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 选择其他游戏
    const expandButton = document.querySelector<HTMLButtonElement>('[aria-label="展开其他可执行文件"]')
    expect(expandButton).not.toBeNull()
    expandButton!.click()
    await nextTick()

    const selectedRow = Array.from(document.body.querySelectorAll('.scan-row')).find((row) =>
      row.textContent?.includes(candidates[1].exeFileName)
    )
    expect(selectedRow).toBeDefined()
    const selectedCheckButton = selectedRow!.querySelector<HTMLButtonElement>('.check-button')
    expect(selectedCheckButton).not.toBeNull()
    selectedCheckButton!.click()
    await nextTick()

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toEqual([[[candidates[0], candidates[1]]]])
  })

  it('trims game name when importing', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: [
          {
            ...candidates[0],
            name: '    games   '
          }
        ]
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toEqual([[[candidates[0]]]])
  })

  it('uses exeFileName when name is empty', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: [
          {
            ...candidates[0],
            name: ''
          }
        ]
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toEqual([
      [
        [
          {
            ...candidates[0],
            name: 'Games/test'
          }
        ]
      ]
    ])
  })

  it('disabled import when no game is selected', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: [candidates[0]]
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 取消选择
    const selectedRow = Array.from(document.body.querySelectorAll('.scan-row')).find((row) =>
      row.textContent?.includes(candidates[0].exeFileName)
    )
    expect(selectedRow).toBeDefined()
    const selectedCheckButton = selectedRow!.querySelector<HTMLButtonElement>('.check-button')
    expect(selectedCheckButton).not.toBeNull()
    selectedCheckButton!.click()
    await nextTick()

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    expect(importButton?.disabled).toBe(true)
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toBeUndefined()
  })

  it('emits close when close button is clicked', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: [candidates[0]]
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 点击取消按钮
    const closeButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '取消'
    )
    expect(closeButton).toBeDefined()
    closeButton!.click()
    await flushPromises()
    expect(wrapper.emitted('close')).toEqual([[]])
  })

  it('prevents actions while importing', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        importing: true,
        candidates: [candidates[0]]
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 点击取消按钮
    const allButtons = Array.from(document.body.querySelectorAll('button'))

    const importButton = allButtons.find((button) => button.textContent?.trim() === '导入')
    const closeButton = allButtons.find((button) => button.textContent?.trim() === '取消')
    expect(importButton).not.toBeUndefined()
    expect(closeButton).not.toBeUndefined()
    expect(importButton?.disabled).toBe(true)
    expect(closeButton?.disabled).toBe(true)

    importButton?.click()
    expect(wrapper.emitted('import')).toBeUndefined()

    closeButton?.click()
    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('selects all games in a section', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: [{ ...candidates[0], recommended: false }, candidates[1]]
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 点击展开和全选
    const expandButton = document.body.querySelector<HTMLButtonElement>('[aria-label="展开其他可执行文件"]')
    const selectAllButton = document.body.querySelector<HTMLButtonElement>('[aria-label="全选其他可执行文件"]')
    expect(expandButton).not.toBeNull()
    expect(selectAllButton).not.toBeNull()
    expandButton!.click()
    selectAllButton!.click()
    await nextTick()

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toEqual([[[{ ...candidates[0], recommended: false }, candidates[1]]]])
  })

  it('cancels selecting all games in a section', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: [{ ...candidates[0], recommended: false }, candidates[1]]
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()

    // 点击展开和全选
    const expandButton = document.body.querySelector<HTMLButtonElement>('[aria-label="展开其他可执行文件"]')
    const selectAllButton = document.body.querySelector<HTMLButtonElement>('[aria-label="全选其他可执行文件"]')
    expect(expandButton).not.toBeNull()
    expect(selectAllButton).not.toBeNull()
    expandButton!.click()
    selectAllButton!.click()
    await nextTick()

    const cancelSelectAllButton =
      document.body.querySelector<HTMLButtonElement>('[aria-label="取消全选其他可执行文件"]')
    expect(cancelSelectAllButton).not.toBeNull()
    cancelSelectAllButton!.click()
    await nextTick()

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    expect(importButton?.disabled).toBe(true)
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toBeUndefined()
  })

  it('prevents emit when candidates is empty', async () => {
    const wrapper = mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        candidates: []
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()
    expect(scanDialog?.textContent).toContain('没有扫描到可导入的 .exe 文件')

    // 点击导入按钮
    const importButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '导入'
    )
    expect(importButton).toBeDefined()
    expect(importButton?.disabled).toBe(true)
    importButton!.click()
    await flushPromises()
    expect(wrapper.emitted('import')).toBeUndefined()
  })

  it('shows error message when scan error', async () => {
    mount(ScanResultsDialog, {
      attachTo: document.body,
      global: {
        stubs: {
          BaseTooltip: true
        }
      },
      props: {
        open: true,
        errorMessage: 'has error',
        candidates: []
      }
    })
    await nextTick()
    const scanDialog = document.querySelector('[role="dialog"]')
    expect(scanDialog).not.toBeNull()
    expect(scanDialog?.textContent).toContain('has error')
  })
})
