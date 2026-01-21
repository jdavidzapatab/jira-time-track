<template>
  <div class="min-h-full flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8 card">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900 dark:text-white">Sign in to your account</h2>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="login">
        <div class="rounded-md shadow-sm space-y-4">
          <div>
            <label for="email-address" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Email address</label>
            <input id="email-address" name="email" type="email" required v-model="email" class="input mt-1" placeholder="Email address" />
          </div>
          <div>
            <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Password</label>
            <input id="password" name="password" type="password" required v-model="password" class="input mt-1" placeholder="Password" />
          </div>
        </div>

        <div>
          <button type="submit" :disabled="loading" class="w-full btn btn-primary">
            {{ loading ? 'Signing in...' : 'Sign in' }}
          </button>
        </div>
        
        <div class="text-center text-sm text-gray-600 dark:text-gray-400">
          Don't have an account? 
          <router-link to="/register" class="font-medium text-blue-600 dark:text-blue-400 hover:text-blue-500 dark:hover:text-blue-300">Register</router-link>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, inject } from 'vue';
import { useRouter } from 'vue-router';

const email = ref('');
const password = ref('');
const loading = ref(false);
const router = useRouter();
const toast = inject('toast');

const login = async () => {
  loading.value = true;
  try {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: email.value,
        password: password.value,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || 'Login failed');
    }

    const data = await response.json();
    localStorage.setItem('token', data.token);
    toast('Logged in successfully');
    router.push('/tickets');
  } catch (err) {
    toast(err.message, 'error');
  } finally {
    loading.value = false;
  }
};
</script>
