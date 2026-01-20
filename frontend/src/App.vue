<template>
  <div class="container">
    <nav v-if="isLoggedIn">
      <router-link to="/servers">Jira Servers</router-link> |
      <router-link to="/tickets">Jira Tickets</router-link> |
      <router-link to="/change-password">Change Password</router-link> |
      <a @click="logout" href="#">Logout</a>
    </nav>
    <router-view></router-view>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
const isLoggedIn = computed(() => !!localStorage.getItem('token'));

const logout = () => {
  localStorage.removeItem('token');
  router.push('/login');
};
</script>

<style>
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  font-family: sans-serif;
}
nav {
  margin-bottom: 20px;
}
button {
  cursor: pointer;
}
</style>
