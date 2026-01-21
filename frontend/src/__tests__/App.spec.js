import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import App from '../App.vue';
import { createRouter, createWebHistory } from 'vue-router';

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), 
    removeListener: vi.fn(), 
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: '/', component: { template: '<div>Home</div>' } }]
});

describe('App.vue Theme logic', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.className = '';
  });

  it('defaults to system theme', async () => {
    mount(App, {
      global: {
        plugins: [router]
      }
    });
    // The ref is initialized with 'system' if nothing in localStorage
    // Logic applies theme on mount
  });

  it('applies dark class when theme is dark', async () => {
    localStorage.setItem('theme', 'dark');
    mount(App, {
      global: {
        plugins: [router]
      }
    });
    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });

  it('removes dark class when theme is light', async () => {
    document.documentElement.classList.add('dark');
    localStorage.setItem('theme', 'light');
    mount(App, {
      global: {
        plugins: [router]
      }
    });
    expect(document.documentElement.classList.contains('dark')).toBe(false);
  });

  it('changes theme when clicking buttons', async () => {
    localStorage.setItem('token', 'fake-token'); // To show nav
    const wrapper = mount(App, {
      global: {
        plugins: [router]
      }
    });
    
    const darkButton = wrapper.find('button[title="Dark theme"]');
    await darkButton.trigger('click');
    expect(localStorage.getItem('theme')).toBe('dark');
    expect(document.documentElement.classList.contains('dark')).toBe(true);

    const lightButton = wrapper.find('button[title="Light theme"]');
    await lightButton.trigger('click');
    expect(localStorage.getItem('theme')).toBe('light');
    expect(document.documentElement.classList.contains('dark')).toBe(false);
  });
});
