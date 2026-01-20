<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900">Jira Tickets</h1>
      <button @click="addTicket" class="btn btn-primary" title="Add a new ticket line item">
        Add Ticket
      </button>
    </div>

    <div class="space-y-4">
      <div v-for="ticket in tickets" :key="ticket.id" class="card flex flex-wrap items-center gap-4 py-4 px-6">
        <div class="w-full md:w-48">
          <select v-model="ticket.server_id" @change="updateTicket(ticket)" class="input py-1.5" title="Select Jira Server">
            <option :value="null">Select Server</option>
            <option v-for="s in servers" :key="s.id" :value="s.id">{{ s.name }}</option>
          </select>
        </div>

        <div class="w-full md:w-32">
          <input v-model="ticket.ticket_number" @blur="onTicketNumberBlur(ticket)" class="input py-1.5" placeholder="Ticket #" title="Enter Jira Ticket Number (e.g. PROJ-123)" />
        </div>

        <div class="flex-1 min-w-[200px] text-sm text-gray-600 truncate italic">
          {{ ticket.ticket_summary || 'No summary fetched' }}
        </div>

        <div class="w-28">
          <input :value="formatTime(ticket.time_spent_seconds)" @input="e => onTimeInput(ticket, e.target.value)" class="input py-1.5 text-center font-mono" placeholder="0m" title="Enter time (e.g. 1h 30m, 2d 6h)" />
        </div>

        <div class="flex items-center space-x-2">
          <button @click="toggleStopwatch(ticket)" class="p-2 rounded-full transition-colors" :class="ticket.last_stopwatch_start ? 'text-orange-600 bg-orange-50 hover:bg-orange-100' : 'text-green-600 bg-green-50 hover:bg-green-100'" :title="ticket.last_stopwatch_start ? 'Pause stopwatch' : 'Start stopwatch'">
            <Pause v-if="ticket.last_stopwatch_start" :size="20" />
            <Play v-else :size="20" />
          </button>

          <button @click="openSaveDialog(ticket)" class="p-2 text-blue-600 hover:bg-blue-50 rounded-full transition-colors" title="Save worklog to Jira">
            <Save :size="20" />
          </button>
          
          <button @click="confirmClear(ticket)" class="p-2 text-gray-500 hover:bg-gray-100 rounded-full transition-colors" title="Clear ticket data">
            <Eraser :size="20" />
          </button>
          
          <button @click="confirmDelete(ticket)" class="p-2 text-red-500 hover:bg-red-50 rounded-full transition-colors" title="Delete ticket line">
            <Trash2 :size="20" />
          </button>
        </div>
      </div>

      <div v-if="tickets.length === 0" class="card py-12 text-center text-gray-500 italic">
        No tickets added. Click "Add Ticket" to start tracking time.
      </div>
    </div>

    <!-- Save Dialog Modal -->
    <div v-if="showSaveDialog" class="fixed inset-0 z-10 overflow-y-auto" aria-labelledby="modal-title" role="dialog" aria-modal="true">
      <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" aria-hidden="true" @click="showSaveDialog = false"></div>

        <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>

        <div class="inline-block align-middle bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div class="sm:flex sm:items-start">
              <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left w-full">
                <h3 class="text-lg leading-6 font-medium text-gray-900" id="modal-title">
                  Save Worklog to Jira
                </h3>
                <div class="mt-4 space-y-3">
                  <div class="bg-gray-50 p-3 rounded text-sm space-y-1">
                    <p><span class="font-semibold text-gray-700">Ticket:</span> {{ currentTicket.ticket_number }} - {{ currentTicket.ticket_summary }}</p>
                    <p><span class="font-semibold text-gray-700">Time to log:</span> {{ formatTime(currentTicket.time_spent_seconds) }}</p>
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">Worklog Description</label>
                    <textarea v-model="worklogDescription" class="input h-32" placeholder="Describe what you did..." title="Describe what you did"></textarea>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse gap-2">
            <button @click="submitWorklog" class="btn btn-primary sm:ml-3">
              Submit to Jira
            </button>
            <button @click="saveForLater" class="btn btn-secondary mt-3 sm:mt-0">
              Save for Later
            </button>
            <button @click="showSaveDialog = false" class="btn border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 mt-3 sm:mt-0">
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import axios from 'axios';
import { Play, Pause, Save, Eraser, Trash2 } from 'lucide-vue-next';

const servers = ref([]);
const tickets = ref([]);
const showSaveDialog = ref(false);
const currentTicket = ref(null);
const worklogDescription = ref('');

let timerInterval;

const fetchServers = async () => {
  const token = localStorage.getItem('token');
  const response = await axios.get('/api/servers', { headers: { Authorization: `Bearer ${token}` } });
  servers.value = response.data;
};

const fetchTickets = async () => {
  const token = localStorage.getItem('token');
  const response = await axios.get('/api/tickets', { headers: { Authorization: `Bearer ${token}` } });
  tickets.value = response.data.map(t => ({
    ...t,
    time_spent_seconds: t.time_spent_seconds + (t.last_stopwatch_start ? Math.floor((new Date() - new Date(t.last_stopwatch_start)) / 1000) : 0)
  }));
};

const addTicket = async () => {
  const token = localStorage.getItem('token');
  const response = await axios.post('/api/tickets', {}, { headers: { Authorization: `Bearer ${token}` } });
  tickets.value.push(response.data);
};

const updateTicket = async (ticket) => {
  const token = localStorage.getItem('token');
  await axios.put(`/api/tickets/${ticket.id}`, {
    server_id: ticket.server_id,
    ticket_number: ticket.ticket_number,
    ticket_summary: ticket.ticket_summary,
    time_spent_seconds: ticket.time_spent_seconds,
    saved_description: ticket.saved_description,
    last_stopwatch_start: ticket.last_stopwatch_start,
  }, { headers: { Authorization: `Bearer ${token}` } });
};

const onTicketNumberBlur = async (ticket) => {
  if (ticket.server_id && ticket.ticket_number) {
    const token = localStorage.getItem('token');
    try {
      const response = await axios.get(`/api/tickets/${ticket.id}/summary`, { headers: { Authorization: `Bearer ${token}` } });
      ticket.ticket_summary = response.data.summary;
    } catch (e) {
      console.error('Failed to fetch summary', e);
    }
  }
  updateTicket(ticket);
};

const onTimeInput = (ticket, value) => {
  const seconds = parseTime(value);
  if (!isNaN(seconds)) {
    ticket.time_spent_seconds = seconds;
    ticket.last_stopwatch_start = null;
    updateTicket(ticket);
  }
};

const toggleStopwatch = (ticket) => {
  if (ticket.last_stopwatch_start) {
    const elapsed = Math.floor((new Date() - new Date(ticket.last_stopwatch_start)) / 1000);
    ticket.time_spent_seconds += elapsed;
    ticket.last_stopwatch_start = null;
  } else {
    ticket.last_stopwatch_start = new Date().toISOString();
  }
  updateTicket(ticket);
};

const openSaveDialog = (ticket) => {
  currentTicket.value = ticket;
  worklogDescription.value = ticket.saved_description || '';
  showSaveDialog.value = true;
};

const saveForLater = async () => {
  currentTicket.value.saved_description = worklogDescription.value;
  await updateTicket(currentTicket.value);
  showSaveDialog.value = false;
};

const submitWorklog = async () => {
  try {
    const token = localStorage.getItem('token');
    await axios.post(`/api/tickets/${currentTicket.value.id}/worklog`, {
      time_spent_formatted: formatTime(currentTicket.value.time_spent_seconds),
      description: worklogDescription.value,
    }, { headers: { Authorization: `Bearer ${token}` } });
    
    alert('Worklog submitted successfully!');
    showSaveDialog.value = false;
    currentTicket.value.time_spent_seconds = 0;
    currentTicket.value.saved_description = '';
    currentTicket.value.last_stopwatch_start = null;
    updateTicket(currentTicket.value);
  } catch (e) {
    alert('Failed to submit: ' + (e.response?.data || 'Unknown error'));
  }
};

const confirmClear = async (ticket) => {
  if (confirm('Are you sure you want to clear this ticket line data?')) {
    ticket.server_id = null;
    ticket.ticket_number = '';
    ticket.ticket_summary = '';
    ticket.time_spent_seconds = 0;
    ticket.saved_description = '';
    ticket.last_stopwatch_start = null;
    await updateTicket(ticket);
  }
};

const confirmDelete = async (ticket) => {
  if (confirm('Are you sure you want to delete this ticket line?')) {
    const token = localStorage.getItem('token');
    await axios.delete(`/api/tickets/${ticket.id}`, { headers: { Authorization: `Bearer ${token}` } });
    tickets.value = tickets.value.filter(t => t.id !== ticket.id);
  }
};

const formatTime = (totalSeconds) => {
  if (!totalSeconds) return '0m';
  const days = Math.floor(totalSeconds / (8 * 3600));
  const remainingAfterDays = totalSeconds % (8 * 3600);
  const hours = Math.floor(remainingAfterDays / 3600);
  const minutes = Math.floor((remainingAfterDays % 3600) / 60);
  
  let parts = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0 || parts.length === 0) parts.push(`${minutes}m`);
  return parts.join(' ');
};

const parseTime = (str) => {
  const regex = /((?<d>\d+)d)?\s*((?<h>\d+)h)?\s*((?<m>\d+)m)?/;
  const match = str.match(regex);
  if (!match) return 0;
  const d = parseInt(match.groups.d || 0);
  const h = parseInt(match.groups.h || 0);
  const m = parseInt(match.groups.m || 0);
  return d * 8 * 3600 + h * 3600 + m * 60;
};

onMounted(async () => {
  await fetchServers();
  await fetchTickets();
  timerInterval = setInterval(() => {
    tickets.value.forEach(t => {
      if (t.last_stopwatch_start) {
        t.time_spent_seconds++;
      }
    });
  }, 1000);
});

onUnmounted(() => {
  clearInterval(timerInterval);
});
</script>
