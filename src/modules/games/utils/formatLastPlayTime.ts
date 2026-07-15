const DAY_IN_MILLISECONDS = 86_400_000

function pad(value: number) {
  return String(value).padStart(2, '0')
}

function formatClock(date: Date) {
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`
}

export function formatLastPlayTime(timestamp?: number | null, nowTimestamp = Date.now()) {
  if (!timestamp) return '无启动记录'

  const playedAt = new Date(timestamp)
  const now = new Date(nowTimestamp)
  const today = Date.UTC(now.getFullYear(), now.getMonth(), now.getDate())
  const playedDay = Date.UTC(playedAt.getFullYear(), playedAt.getMonth(), playedAt.getDate())
  const dayDifference = Math.floor((today - playedDay) / DAY_IN_MILLISECONDS)
  const clock = formatClock(playedAt)

  if (dayDifference === 0) return `今天 ${clock}`
  if (dayDifference === 1) return `昨天 ${clock}`

  return `${playedAt.getFullYear()}/${pad(playedAt.getMonth() + 1)}/${pad(playedAt.getDate())} ${clock}`
}
