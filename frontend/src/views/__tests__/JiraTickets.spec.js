import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import JiraTickets from '../JiraTickets.vue';

// Mock dependencies
vi.mock('lucide-vue-next', () => ({
  Play: { template: '<span>Play</span>' },
  Pause: { template: '<span>Pause</span>' },
  Save: { template: '<span>Save</span>' },
  Eraser: { template: '<span>Eraser</span>' },
  Trash2: { template: '<span>Trash2</span>' },
  Plus: { template: '<span>Plus</span>' },
  GripVertical: { template: '<span>GripVertical</span>' },
  ExternalLink: { template: '<span>ExternalLink</span>' },
}));

vi.mock('vuedraggable', () => ({
  default: {
    template: '<div><slot v-for="item in modelValue" :element="item" name="item"></slot></div>',
    props: ['modelValue']
  }
}));

// Mock IntersectionObserver
global.IntersectionObserver = class {
  constructor() {}
  observe() {}
  unobserve() {}
  disconnect() {}
};

describe('JiraTickets.vue', () => {
  let wrapper;
  const mockToast = vi.fn();

  beforeEach(() => {
    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve([]),
      })
    );

    wrapper = mount(JiraTickets, {
      attachTo: document.body,
      global: {
        provide: {
          toast: mockToast
        },
        stubs: {
          ConfirmModal: true
        }
      }
    });
  });

  afterEach(() => {
    wrapper.unmount();
  });

  it('renders the "Open in Jira" button for ticket line items', async () => {
    // Inject some ticket data
    wrapper.vm.tickets = [
      { 
        id: '1', 
        server_id: 's1', 
        ticket_number: 'PROJ-123', 
        ticket_summary: 'Test Summary',
        time_spent_seconds: 0,
        last_stopwatch_start: null
      }
    ];
    
    await wrapper.vm.$nextTick();

    const openButton = wrapper.find('button[title="Open ticket in Jira"]');
    expect(openButton.exists()).toBe(true);
  });

  it('calls window.open with correct URL when "Open in Jira" is clicked', async () => {
    const windowSpy = vi.spyOn(window, 'open').mockImplementation(() => {});
    
    const serverId = 'server-uuid';
    wrapper.vm.servers = [{ id: serverId, url: 'https://jira.example.com' }];
    wrapper.vm.tickets = [
      { 
        id: '1', 
        server_id: serverId, 
        ticket_number: 'PROJ-123', 
        ticket_summary: 'Test Summary',
        time_spent_seconds: 0,
        last_stopwatch_start: null
      }
    ];

    await wrapper.vm.$nextTick();

    const openButton = wrapper.find('button[title="Open ticket in Jira"]');
    await openButton.trigger('click');

    expect(windowSpy).toHaveBeenCalledWith('https://jira.example.com/browse/PROJ-123', '_blank');
    windowSpy.mockRestore();
  });

  it('shows warning toast if server or ticket number is missing', async () => {
    wrapper.vm.tickets = [
      { 
        id: '1', 
        server_id: null, 
        ticket_number: '', 
        ticket_summary: '',
        time_spent_seconds: 0,
        last_stopwatch_start: null
      }
    ];

    await wrapper.vm.$nextTick();

    const openButton = wrapper.find('button[title="Open ticket in Jira"]');
    await openButton.trigger('click');

    expect(mockToast).toHaveBeenCalledWith('Please select a server and enter a ticket number first', 'warning');
  });

  it('scrolls and focuses new ticket when added', async () => {
    // Mock scrollIntoView if it doesn't exist in JSDOM
    if (!HTMLElement.prototype.scrollIntoView) {
      HTMLElement.prototype.scrollIntoView = vi.fn();
    }
    const scrollSpy = vi.spyOn(HTMLElement.prototype, 'scrollIntoView').mockImplementation(() => {});
    const focusSpy = vi.spyOn(HTMLElement.prototype, 'focus').mockImplementation(() => {});

    global.fetch = vi.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ 
          id: 'new-id', 
          server_id: null, 
          ticket_number: '', 
          ticket_summary: '',
          time_spent_seconds: 0,
          saved_description: '',
          last_stopwatch_start: null
        }),
      })
    );

    await wrapper.vm.addTicket();
    
    expect(scrollSpy).toHaveBeenCalled();
    expect(focusSpy).toHaveBeenCalled();
    
    scrollSpy.mockRestore();
    focusSpy.mockRestore();
  });
});
