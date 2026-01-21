<template>
  <div class="min-h-screen bg-gray-50">
    <nav v-if="isLoggedIn" class="bg-white shadow-sm border-b border-gray-200 sticky top-0 z-40">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
          <div class="flex">
            <!-- Burger Menu Button (Mobile) -->
            <div class="flex items-center sm:hidden mr-4">
              <button @click="mobileMenuOpen = !mobileMenuOpen" class="text-gray-500 hover:text-gray-700 focus:outline-none">
                <Menu v-if="!mobileMenuOpen" :size="24" />
                <X v-else :size="24" />
              </button>
            </div>
            
            <div class="flex-shrink-0 flex items-center font-bold text-xl text-blue-600">
              JiraTrack
            </div>
            <div class="hidden sm:-my-px sm:ml-6 sm:flex sm:space-x-8">
              <router-link to="/servers" class="inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium" :class="[$route.path.startsWith('/servers') ? 'border-blue-500 text-gray-900' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300']">
                Jira Servers
              </router-link>
              <router-link to="/tickets" class="inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium" :class="[$route.path.startsWith('/tickets') ? 'border-blue-500 text-gray-900' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300']">
                Jira Tickets
              </router-link>
            </div>
          </div>
          <div class="hidden sm:ml-6 sm:flex sm:items-center space-x-4">
            <router-link to="/change-password" class="text-gray-500 hover:text-gray-700 text-sm font-medium">Change Password</router-link>
            <button @click="logout" class="text-gray-500 hover:text-gray-700 text-sm font-medium">Logout</button>
          </div>
        </div>
      </div>
      
      <!-- Mobile Menu -->
      <div v-if="mobileMenuOpen" class="sm:hidden bg-white border-b border-gray-200 pb-3 pt-2">
        <div class="space-y-1">
          <router-link to="/servers" @click="mobileMenuOpen = false" class="block pl-3 pr-4 py-2 border-l-4 text-base font-medium" :class="[$route.path.startsWith('/servers') ? 'bg-blue-50 border-blue-500 text-blue-700' : 'border-transparent text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800']">
            Jira Servers
          </router-link>
          <router-link to="/tickets" @click="mobileMenuOpen = false" class="block pl-3 pr-4 py-2 border-l-4 text-base font-medium" :class="[$route.path.startsWith('/tickets') ? 'bg-blue-50 border-blue-500 text-blue-700' : 'border-transparent text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800']">
            Jira Tickets
          </router-link>
          <router-link to="/change-password" @click="mobileMenuOpen = false" class="block pl-3 pr-4 py-2 border-l-4 border-transparent text-base font-medium text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800">
            Change Password
          </router-link>
          <button @click="logoutAndClose" class="block w-full text-left pl-3 pr-4 py-2 border-l-4 border-transparent text-base font-medium text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800">
            Logout
          </button>
        </div>
      </div>
    </nav>
    <main class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
      <router-view></router-view>
    </main>

    <!-- Global Toast Notifications -->
    <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm w-full">
      <TransitionGroup name="toast">
        <div v-for="toast in toasts" :key="toast.id" 
             class="p-4 rounded-lg shadow-lg border flex items-start gap-3 transition-all duration-300"
             :class="{
               'bg-green-50 border-green-200 text-green-800': toast.type === 'success',
               'bg-red-50 border-red-200 text-red-800': toast.type === 'error',
               'bg-yellow-50 border-yellow-200 text-yellow-800': toast.type === 'warning'
             }">
          <div class="flex-shrink-0 mt-0.5">
            <CheckCircle v-if="toast.type === 'success'" :size="18" />
            <AlertCircle v-else-if="toast.type === 'error'" :size="18" />
            <AlertTriangle v-else :size="18" />
          </div>
          <div class="flex-1 text-sm font-medium">
            {{ toast.message }}
          </div>
          <button @click="removeToast(toast.id)" class="flex-shrink-0 text-gray-400 hover:text-gray-600 transition-colors">
            <X :size="16" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, provide } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { Menu, X, CheckCircle, AlertCircle, AlertTriangle } from 'lucide-vue-next';

const router = useRouter();
const route = useRoute();
const isLoggedIn = ref(!!localStorage.getItem('token'));
const mobileMenuOpen = ref(false);

const toasts = ref([]);
let toastId = 0;

const addToast = (message, type = 'success') => {
  const id = toastId++;
  toasts.value.push({ id, message, type });
  const duration = type === 'success' ? 10000 : 30000;
  setTimeout(() => removeToast(id), duration);
};

const removeToast = (id) => {
  toasts.value = toasts.value.filter(t => t.id !== id);
};

provide('toast', addToast);

watch(() => route.path, () => {
  isLoggedIn.value = !!localStorage.getItem('token');
});

const logout = () => {
  localStorage.removeItem('token');
  isLoggedIn.value = false;
  router.push('/login');
};

const logoutAndClose = () => {
  mobileMenuOpen.value = false;
  logout();
};
</script>

<style>
/* Global styles are in style.css */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(30px);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
