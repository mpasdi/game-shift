<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import { ImageOff, Images, Search } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import BaseModal from '../../../shared/components/BaseModal.vue'
  import BaseSelect from '../../../shared/components/BaseSelect.vue'
  import TextField from '../../../shared/components/TextField.vue'
  import { listCoverCandidates, searchCoverCandidates } from '../api'
  import type { CoverCandidate, GameMatch } from '../types/cover'

  const props = defineProps<{
    open: boolean
    initialQuery: string
  }>()

  const emit = defineEmits<{
    close: []
    select: [candidate: CoverCandidate]
  }>()

  const query = ref('')
  const matchedGame = ref<GameMatch | null>(null)
  const gameResults = ref<GameMatch[]>([])
  const candidates = ref<CoverCandidate[]>([])
  const selectedCandidate = ref<CoverCandidate | null>(null)
  const isSearching = ref(false)
  const isSwitchingGame = ref(false)
  const hasSearched = ref(false)
  const errorMessage = ref<string | null>(null)
  const candidateScroller = ref<HTMLElement | null>(null)
  let coverRequestId = 0

  const gameOptions = computed(() => gameResults.value)
  const gameSelectOptions = computed(() =>
    gameOptions.value.map((game) => ({
      value: game.providerGameId,
      label: game.name,
      description: game.releaseYear ? String(game.releaseYear) : undefined
    }))
  )
  const selectedGameId = computed({
    get: () => matchedGame.value?.providerGameId ?? '',
    set: (providerGameId: string) => {
      const game = gameOptions.value.find((option) => option.providerGameId === providerGameId)
      if (game) void switchGame(game)
    }
  })

  watch(
    () => props.open,
    (open) => {
      invalidateCoverRequests()
      if (!open) return
      query.value = props.initialQuery.trim()
      matchedGame.value = null
      gameResults.value = []
      candidates.value = []
      selectedCandidate.value = null
      hasSearched.value = false
      errorMessage.value = null
      if (query.value) void search()
    }
  )

  watch(
    query,
    (value) => {
      invalidateCoverRequests()
      if (
        hasSearched.value ||
        matchedGame.value ||
        gameResults.value.length ||
        candidates.value.length ||
        selectedCandidate.value
      ) {
        clearSearchResults()
      }
      if (!value.trim()) {
        errorMessage.value = null
      } else if (errorMessage.value) {
        errorMessage.value = null
      }
    },
    { flush: 'sync' }
  )

  async function search() {
    const normalizedQuery = query.value.trim()
    if (!normalizedQuery) {
      clearSearchResults()
      errorMessage.value = '请输入游戏名称后再搜索'
      return
    }

    const requestId = ++coverRequestId
    resetCandidateScroll()
    isSearching.value = true
    isSwitchingGame.value = false
    errorMessage.value = null
    clearSearchResults()
    try {
      const result = await searchCoverCandidates(normalizedQuery)
      if (requestId !== coverRequestId) return
      matchedGame.value = result.matchedGame ?? null
      gameResults.value = result.matchedGame
        ? [result.matchedGame, ...result.alternativeGames]
        : result.alternativeGames
      candidates.value = result.candidates
      hasSearched.value = true
    } catch (error) {
      if (requestId !== coverRequestId) return
      errorMessage.value = getErrorMessage(error)
    } finally {
      if (requestId === coverRequestId) isSearching.value = false
    }
  }

  async function switchGame(game: GameMatch) {
    if (matchedGame.value?.providerGameId === game.providerGameId) return

    const requestId = ++coverRequestId
    isSearching.value = false
    isSwitchingGame.value = true
    resetCandidateScroll()
    errorMessage.value = null
    selectedCandidate.value = null
    try {
      const nextCandidates = await listCoverCandidates(game.provider, game.providerGameId)
      if (requestId !== coverRequestId) return
      candidates.value = nextCandidates
      matchedGame.value = game
    } catch (error) {
      if (requestId !== coverRequestId) return
      errorMessage.value = getErrorMessage(error)
    } finally {
      if (requestId === coverRequestId) isSwitchingGame.value = false
    }
  }

  function chooseCandidate(candidate: CoverCandidate) {
    selectedCandidate.value = candidate
  }

  function confirmSelection() {
    if (!selectedCandidate.value) return
    emit('select', selectedCandidate.value)
  }

  function clearSearchResults() {
    matchedGame.value = null
    gameResults.value = []
    candidates.value = []
    selectedCandidate.value = null
    hasSearched.value = false
  }

  function invalidateCoverRequests() {
    coverRequestId += 1
    isSearching.value = false
    isSwitchingGame.value = false
  }

  function resetCandidateScroll() {
    requestAnimationFrame(() => candidateScroller.value?.scrollTo({ top: 0 }))
  }

  function getErrorMessage(error: unknown) {
    if (typeof error === 'string') return error
    if (error instanceof Error) return error.message
    return '联网封面搜索失败，请稍后重试'
  }
</script>

<template>
  <BaseModal :open="props.open" title="联网搜索封面" size="lg" :body-scrollable="false" @close="emit('close')">
    <div class="online-cover-dialog">
      <div class="cover-toolbar">
        <form class="cover-search" @submit.prevent="search">
          <TextField id="online-cover-query" v-model="query" type="search" placeholder="输入其他名称重新搜索">
            <template #icon><Search :size="16" /></template>
          </TextField>
          <BaseButton variant="primary" type="submit" :loading="isSearching" :disabled="isSwitchingGame">
            <template #icon><Search :size="16" /></template>
            搜索
          </BaseButton>
        </form>

        <p class="cover-search__notice" :class="{ 'cover-search__notice--error': errorMessage }">
          {{ errorMessage || '仅发送搜索词，不会上传本地游戏路径。' }}
        </p>

        <div v-if="hasSearched && gameOptions.length" class="game-matches">
          <div class="section-heading">
            <span>匹配游戏</span>
            <small>选择正确的游戏条目</small>
          </div>
          <BaseSelect
            v-model="selectedGameId"
            :options="gameSelectOptions"
            accessible-label="选择匹配游戏"
            placeholder="没有匹配游戏"
            :loading="isSwitchingGame"
          />
        </div>
      </div>

      <div ref="candidateScroller" class="cover-results">
        <div v-if="isSearching" class="candidate-section">
          <div class="section-heading">
            <span>正在搜索封面</span>
            <small>{{ query.trim() }}</small>
          </div>
          <div class="candidate-grid" aria-label="正在加载封面候选">
            <span v-for="index in 8" :key="index" class="cover-skeleton" />
          </div>
        </div>

        <template v-else-if="hasSearched">
          <div class="candidate-section">
            <div class="section-heading">
              <span>封面候选</span>
              <small v-if="matchedGame">{{ matchedGame.name }}</small>
            </div>

            <div v-if="isSwitchingGame" class="candidate-state">正在加载这个游戏的封面...</div>
            <div v-else-if="candidates.length" class="candidate-grid">
              <button
                v-for="candidate in candidates"
                :key="`${candidate.provider}:${candidate.assetId}`"
                class="cover-candidate"
                :class="{ 'cover-candidate--selected': selectedCandidate?.assetId === candidate.assetId }"
                type="button"
                :aria-label="selectedCandidate?.assetId === candidate.assetId ? '已选择此封面' : '选择此封面'"
                @click="chooseCandidate(candidate)"
              >
                <img :src="candidate.previewUrl" alt="" loading="lazy" />
                <span class="cover-candidate__check">✓</span>
              </button>
            </div>
            <div v-else class="candidate-state">
              <ImageOff :size="24" />
              <span>{{ matchedGame ? '这个游戏暂无可用的竖版封面' : '没有找到匹配的游戏' }}</span>
            </div>
          </div>
        </template>

        <div v-else class="candidate-state candidate-state--welcome">
          <Images :size="28" />
          <strong>输入游戏名称开始搜索</strong>
          <span>找到封面后，可以在这里预览并选择。</span>
        </div>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="secondary" type="button" @click="emit('close')">取消</BaseButton>
      <BaseButton variant="primary" type="button" :disabled="!selectedCandidate" @click="confirmSelection">
        使用此封面
      </BaseButton>
    </template>
  </BaseModal>
</template>

<style scoped>
  .online-cover-dialog {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: min(620px, calc(100vh - 190px));
    gap: 14px;
  }

  .cover-toolbar {
    display: grid;
    gap: 10px;
    min-width: 0;
    border-bottom: 1px solid var(--border);
    padding-bottom: 14px;
  }

  .cover-results {
    min-height: 0;
    overflow-y: auto;
    padding-right: 8px;
    overscroll-behavior: contain;
  }

  .cover-search {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px;
  }

  .cover-search :deep(.text-field) {
    width: 100%;
  }

  .cover-search__notice {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--text-subtle);
  }

  .cover-search__notice--error {
    color: #fecdd3;
  }

  .game-matches,
  .candidate-section {
    display: grid;
    gap: 10px;
  }

  .section-heading {
    display: flex;
    gap: 12px;
    align-items: baseline;
    justify-content: space-between;
    color: var(--text);
    font-weight: 700;
  }

  .section-heading small {
    overflow: hidden;
    color: var(--text-subtle);
    font-size: var(--font-size-sm);
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .candidate-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(112px, 1fr));
    gap: 12px;
  }

  .cover-candidate {
    position: relative;
    overflow: hidden;
    aspect-ratio: 2 / 3;
    border: 2px solid transparent;
    border-radius: 8px;
    padding: 0;
    background: var(--surface);
    transition: border-color 160ms ease;
  }

  .cover-skeleton {
    position: relative;
    overflow: hidden;
    aspect-ratio: 2 / 3;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
  }

  .cover-skeleton::after {
    position: absolute;
    inset: 0;
    background: linear-gradient(110deg, transparent 18%, rgba(255, 255, 255, 0.08) 42%, transparent 66%);
    content: '';
    transform: translateX(-100%);
    animation: skeleton-shimmer 1.25s ease-in-out infinite;
  }

  .cover-candidate:hover:not(.cover-candidate--selected) {
    border-color: var(--border-strong);
  }

  .cover-candidate--selected {
    border-color: var(--accent-strong);
  }

  .cover-candidate img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-candidate__check {
    position: absolute;
    right: 8px;
    bottom: 8px;
    display: grid;
    width: 24px;
    height: 24px;
    border-radius: 999px;
    background: var(--accent);
    color: #fff;
    opacity: 0;
    place-items: center;
    font-weight: 800;
    transform: scale(0.8);
    transition:
      opacity 160ms ease,
      transform 160ms ease;
  }

  .cover-candidate--selected .cover-candidate__check {
    opacity: 1;
    transform: scale(1);
  }

  .candidate-state {
    display: grid;
    min-height: 180px;
    place-items: center;
    align-content: center;
    gap: 10px;
    border: 1px dashed var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.02);
    color: var(--text-subtle);
  }

  .candidate-state--welcome strong {
    color: var(--text-muted);
    font-size: var(--font-size-md);
  }

  .candidate-state--welcome span {
    font-size: var(--font-size-sm);
  }

  .cover-results > .candidate-state--welcome {
    min-height: 100%;
  }

  @keyframes skeleton-shimmer {
    to {
      transform: translateX(100%);
    }
  }

  @media (max-width: 720px) {
    .cover-search {
      grid-template-columns: 1fr;
    }

    .candidate-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
