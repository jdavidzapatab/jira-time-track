<template>
  <div class="min-h-full flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8 card">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Sign in to your account</h2>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="login">
        <div class="rounded-md shadow-sm space-y-4">
          <div>
            <label for="email-address" class="block text-sm font-medium text-gray-700">Email address</label>
            <input id="email-address" name="email" type="email" required v-model="email" class="input mt-1" placeholder="Email address" />
          </div>
          <div>
            <label for="password" class="block text-sm font-medium text-gray-700">Password</label>
            <input id="password" name="password" type="password" required v-model="password" class="input mt-1" placeholder="Password" />
          </div>
        </div>

        <div v-if="error" class="text-red-600 text-sm text-center bg-red-50 p-2 rounded">
          {{ error }}
        </div>

        <div>
          <button type="submit" :disabled="loading" class="w-full btn btn-primary">
            {{ loading ? 'Signing in...' : 'Sign in' }}
          </button>
        </div>
        
        <div class="text-center text-sm text-gray-600">
          Don't have an account? 
          <router-link to="/register" class="font-medium text-blue-600 hover:text-blue-500">Register</router-link>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import axios from 'axios';
import { useRouter } from 'vue-router';

const email = ref('');
const password = ref('');
const error = ref('');
const loading = ref(false);
const router = useRouter();

const login = async () => {
  loading.value = true;
  error.value = '';
  try {
    const response = await axios.post('/api/auth/login', {
      email: email.value,
      password: password.value,
    });
    localStorage.setItem('token', response.data.token);
    router.push('/tickets');
  } catch (err) {
    error.value = err.response?.data || 'Login failed';
  } finally {
    loading.value = false;
  }
};
</script>
