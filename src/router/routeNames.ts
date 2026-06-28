export const routeNames = {
  home: 'home',
  games: 'games',
  favorites: 'favorites',
  recent: 'recent',
  categories: 'categories',
  settings: 'settings'
} as const

export type RouteName = (typeof routeNames)[keyof typeof routeNames]
