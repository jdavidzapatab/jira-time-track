<template>
  <div class="space-y-6 pb-20 sm:pb-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Jira Tickets</h1>
      <div class="hidden sm:flex gap-2">
        <button @click="addTicket(true)" class="btn btn-secondary flex items-center gap-1" title="Add a new ticket at the TOP of the list">
          <Plus :size="18" />
          <span>Add at Top</span>
        </button>
        <button ref="topAddButton" @click="addTicket(false)" class="btn btn-primary flex items-center gap-1" title="Add a new ticket at the BOTTOM of the list">
          <Plus :size="18" />
          <span>Add at Bottom</span>
        </button>
      </div>
    </div>

    <!-- Floating Add Button (shows when top button is not visible) -->
    <div v-if="!isTopButtonVisible" class="fixed bottom-6 right-6 z-50 flex flex-col gap-2">
      <button 
        @click="addTicket(true)" 
        class="p-3 bg-gray-600 text-white rounded-full shadow-lg hover:bg-gray-700 transition-transform active:scale-95 flex items-center justify-center" 
        title="Add a new ticket at the TOP of the list"
      >
        <Plus :size="20" />
      </button>
      <button 
        @click="addTicket(false)" 
        class="p-4 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 transition-transform active:scale-95 flex items-center justify-center" 
        title="Add a new ticket at the BOTTOM of the list"
      >
        <Plus :size="24" />
      </button>
    </div>

    <draggable 
      v-model="tickets" 
      @end="onDragEnd" 
      item-key="id" 
      handle=".drag-handle"
      class="space-y-4"
    >
      <template #item="{ element: ticket }">
        <div class="card flex flex-col gap-0 p-0 overflow-hidden relative group border-l-4 transition-colors" 
             :class="[
               ticket.fetchError ? 'border-l-red-500 bg-red-50 dark:bg-red-900/10' : 'border-l-transparent',
               ticket.last_stopwatch_start ? 'animate-soft-pulse' : ''
             ]">
          <div class="flex flex-wrap sm:flex-nowrap items-center gap-2 sm:gap-4 py-3 sm:py-4 px-2 sm:px-6">
            <!-- Drag Handle -->
            <div class="drag-handle cursor-grab active:cursor-grabbing text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300 p-1 flex-shrink-0">
              <GripVertical :size="20" />
            </div>

            <div class="flex-1 min-w-0 flex flex-col gap-1">
              <!-- Server Selector (Small dropdown above Ticket Number) -->
              <select v-model="ticket.server_id" @change="updateTicket(ticket)" class="border-none bg-transparent p-0 text-[10px] text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300 focus:ring-0 w-fit cursor-pointer" title="Select Jira Server">
                <option :value="null">Select Server</option>
                <option v-for="s in servers" :key="s.id" :value="s.id">{{ s.name }}</option>
              </select>
              
              <div class="flex items-center gap-2">
                <!-- Ticket Number -->
                <input v-model="ticket.ticket_number" @blur="onTicketNumberBlur(ticket)" @keyup.enter="onTicketNumberBlur(ticket)" class="input py-1 text-xs sm:text-sm font-bold w-24 sm:w-32 flex-shrink-0" placeholder="Ticket #" title="Enter Jira Ticket Number" />
                
                <!-- Ticket Summary -->
                <div class="flex-1 text-[11px] sm:text-xs text-gray-800 sm:text-gray-600 dark:text-gray-200 sm:dark:text-gray-400 font-medium sm:font-normal line-clamp-1 sm:line-clamp-2 leading-tight">
                  {{ ticket.ticket_summary || 'No summary fetched' }}
                </div>
              </div>
            </div>

            <div class="w-full sm:w-auto flex items-center justify-between sm:justify-end gap-3 sm:gap-4 mt-2 sm:mt-0 pt-2 sm:pt-0 border-t border-gray-50 dark:border-gray-800 sm:border-t-0">
              <div class="flex flex-col items-center">
                <div v-if="ticket.last_stopwatch_start" class="text-[10px] sm:text-xs font-mono text-green-600 dark:text-green-400 animate-pulse">
                  {{ formatStopwatch(getLiveTotal(ticket)) }}
                </div>
                <div v-else class="text-[10px] sm:text-xs font-mono text-gray-400 dark:text-gray-500">
                  {{ formatStopwatch(ticket.time_spent_seconds) }}
                </div>
                <input 
                  v-model="ticket.inputTime" 
                  @focus="onTimeFocus(ticket)"
                  @blur="onTimeBlur(ticket)"
                  @keyup.enter="toggleStopwatch(ticket)"
                  class="input py-1 text-[10px] sm:text-sm text-center font-mono w-20 sm:w-28 mt-0.5" 
                  placeholder="0m" 
                  title="Enter time (e.g. 1h 30m). Press ENTER to start/pause." 
                />
              </div>

              <div class="flex items-center space-x-1 sm:space-x-2">
                <button @click="toggleStopwatch(ticket)" class="p-1.5 sm:p-2 rounded-full transition-colors" :class="ticket.last_stopwatch_start ? 'text-orange-600 bg-orange-50 dark:bg-orange-900/20 hover:bg-orange-100 dark:hover:bg-orange-900/30' : 'text-green-600 bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/30'" :title="ticket.last_stopwatch_start ? 'Pause stopwatch' : 'Start stopwatch'">
                  <template v-if="ticket.last_stopwatch_start">
                    <Pause :size="16" class="sm:hidden" />
                    <Pause :size="20" class="hidden sm:block" />
                  </template>
                  <template v-else>
                    <Play :size="16" class="sm:hidden" />
                    <Play :size="20" class="hidden sm:block" />
                  </template>
                </button>

                <button @click="openJiraTicket(ticket)" class="p-1.5 sm:p-2 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded-full transition-colors" title="Open ticket in Jira">
                  <ExternalLink :size="16" class="sm:hidden" />
                  <ExternalLink :size="20" class="hidden sm:block" />
                </button>

                <button @click="openSaveDialog(ticket)" class="p-1.5 sm:p-2 text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-full transition-colors" title="Save worklog to Jira">
                  <Save :size="16" class="sm:hidden" />
                  <Save :size="20" class="hidden sm:block" />
                </button>
                
                <button @click="confirmClear(ticket)" class="p-1.5 sm:p-2 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-full transition-colors" title="Clear ticket data">
                  <Eraser :size="16" class="sm:hidden" />
                  <Eraser :size="20" class="hidden sm:block" />
                </button>
                
                <button @click="confirmDelete(ticket)" class="p-1.5 sm:p-2 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-full transition-colors" title="Delete ticket line">
                  <Trash2 :size="16" class="sm:hidden" />
                  <Trash2 :size="20" class="hidden sm:block" />
                </button>
              </div>
            </div>
          </div>
          <!-- Error message at the bottom -->
          <div v-if="ticket.fetchError" class="bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 text-[10px] px-4 py-1 border-t border-red-200 dark:border-red-900/50">
            {{ ticket.fetchError }}
          </div>
        </div>
      </template>
    </draggable>

    <div v-if="tickets.length === 0" class="card py-12 text-center text-gray-500 italic text-sm">
      No tickets added. Click the plus button to start tracking time.
    </div>

    <!-- Save Dialog Modal -->
    <div v-if="showSaveDialog" class="fixed inset-0 z-50 overflow-y-auto" aria-labelledby="modal-title" role="dialog" aria-modal="true">
      <div class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 bg-gray-600 bg-opacity-75 backdrop-blur-sm transition-opacity" aria-hidden="true" @click="showSaveDialog = false"></div>

        <!-- This element is to trick the browser into centering the modal contents. -->
        <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>

        <div class="relative inline-block align-middle bg-white dark:bg-gray-900 rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div class="bg-white dark:bg-gray-900 px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div class="sm:flex sm:items-start">
              <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left w-full">
                <h3 class="text-lg leading-6 font-medium text-gray-900 dark:text-white" id="modal-title">
                  Save Worklog to Jira
                </h3>
                <div class="mt-4 space-y-3">
                  <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded text-sm space-y-1">
                    <p><span class="font-semibold text-gray-700 dark:text-gray-300">Ticket:</span> {{ currentTicket.ticket_number }} - {{ currentTicket.ticket_summary }}</p>
                    <p><span class="font-semibold text-gray-700 dark:text-gray-300">Time to log:</span> {{ formatTime(getLiveTotal(currentTicket)) }}</p>
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Worklog Description <span class="text-red-500">*</span>
                    </label>
                    <textarea v-model="worklogDescription" class="input h-32" placeholder="Describe what you did..." title="Describe what you did" required></textarea>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="bg-gray-50 dark:bg-gray-800/50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse gap-2">
            <button @click="submitWorklog" class="btn btn-primary sm:ml-3">
              Submit to Jira
            </button>
            <button @click="saveForLater" class="btn btn-secondary mt-3 sm:mt-0">
              Save for Later
            </button>
            <button @click="showSaveDialog = false" class="btn border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 mt-3 sm:mt-0">
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>

    <ConfirmModal 
      :show="confirmModal.show"
      :title="confirmModal.title"
      :message="confirmModal.message"
      :confirmLabel="confirmModal.confirmLabel"
      :type="confirmModal.type"
      @confirm="confirmModal.action"
      @cancel="confirmModal.show = false"
    />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, inject, reactive, nextTick } from 'vue';
import draggable from 'vuedraggable';
import { Play, Pause, Save, Eraser, Trash2, Plus, GripVertical, ExternalLink } from 'lucide-vue-next';
import ConfirmModal from '../components/ConfirmModal.vue';

const servers = ref([]);
const tickets = ref([]);
const showSaveDialog = ref(false);
const currentTicket = ref(null);
const worklogDescription = ref('');
const toast = inject('toast');

const topAddButton = ref(null);
const isTopButtonVisible = ref(true);
let scrollObserver;

const confirmModal = reactive({
  show: false,
  title: '',
  message: '',
  confirmLabel: '',
  type: 'info',
  action: () => {}
});

let timerInterval;

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

const fetchTickets = async () => {
  try {
    const response = await fetchWithAuth('/api/tickets');
    const data = await response.json();
    tickets.value = data.map(t => ({
      ...t,
      fetchError: null,
      isEditing: false,
      inputTime: formatTime(t.time_spent_seconds + (t.last_stopwatch_start ? Math.floor((new Date() - new Date(t.last_stopwatch_start)) / 1000) : 0))
    }));
  } catch (e) {
    toast(e.message, 'error');
  }
};

const addTicket = async (atTop = false) => {
  let server_id = null;
  if (tickets.value.length > 0) {
    server_id = atTop ? tickets.value[0].server_id : tickets.value[tickets.value.length - 1].server_id;
  } else if (servers.value.length === 1) {
    server_id = servers.value[0].id;
  }

  try {
    const response = await fetchWithAuth('/api/tickets', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ server_id, at_top: atTop }),
    });
    const newTicket = await response.json();
    if (atTop) {
      tickets.value.unshift({ ...newTicket, fetchError: null, isEditing: false, inputTime: '0m' });
    } else {
      tickets.value.push({ ...newTicket, fetchError: null, isEditing: false, inputTime: '0m' });
    }

    // Scroll to new item and focus input
    await nextTick();
    const ticketElements = document.querySelectorAll('.card');
    const targetTicket = atTop ? ticketElements[0] : ticketElements[ticketElements.length - 1];
    if (targetTicket) {
      targetTicket.scrollIntoView({ behavior: 'smooth', block: 'center' });
      const input = targetTicket.querySelector('input[placeholder="Ticket #"]');
      if (input) input.focus();
    }
  } catch (e) {
    toast(e.message, 'error');
  }
};

const onDragEnd = async () => {
  const ticketIds = tickets.value.map(t => t.id);
  try {
    await fetchWithAuth('/api/tickets/reorder', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ticket_ids: ticketIds }),
    });
  } catch (e) {
    toast('Failed to save ticket order: ' + e.message, 'error');
  }
};

const updateTicket = async (ticket) => {
  try {
    await fetchWithAuth(`/api/tickets/${ticket.id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        server_id: ticket.server_id,
        ticket_number: ticket.ticket_number,
        ticket_summary: ticket.ticket_summary,
        time_spent_seconds: ticket.time_spent_seconds,
        saved_description: ticket.saved_description,
        last_stopwatch_start: ticket.last_stopwatch_start,
      }),
    });
  } catch (e) {
    toast('Failed to update ticket: ' + e.message, 'error');
  }
};

const onTicketNumberBlur = async (ticket) => {
  ticket.fetchError = null;
  await updateTicket(ticket);
  if (ticket.server_id && ticket.ticket_number) {
    try {
      const response = await fetchWithAuth(`/api/tickets/${ticket.id}/summary`);
      const data = await response.json();
      ticket.ticket_summary = data.summary;
    } catch (e) {
      ticket.fetchError = e.message;
      toast('Failed to fetch summary: ' + e.message, 'warning');
    }
  }
};

const onTimeFocus = (ticket) => {
  ticket.isEditing = true;
  ticket.originalInput = ticket.inputTime;
};

const onTimeBlur = (ticket) => {
  ticket.isEditing = false;
  if (ticket.inputTime !== ticket.originalInput) {
    const seconds = parseTime(ticket.inputTime);
    if (!isNaN(seconds)) {
      ticket.time_spent_seconds = seconds;
      if (ticket.last_stopwatch_start) {
        ticket.last_stopwatch_start = new Date().toISOString();
      }
      updateTicket(ticket);
    }
  }
  ticket.inputTime = formatTime(getLiveTotal(ticket));
};

const toggleStopwatch = (ticket) => {
  if (ticket.last_stopwatch_start) {
    const elapsed = Math.floor((new Date() - new Date(ticket.last_stopwatch_start)) / 1000);
    ticket.time_spent_seconds += elapsed;
    ticket.last_stopwatch_start = null;
  } else {
    if (ticket.isEditing && ticket.inputTime !== ticket.originalInput) {
      const seconds = parseTime(ticket.inputTime);
      if (!isNaN(seconds)) {
        ticket.time_spent_seconds = seconds;
      }
      ticket.isEditing = false;
    }
    const startTime = new Date();
    ticket.last_stopwatch_start = startTime.toISOString();
    now.value = startTime;
  }
  ticket.inputTime = formatTime(getLiveTotal(ticket));
  updateTicket(ticket);
};

const openJiraTicket = (ticket) => {
  if (!ticket.server_id || !ticket.ticket_number) {
    toast('Please select a server and enter a ticket number first', 'warning');
    return;
  }
  const server = servers.value.find(s => s.id === ticket.server_id);
  if (server) {
    const baseUrl = server.url.replace(/\/+$/, '');
    window.open(`${baseUrl}/browse/${ticket.ticket_number}`, '_blank');
  }
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
  toast('Description saved for later');
};

const submitWorklog = async () => {
  if (!worklogDescription.value.trim()) {
    toast('Worklog description is required', 'warning');
    return;
  }
  
  try {
    await fetchWithAuth(`/api/tickets/${currentTicket.value.id}/worklog`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        time_spent_formatted: formatTime(getLiveTotal(currentTicket.value)),
        description: worklogDescription.value,
      }),
    });
    
    toast('Worklog submitted successfully!');
    showSaveDialog.value = false;
    currentTicket.value.time_spent_seconds = 0;
    currentTicket.value.saved_description = '';
    currentTicket.value.last_stopwatch_start = null;
    updateTicket(currentTicket.value);
  } catch (e) {
    toast('Failed to submit: ' + e.message, 'error');
  }
};

const confirmClear = (ticket) => {
  confirmModal.title = 'Clear Ticket Data';
  confirmModal.message = `Are you sure you want to clear the data for ticket ${ticket.ticket_number || '(No Number)'} - ${ticket.ticket_summary || '(No Summary)'}? This will reset the timer and description.`;
  confirmModal.confirmLabel = 'Clear';
  confirmModal.type = 'danger';
  confirmModal.action = async () => {
    ticket.server_id = null;
    ticket.ticket_number = '';
    ticket.ticket_summary = '';
    ticket.time_spent_seconds = 0;
    ticket.saved_description = '';
    ticket.last_stopwatch_start = null;
    await updateTicket(ticket);
    toast('Ticket data cleared');
    confirmModal.show = false;
  };
  confirmModal.show = true;
};

const confirmDelete = (ticket) => {
  confirmModal.title = 'Delete Ticket';
  confirmModal.message = `Are you sure you want to delete ticket ${ticket.ticket_number || '(No Number)'} - ${ticket.ticket_summary || '(No Summary)'}?`;
  confirmModal.confirmLabel = 'Delete';
  confirmModal.type = 'danger';
  confirmModal.action = async () => {
    try {
      await fetchWithAuth(`/api/tickets/${ticket.id}`, {
        method: 'DELETE',
      });
      tickets.value = tickets.value.filter(t => t.id !== ticket.id);
      toast('Ticket deleted');
    } catch (e) {
      toast(e.message, 'error');
    } finally {
      confirmModal.show = false;
    }
  };
  confirmModal.show = true;
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

const formatStopwatch = (totalSeconds) => {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  return [h, m, s].map(v => v.toString().padStart(2, '0')).join(':');
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

const now = ref(new Date());

const getLiveTotal = (ticket) => {
  let total = ticket.time_spent_seconds;
  if (ticket.last_stopwatch_start) {
    const elapsed = Math.floor((now.value - new Date(ticket.last_stopwatch_start)) / 1000);
    total += Math.max(0, elapsed);
  }
  return total;
};

onMounted(async () => {
  await fetchServers();
  await fetchTickets();

  scrollObserver = new IntersectionObserver(([entry]) => {
    isTopButtonVisible.value = entry.isIntersecting;
  }, { threshold: 0 });

  if (topAddButton.value) {
    scrollObserver.observe(topAddButton.value);
  }

  timerInterval = setInterval(() => {
    now.value = new Date();
    tickets.value.forEach(t => {
      if (t.last_stopwatch_start && !t.isEditing) {
        t.inputTime = formatTime(getLiveTotal(t));
      }
    });
  }, 1000);
});

onUnmounted(() => {
  if (scrollObserver) {
    scrollObserver.disconnect();
  }
  clearInterval(timerInterval);
});
</script>
