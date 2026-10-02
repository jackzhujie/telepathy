import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  getUserProfile,
  updateUserProfile as updateUserProfileApi,
  getUserInterests,
  addUserInterest as addUserInterestApi,
  removeUserInterest as removeUserInterestApi,
  type UserProfile,
} from '@/api/tauri';

export const useUserProfileStore = defineStore('profile', () => {
  const profile = ref<UserProfile>({
    name: '',
    gender: '',
    age_group: '',
    occupation: '',
    industry: '',
    language: 'auto',
  });

  const interests = ref<string[]>([]);
  const isLoading = ref(false);
  const isSaving = ref(false);

  async function fetchProfile() {
    isLoading.value = true;
    try {
      const [profileData, interestsData] = await Promise.all([
        getUserProfile(),
        getUserInterests(),
      ]);
      profile.value = profileData;
      interests.value = interestsData;
    } catch (e) {
      console.error('Failed to fetch user profile:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function updateProfile(data: Partial<UserProfile>) {
    isSaving.value = true;
    try {
      const updated = { ...profile.value, ...data };
      await updateUserProfileApi(updated);
      profile.value = updated;
    } catch (e) {
      console.error('Failed to update user profile:', e);
      throw e;
    } finally {
      isSaving.value = false;
    }
  }

  async function addInterest(interest: string) {
    if (!interest.trim() || interests.value.includes(interest)) return;

    await addUserInterestApi(interest);
    interests.value.push(interest);
  }

  async function removeInterest(interest: string) {
    await removeUserInterestApi(interest);
    interests.value = interests.value.filter(i => i !== interest);
  }

  return {
    profile,
    interests,
    isLoading,
    isSaving,
    fetchProfile,
    updateProfile,
    addInterest,
    removeInterest,
  };
});