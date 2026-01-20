<template>
  <div>
    <h2>Confirming your email...</h2>
  </div>
</template>

<script setup>
import { onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import axios from 'axios';

const route = useRoute();
const router = useRouter();

onMounted(async () => {
  const token = route.query.token;
  if (token) {
    try {
      await axios.post('/api/auth/confirm', { token });
      alert('Email confirmed successfully! You can now login.');
      router.push('/login');
    } catch (e) {
      alert('Confirmation failed: ' + (e.response?.data || 'Invalid token'));
      router.push('/login');
    }
  }
});
</script>
