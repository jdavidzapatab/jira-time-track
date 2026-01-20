import { createRouter, createWebHistory } from 'vue-router'
import Login from '../views/Login.vue'
import Register from '../views/Register.vue'
import Confirm from '../views/Confirm.vue'
import JiraServers from '../views/JiraServers.vue'
import JiraTickets from '../views/JiraTickets.vue'
import ChangePassword from '../views/ChangePassword.vue'

const routes = [
  { path: '/', redirect: '/tickets' },
  { path: '/login', component: Login },
  { path: '/register', component: Register },
  { path: '/confirm', component: Confirm },
  { path: '/change-password', component: ChangePassword },
  { path: '/servers', component: JiraServers, meta: { requiresAuth: true } },
  { path: '/tickets', component: JiraTickets, meta: { requiresAuth: true } },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to, from, next) => {
  const loggedIn = !!localStorage.getItem('token')
  if (to.matched.some(record => record.meta.requiresAuth) && !loggedIn) {
    next('/login')
  } else {
    next()
  }
})

export default router
