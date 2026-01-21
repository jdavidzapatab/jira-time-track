<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Jira Servers</h1>
    </div>

    <div class="card p-4 sm:p-6">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Add New Server</h2>
      <form @submit.prevent="addServer" class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4">
        <div>
          <label class="block text-xs sm:text-sm font-medium text-gray-700 dark:text-gray-300">Name</label>
          <input v-model="newServer.name" placeholder="E.g. Company Jira" required class="input mt-1 text-sm" title="Friendly name for the server" />
        </div>
        <div>
          <label class="block text-xs sm:text-sm font-medium text-gray-700 dark:text-gray-300">URL</label>
          <input v-model="newServer.url" placeholder="https://your-domain.atlassian.net" required class="input mt-1 text-sm" title="Jira Server URL" />
        </div>
        <div>
          <label class="block text-xs sm:text-sm font-medium text-gray-700 dark:text-gray-300">Username</label>
          <input v-model="newServer.username" placeholder="Email or Username" required class="input mt-1 text-sm" title="Your Jira username or email" />
        </div>
        <div>
          <label class="block text-xs sm:text-sm font-medium text-gray-700 dark:text-gray-300">Password / API Token</label>
          <div class="relative mt-1">
            <input :type="showPassword ? 'text' : 'password'" v-model="newServer.password" placeholder="Password" required class="input pr-10 text-sm" title="Your Jira password or API token" />
            <button type="button" @click="showPassword = !showPassword" class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" title="Toggle password visibility">
              <Eye v-if="!showPassword" :size="18" />
              <EyeOff v-else :size="18" />
            </button>
          </div>
        </div>
        <div class="md:col-span-2 flex flex-col sm:flex-row justify-end gap-2">
          <button type="button" @click="testNewServer" class="btn btn-secondary text-sm py-2" title="Test connection with these credentials">
            <RefreshCcw :size="18" />
            Test Connection
          </button>
          <button type="submit" class="btn btn-primary text-sm py-2" title="Add new Jira server">
            <Plus :size="18" />
            Add Server
          </button>
        </div>
      </form>
    </div>

    <div class="card overflow-hidden p-0 sm:p-0">
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-800">
          <thead class="bg-gray-50 dark:bg-gray-800/50">
            <tr>
              <th scope="col" class="px-3 sm:px-6 py-3 text-left text-[10px] sm:text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Name</th>
              <th scope="col" class="px-3 sm:px-6 py-3 text-left text-[10px] sm:text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">URL</th>
              <th scope="col" class="hidden sm:table-cell px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Username</th>
              <th scope="col" class="px-3 sm:px-6 py-3 text-right text-[10px] sm:text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-800">
            <tr v-for="server in servers" :key="server.id" class="hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
              <td class="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap text-xs sm:text-sm font-medium text-gray-900 dark:text-white">{{ server.name }}</td>
              <td class="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap text-xs sm:text-sm text-gray-500 dark:text-gray-400">{{ server.url }}</td>
              <td class="hidden sm:table-cell px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">{{ server.username }}</td>
              <td class="px-3 sm:px-6 py-3 sm:py-4 whitespace-nowrap text-right text-xs sm:text-sm font-medium space-x-2">
                <button @click="testCredentials(server.id)" class="text-blue-600 dark:text-blue-400 hover:text-blue-900 dark:hover:text-blue-300 inline-flex items-center" title="Test connection to this server">
                  <RefreshCcw :size="16" class="sm:hidden" />
                  <RefreshCcw :size="18" class="hidden sm:block" />
                </button>
                <button @click="confirmDelete(server)" class="text-red-600 dark:text-red-400 hover:text-red-900 dark:hover:text-red-300 inline-flex items-center" title="Delete this server">
                  <Trash2 :size="16" class="sm:hidden" />
                  <Trash2 :size="18" class="hidden sm:block" />
                </button>
              </td>
            </tr>
            <tr v-if="servers.length === 0">
              <td colspan="4" class="px-6 py-10 text-center text-gray-500 dark:text-gray-400 text-sm italic">
                No servers added yet.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <ConfirmModal 
      :show="showDeleteModal"
      title="Delete Server"
      :message="`Are you sure you want to delete server '${serverToDelete?.name}'? This will also remove all associated tickets.`"
      confirmLabel="Delete"
      type="danger"
      @confirm="deleteServer"
      @cancel="showDeleteModal = false"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, inject } from 'vue';
import { Eye, EyeOff, RefreshCcw, Trash2, Plus } from 'lucide-vue-next';
import ConfirmModal from '../components/ConfirmModal.vue';

const servers = ref([]);
const newServer = ref({ name: '', url: '', username: '', password: '' });
const showPassword = ref(false);
const toast = inject('toast');

const showDeleteModal = ref(false);
const serverToDelete = ref(null);

const fetchWithAuth = async (url, options = {}) => {
  const token = localStorage.getItem('token');
  const headers = {
    ...options.headers,
    'Authorization': `Bearer ${token}`,
  };
  
  const response = await fetch(url, { ...options, headers });
  if (!response.ok) {
    if (response.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/login';
    }
    const errorText = await response.text();
    throw new Error(errorText || 'Request failed');
  }
  return response;
};

const fetchServers = async () => {
  try {
    const response = await fetchWithAuth('/api/servers');
    servers.value = await response.json();
  } catch (e) {
    toast(e.message, 'error');
  }
};

const addServer = async () => {
  try {
    await fetchWithAuth('/api/servers', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newServer.value),
    });
    newServer.value = { name: '', url: '', username: '', password: '' };
    fetchServers();
    toast('Server added successfully');
  } catch (e) {
    toast(e.message, 'error');
  }
};

const testNewServer = async () => {
  try {
    await fetchWithAuth('/api/servers/test-new', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        url: newServer.value.url,
        username: newServer.value.username,
        password: newServer.value.password
      }),
    });
    toast('Connection successful!');
  } catch (e) {
    toast('Connection failed: ' + e.message, 'error');
  }
};

const testCredentials = async (id) => {
  try {
    await fetchWithAuth(`/api/servers/${id}/test`, {
      method: 'POST',
    });
    toast('Connection successful!');
  } catch (e) {
    toast('Connection failed: ' + e.message, 'error');
  }
};

const confirmDelete = (server) => {
  serverToDelete.value = server;
  showDeleteModal.value = true;
};

const deleteServer = async () => {
  if (!serverToDelete.value) return;
  try {
    await fetchWithAuth(`/api/servers/${serverToDelete.value.id}`, {
      method: 'DELETE',
    });
    fetchServers();
    toast('Server deleted');
  } catch (e) {
    toast(e.message, 'error');
  } finally {
    showDeleteModal.value = false;
    serverToDelete.value = null;
  }
};

onMounted(fetchServers);
</script>
