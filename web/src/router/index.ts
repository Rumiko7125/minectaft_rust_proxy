import { createRouter, createWebHistory } from 'vue-router'
import { useAppStore } from '../store/app'
import { authApi } from '../api'

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('../pages/Login.vue')
  },
  {
    path: '/setup',
    name: 'Setup',
    component: () => import('../pages/Setup.vue')
  },
  {
    path: '/',
    component: () => import('../layouts/AppLayout.vue'),
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        redirect: '/dashboard'
      },
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('../pages/Dashboard.vue')
      },
      {
        path: 'players',
        name: 'Players',
        component: () => import('../pages/Players.vue')
      },
      {
        path: 'access',
        name: 'Access',
        component: () => import('../pages/Access.vue')
      },
      {
        path: 'routes',
        name: 'Routes',
        component: () => import('../pages/Routes.vue')
      },
      {
        path: 'backend',
        name: 'Backend',
        component: () => import('../pages/Backend.vue')
      },
      {
        path: 'twofactor',
        name: 'TwoFactor',
        component: () => import('../pages/TwoFactor.vue')
      },
      {
        path: 'logs',
        name: 'Logs',
        component: () => import('../pages/Logs.vue')
      },
      {
        path: 'config',
        name: 'Config',
        component: () => import('../pages/Config.vue')
      },
      {
        path: 'account',
        name: 'Account',
        component: () => import('../pages/Account.vue')
      }
    ]
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

router.beforeEach(async (to, from, next) => {
  const appStore = useAppStore()

  // If accessing root path, check if initialization is needed
  if (to.path === '/' || to.path === '') {
    try {
      const status = await authApi.getStatus()
      if (status.needs_setup) {
        next('/setup')
        return
      }
    } catch (e) {
      // Ignore error, continue normal flow
    }
  }

  if (to.meta.requiresAuth && !appStore.isLoggedIn) {
    next('/login')
  } else {
    next()
  }
})

export default router
