<template>
  <div>
    <h2>Jira Tickets</h2>
    <button @click="addTicket" title="Add a new ticket line item">Add Ticket</button>

    <div v-for="ticket in tickets" :key="ticket.id" class="ticket-row">
      <select v-model="ticket.server_id" @change="updateTicket(ticket)" title="Select Jira Server">
        <option :value="null">Select Server</option>
        <option v-for="s in servers" :key="s.id" :value="s.id">{{ s.name }}</option>
      </select>

      <input v-model="ticket.ticket_number" @blur="onTicketNumberBlur(ticket)" placeholder="Ticket #" title="Enter Jira Ticket Number (e.g. PROJ-123)" />

      <span class="ticket-summary">{{ ticket.ticket_summary || '...' }}</span>

      <input :value="formatTime(ticket.time_spent_seconds)" @input="e => onTimeInput(ticket, e.target.value)" placeholder="0m" title="Enter time (e.g. 1h 30m, 2d 6h)" />

      <button @click="toggleStopwatch(ticket)" :title="ticket.last_stopwatch_start ? 'Pause stopwatch' : 'Start stopwatch'">
        <Pause v-if="ticket.last_stopwatch_start" :size="16" />
        <Play v-else :size="16" />
      </button>

      <button @click="openSaveDialog(ticket)" title="Save worklog to Jira"><Save :size="16" /></button>
      <button @click="confirmClear(ticket)" title="Clear ticket data"><Eraser :size="16" /></button>
      <button @click="confirmDelete(ticket)" title="Delete ticket line"><Trash2 :size="16" /></button>
    </div>

    <!-- Save Dialog -->
    <div v-if="showSaveDialog" class="modal">
      <div class="modal-content">
        <h3>Save Worklog</h3>
        <p><strong>Ticket:</strong> {{ currentTicket.ticket_number }} - {{ currentTicket.ticket_summary }}</p>
        <p><strong>Time:</strong> {{ formatTime(currentTicket.time_spent_seconds) }}</p>
        <textarea v-model="worklogDescription" placeholder="Enter description of time spent" title="Describe what you did"></textarea>
        <div class="modal-actions">
          <button @click="showSaveDialog = false">Cancel</button>
          <button @click="saveForLater">Save for Later</button>
          <button @click="submitWorklog">Submit</button>
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
    // If it was running, calculate elapsed time since last start
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
    // Reset stopwatch if manually changed
    ticket.last_stopwatch_start = null;
    updateTicket(ticket);
  }
};

const toggleStopwatch = (ticket) => {
  if (ticket.last_stopwatch_start) {
    // Pause
    const elapsed = Math.floor((new Date() - new Date(ticket.last_stopwatch_start)) / 1000);
    ticket.time_spent_seconds += elapsed;
    ticket.last_stopwatch_start = null;
  } else {
    // Start
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
    // Optionally clear after submit
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
  const days = Math.floor(totalSeconds / (8 * 3600)); // Assuming 8h workday for Jira? Standard is usually customizable, let's use 8h
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

<style scoped>
.ticket-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  border-bottom: 1px solid #eee;
  padding-bottom: 10px;
}
.ticket-summary {
  flex: 1;
  font-size: 0.9em;
  color: #666;
}
.modal {
  position: fixed;
  top: 0; left: 0; width: 100%; height: 100%;
  background: rgba(0,0,0,0.5);
  display: flex;
  justify-content: center;
  align-items: center;
}
.modal-content {
  background: white;
  padding: 20px;
  border-radius: 8px;
  width: 400px;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
}
textarea {
  width: 100%;
  height: 100px;
}
</style>
