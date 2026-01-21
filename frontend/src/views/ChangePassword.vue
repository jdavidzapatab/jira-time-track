<template>
  <div class="min-h-full flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8 card">
      <div v-if="!token">
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

      <div v-else>
        <div>
          <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Set New Password</h2>
          <p class="mt-2 text-center text-sm text-gray-600">
            Enter your new password below.
          </p>
        </div>
        <form class="mt-8 space-y-6" @submit.prevent="executeChange">
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700">New Password</label>
              <input type="password" required v-model="password" class="input mt-1" placeholder="Minimum 8 characters" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700">Confirm New Password</label>
              <input type="password" required v-model="passwordConfirmation" class="input mt-1" placeholder="Confirm password" />
            </div>
          </div>

          <div v-if="success" class="text-green-600 text-sm text-center bg-green-50 p-2 rounded">
            {{ success }}
          </div>

          <div>
            <button type="submit" :disabled="loading" class="w-full btn btn-primary">
              {{ loading ? 'Updating...' : 'Update Password' }}
            </button>
          </div>
          
          <div class="text-center mt-4" v-if="success">
            <router-link to="/login" class="text-blue-600 hover:text-blue-500 font-medium">Go to Login</router-link>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed, inject } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const token = computed(() => route.query.token || '');
const email = ref('');
const password = ref('');
const passwordConfirmation = ref('');
const success = ref('');
const loading = ref(false);
const toast = inject('toast');

const requestChange = async () => {
  loading.value = true;
  success.value = '';
  try {
    const response = await fetch('/api/auth/change-password-request', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: email.value }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || 'Failed to request password change');
    }

    success.value = 'If an account exists for this email, a reset link has been sent to your console.';
    toast(success.value);
  } catch (err) {
    toast(err.message, 'error');
  } finally {
    loading.value = false;
  }
};

const executeChange = async () => {
  if (password.value !== passwordConfirmation.value) {
    toast('Passwords do not match', 'error');
    return;
  }
  
  loading.value = true;
  success.value = '';
  try {
    const response = await fetch('/api/auth/change-password', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        token: token.value,
        password: password.value,
        password_confirmation: passwordConfirmation.value
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || 'Failed to update password');
    }

    success.value = 'Password updated successfully!';
    toast(success.value);
  } catch (err) {
    toast(err.message, 'error');
  } finally {
    loading.value = false;
  }
};
</script>
