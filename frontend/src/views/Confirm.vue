<template>
  <div class="min-h-full flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8 card text-center">
      <h2 class="text-2xl font-bold text-gray-900">Confirming your account...</h2>
      <div v-if="loading" class="animate-pulse flex space-x-4 justify-center">
        <div class="rounded-full bg-gray-200 h-10 w-10"></div>
      </div>
      <div v-if="error" class="text-red-600 bg-red-50 p-4 rounded">
        {{ error }}
        <div class="mt-4">
          <router-link to="/login" class="btn btn-secondary">Go to Login</router-link>
        </div>
      </div>
      <div v-if="success" class="text-green-600 bg-green-50 p-4 rounded">
        Account confirmed successfully!
        <div class="mt-4">
          <router-link to="/login" class="btn btn-primary">Sign in now</router-link>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import axios from 'axios';

const route = useRoute();
const loading = ref(true);
const error = ref('');
const success = ref(false);

onMounted(async () => {
  const token = route.query.token;
  if (!token) {
    error.value = 'Invalid or missing confirmation token.';
    loading.value = false;
    return;
  }

  try {
    await axios.post('/api/auth/confirm', { token });
    success.value = true;
  } catch (err) {
    error.value = err.response?.data || 'Confirmation failed';
  } finally {
    loading.value = false;
  }
});
</script>
