import type { RouteRecordRaw } from 'vue-router'
import GameLibraryLayout from '../app/GameLibraryLayout.vue'
import { routeNames } from './routeNames'

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: GameLibraryLayout,
    children: [
      { path: '', redirect: { name: routeNames.home } },
      { path: 'home', name: routeNames.home, component: () => import('../pages/HomePage.vue') },
      { path: 'games', name: routeNames.games, component: () => import('../pages/AllGamesPage.vue') },
      { path: 'favorites', name: routeNames.favorites, component: () => import('../pages/FavoriteGamesPage.vue') },
      { path: 'recent', name: routeNames.recent, component: () => import('../pages/RecentGamesPage.vue') },
      { path: 'categories', name: routeNames.categories, component: () => import('../pages/CategoriesPage.vue') },
      { path: 'settings', name: routeNames.settings, component: () => import('../pages/SettingsPage.vue') }
    ]
  }
]
