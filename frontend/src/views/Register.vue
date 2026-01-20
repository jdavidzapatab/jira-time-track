<template>
  <div class="auth-page">
    <h2>Register</h2>
    <form @submit.prevent="handleRegister">
      <div>
        <label>Email:</label>
        <input v-model="email" type="email" required />
      </div>
      <div>
        <label>Password:</label>
        <input v-model="password" type="password" required />
      </div>
      <div>
        <label>Confirm Password:</label>
        <input v-model="passwordConfirmation" type="password" required />
      </div>
      <button type="submit">Register</button>
    </form>
    <p>Already have an account? <router-link to="/login">Login</router-link></p>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import axios from 'axios';
import { useRouter } from 'vue-router';

const email = ref('');
const password = ref('');
const passwordConfirmation = ref('');
const router = useRouter();

const handleRegister = async () => {
  try {
    await axios.post('/api/auth/register', {
      email: email.value,
      password: password.value,
      password_confirmation: passwordConfirmation.value,
    });
    alert('Registration successful! Please check your email for confirmation.');
    router.push('/login');
  } catch (e) {
    alert(e.response?.data || 'Registration failed');
  }
};
</script>
