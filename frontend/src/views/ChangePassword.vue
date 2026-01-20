<template>
  <div class="auth-page">
    <h2>Change Password</h2>
    <p>Please enter your email to receive a password change link.</p>
    <form @submit.prevent="handleChangePassword">
      <div>
        <label>Email:</label>
        <input v-model="email" type="email" required />
      </div>
      <button type="submit">Send Link</button>
    </form>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import axios from 'axios';

const email = ref('');

const handleChangePassword = async () => {
  try {
    // According to the requirements: "sending a link to the registered email to proceed with the password change action. 
    // The action to be able to see the link in the email inbox is enough to prove ownership for now."
    // I will mock this for now as per instructions.
    await axios.post('/api/auth/change-password-request', { email: email.value });
    alert('Password change link sent! (Check server logs for the link)');
  } catch (e) {
    alert(e.response?.data || 'Request failed');
  }
};
</script>
