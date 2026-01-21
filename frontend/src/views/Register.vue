<template>
  <div class="min-h-full flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8 card">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Create your account</h2>
      </div>
      <form class="mt-8 space-y-6" @submit.prevent="register">
        <div class="rounded-md shadow-sm space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700">Email address</label>
            <input type="email" required v-model="email" class="input mt-1" placeholder="Email address" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Password</label>
            <input type="password" required v-model="password" class="input mt-1" placeholder="Password (min 8 chars)" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Confirm Password</label>
            <input type="password" required v-model="passwordConfirmation" class="input mt-1" placeholder="Confirm Password" />
          </div>
        </div>

        <div>
          <button type="submit" :disabled="loading" class="w-full btn btn-primary">
            {{ loading ? 'Registering...' : 'Register' }}
          </button>
        </div>

        <div class="text-center text-sm text-gray-600">
          Already have an account? 
          <router-link to="/login" class="font-medium text-blue-600 hover:text-blue-500">Sign in</router-link>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, inject } from 'vue';

const email = ref('');
const password = ref('');
const passwordConfirmation = ref('');
const loading = ref(false);
const toast = inject('toast');

const register = async () => {
  loading.value = true;
  try {
    const response = await fetch('/api/auth/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: email.value,
        password: password.value,
        password_confirmation: passwordConfirmation.value,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || 'Registration failed');
    }

    toast('Registration successful! Please check your console (email) for confirmation link.');
  } catch (err) {
    toast(err.message, 'error');
  } finally {
    loading.value = false;
  }
};
</script>
