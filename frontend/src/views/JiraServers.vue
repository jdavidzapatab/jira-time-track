<template>
  <div>
    <h2>Jira Servers</h2>
    <form @submit.prevent="addServer">
      <input v-model="newServer.name" placeholder="Name" required title="Friendly name for the server" />
      <input v-model="newServer.url" placeholder="URL" required title="Jira Server URL (e.g. https://your-domain.atlassian.net)" />
      <input v-model="newServer.username" placeholder="Username" required title="Your Jira username or email" />
      <div class="password-field">
        <input :type="showPassword ? 'text' : 'password'" v-model="newServer.password" placeholder="Password" required title="Your Jira password or API token" />
        <button type="button" @click="showPassword = !showPassword" title="Toggle password visibility">
          <Eye v-if="!showPassword" :size="16" />
          <EyeOff v-else :size="16" />
        </button>
      </div>
      <button type="submit" title="Add new Jira server">Add Server</button>
    </form>

    <table v-if="servers.length">
      <thead>
        <tr>
          <th>Name</th>
          <th>URL</th>
          <th>Username</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="server in servers" :key="server.id">
          <td>{{ server.name }}</td>
          <td>{{ server.url }}</td>
          <td>{{ server.username }}</td>
          <td>
            <button @click="testCredentials(server.id)" title="Test connection to this server"><RefreshCcw :size="16" /></button>
            <button @click="confirmDelete(server)" title="Delete this server"><Trash2 :size="16" /></button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import axios from 'axios';
import { Eye, EyeOff, RefreshCcw, Trash2 } from 'lucide-vue-next';

const servers = ref([]);
const newServer = ref({ name: '', url: '', username: '', password: '' });
const showPassword = ref(false);

const fetchServers = async () => {
  const token = localStorage.getItem('token');
  const response = await axios.get('/api/servers', {
    headers: { Authorization: `Bearer ${token}` }
  });
  servers.value = response.data;
};

const addServer = async () => {
  try {
    const token = localStorage.getItem('token');
    await axios.post('/api/servers', newServer.value, {
      headers: { Authorization: `Bearer ${token}` }
    });
    newServer.value = { name: '', url: '', username: '', password: '' };
    fetchServers();
  } catch (e) {
    alert(e.response?.data || 'Failed to add server');
  }
};

const testCredentials = async (id) => {
  try {
    const token = localStorage.getItem('token');
    await axios.post(`/api/servers/${id}/test`, {}, {
      headers: { Authorization: `Bearer ${token}` }
    });
    alert('Connection successful!');
  } catch (e) {
    alert('Connection failed: ' + (e.response?.data || 'Unknown error'));
  }
};

const confirmDelete = async (server) => {
  if (confirm(`Are you sure you want to delete server "${server.name}"? This will also remove all associated tickets.`)) {
    try {
      const token = localStorage.getItem('token');
      await axios.delete(`/api/servers/${server.id}`, {
        headers: { Authorization: `Bearer ${token}` }
      });
      fetchServers();
    } catch (e) {
      alert(e.response?.data || 'Failed to delete server');
    }
  }
};

onMounted(fetchServers);
</script>

<style scoped>
.password-field {
  display: inline-flex;
  align-items: center;
}
table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 20px;
}
th, td {
  border: 1px solid #ccc;
  padding: 8px;
  text-align: left;
}
button {
  margin-right: 5px;
}
</style>
