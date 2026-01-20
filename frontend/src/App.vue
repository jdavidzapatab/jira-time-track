<template>
  <div class="min-h-screen bg-gray-50">
    <nav v-if="isLoggedIn" class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
          <div class="flex">
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
    </nav>
    <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
      <router-view></router-view>
    </main>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';

const router = useRouter();
const route = useRoute();
const isLoggedIn = computed(() => !!localStorage.getItem('token'));

const logout = () => {
  localStorage.removeItem('token');
  router.push('/login');
};
</script>

<style>
/* Global styles are in style.css */
</style>
