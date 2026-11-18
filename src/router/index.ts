import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'splash',
    component: () => import('@/pages/SplashPage.vue'),
  },
  {
    path: '/hub',
    name: 'hub',
    component: () => import('@/pages/HubPage.vue'),
  },
  {
    path: '/ide',
    name: 'ide',
    component: () => import('@/pages/IdePage.vue'),
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})