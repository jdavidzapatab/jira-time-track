import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import ConfirmModal from '../ConfirmModal.vue';

describe('ConfirmModal.vue', () => {
  it('renders correctly when show is true', () => {
    const wrapper = mount(ConfirmModal, {
      props: {
        show: true,
        title: 'Delete Item',
        message: 'Are you sure?',
      }
    });

    expect(wrapper.find('#modal-title').text()).toBe('Delete Item');
    expect(wrapper.text()).toContain('Are you sure?');
  });

  it('does not render when show is false', () => {
    const wrapper = mount(ConfirmModal, {
      props: {
        show: false
      }
    });

    expect(wrapper.find('[role="dialog"]').exists()).toBe(false);
  });

  it('emits confirm when confirm button is clicked', async () => {
    const wrapper = mount(ConfirmModal, {
      props: {
        show: true
      }
    });

    await wrapper.find('button.btn-primary').trigger('click');
    expect(wrapper.emitted()).toHaveProperty('confirm');
  });

  it('emits cancel when cancel button is clicked', async () => {
    const wrapper = mount(ConfirmModal, {
      props: {
        show: true
      }
    });

    await wrapper.find('button.bg-white').trigger('click');
    expect(wrapper.emitted()).toHaveProperty('cancel');
  });

  it('applies danger classes when type is danger', () => {
    const wrapper = mount(ConfirmModal, {
      props: {
        show: true,
        type: 'danger'
      }
    });

    expect(wrapper.find('.bg-red-100').exists()).toBe(true);
    expect(wrapper.find('.btn-danger').exists()).toBe(true);
  });
});
