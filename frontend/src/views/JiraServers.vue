<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900">Jira Servers</h1>
    </div>

    <div class="card">
      <h2 class="text-lg font-medium text-gray-900 mb-4">Add New Server</h2>
      <form @submit.prevent="addServer" class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700">Name</label>
          <input v-model="newServer.name" placeholder="E.g. Company Jira" required class="input mt-1" title="Friendly name for the server" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700">URL</label>
          <input v-model="newServer.url" placeholder="https://your-domain.atlassian.net" required class="input mt-1" title="Jira Server URL" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700">Username</label>
          <input v-model="newServer.username" placeholder="Email or Username" required class="input mt-1" title="Your Jira username or email" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700">Password / API Token</label>
          <div class="relative mt-1">
            <input :type="showPassword ? 'text' : 'password'" v-model="newServer.password" placeholder="Password" required class="input pr-10" title="Your Jira password or API token" />
            <button type="button" @click="showPassword = !showPassword" class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600" title="Toggle password visibility">
              <Eye v-if="!showPassword" :size="18" />
              <EyeOff v-else :size="18" />
            </button>
          </div>
        </div>
        <div class="md:col-span-2 flex justify-end">
          <button type="submit" class="btn btn-primary" title="Add new Jira server">
            Add Server
          </button>
        </div>
      </form>
    </div>

    <div class="card overflow-hidden">
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Name</th>
              <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">URL</th>
              <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Username</th>
              <th scope="col" class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            <tr v-for="server in servers" :key="server.id" class="hover:bg-gray-50 transition-colors">
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{{ server.name }}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{{ server.url }}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{{ server.username }}</td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                <button @click="testCredentials(server.id)" class="text-blue-600 hover:text-blue-900 inline-flex items-center" title="Test connection to this server">
                  <RefreshCcw :size="18" />
                </button>
                <button @click="confirmDelete(server)" class="text-red-600 hover:text-red-900 inline-flex items-center" title="Delete this server">
                  <Trash2 :size="18" />
                </button>
              </td>
            </tr>
            <tr v-if="servers.length === 0">
              <td colspan="4" class="px-6 py-10 text-center text-gray-500">
                No servers added yet.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
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
