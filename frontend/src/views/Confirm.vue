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
import { ref, onMounted, inject } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const loading = ref(true);
const error = ref('');
const success = ref(false);
const toast = inject('toast');

onMounted(async () => {
  const token = route.query.token;
  if (!token) {
    error.value = 'Invalid or missing confirmation token.';
    loading.value = false;
    return;
  }

  try {
    const response = await fetch('/api/auth/confirm', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || 'Confirmation failed');
    }

    success.value = true;
    toast('Account confirmed successfully!');
  } catch (err) {
    error.value = err.message;
    toast(err.message, 'error');
  } finally {
    loading.value = false;
  }
});
</script>
