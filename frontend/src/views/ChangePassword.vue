<template>
  <div class="min-h-full flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8 card">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Request Password Change</h2>
        <p class="mt-2 text-center text-sm text-gray-600">
          We'll send a link to your email to reset your password.
        </p>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="requestChange">
        <div>
          <label class="block text-sm font-medium text-gray-700">Email address</label>
          <input type="email" required v-model="email" class="input mt-1" placeholder="Email address" />
        </div>

        <div v-if="error" class="text-red-600 text-sm text-center bg-red-50 p-2 rounded">
          {{ error }}
        </div>
        <div v-if="success" class="text-green-600 text-sm text-center bg-green-50 p-2 rounded">
          {{ success }}
        </div>

        <div>
          <button type="submit" :disabled="loading" class="w-full btn btn-primary">
            {{ loading ? 'Sending...' : 'Send Reset Link' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import axios from 'axios';

const email = ref('');
const error = ref('');
const success = ref('');
const loading = ref(false);

const requestChange = async () => {
  loading.value = true;
  error.value = '';
  success.value = '';
  try {
    await axios.post('/api/auth/change-password-request', { email: email.value });
    success.value = 'If an account exists for this email, a reset link has been sent to your console.';
  } catch (err) {
    error.value = err.response?.data || 'Failed to request password change';
  } finally {
    loading.value = false;
  }
};
</script>
