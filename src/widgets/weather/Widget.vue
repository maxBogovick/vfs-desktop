<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import BaseWidget from '../../components/BaseWidget.vue';
import type { WidgetLayout } from '../../composables/useWidgets';

defineProps<{
  visible: boolean;
  id: string;
  layout: WidgetLayout;
}>();

defineEmits<{
  (e: 'close'): void;
  (e: 'update:layout', layout: Partial<WidgetLayout>): void;
}>();

interface WeatherData {
  temp: number;
  code: number;
  windSpeed: number;
  humidity?: number;
}

const location = ref('');
const isEditingLocation = ref(false);
const manualLocation = ref('');
const loading = ref(true);
const error = ref<string | null>(null);
const weather = ref<WeatherData | null>(null);
const lastUpdated = ref('');

const getWeatherDescription = (code: number) => {
  if (code === 0) return 'Clear sky';
  if ([1, 2, 3].includes(code)) return 'Partly cloudy';
  if ([45, 48].includes(code)) return 'Fog';
  if ([51, 53, 55].includes(code)) return 'Drizzle';
  if ([61, 63, 65].includes(code)) return 'Rain';
  if ([71, 73, 75].includes(code)) return 'Snow';
  if ([80, 81, 82].includes(code)) return 'Rain showers';
  if ([95, 96, 99].includes(code)) return 'Thunderstorm';
  return 'Unknown';
};

const getWeatherIcon = (code: number) => {
  if (code === 0) return '☀️';
  if ([1, 2, 3].includes(code)) return '⛅';
  if ([45, 48].includes(code)) return '🌫️';
  if ([51, 53, 55, 61, 63, 65, 80, 81, 82].includes(code)) return '🌧️';
  if ([71, 73, 75].includes(code)) return '❄️';
  if ([95, 96, 99].includes(code)) return '⛈️';
  return '🌡️';
};

const fetchLocationAndWeather = async () => {
  loading.value = true;
  error.value = null;

  try {
    let lat, lon;
    
    if (manualLocation.value) {
      const geoRes = await fetch(`https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(manualLocation.value)}&count=1&language=en&format=json`);
      const geoData = await geoRes.json();
      
      if (!geoData.results || geoData.results.length === 0) {
        throw new Error('City not found');
      }
      
      lat = geoData.results[0].latitude;
      lon = geoData.results[0].longitude;
      location.value = `${geoData.results[0].name}, ${geoData.results[0].country_code.toUpperCase()}`;
    } else {
      const ipRes = await fetch('https://ipapi.co/json/');
      if (!ipRes.ok) throw new Error('Failed to detect location');
      const ipData = await ipRes.json();
      lat = ipData.latitude;
      lon = ipData.longitude;
      location.value = `${ipData.city}, ${ipData.country_code}`;
    }

    const weatherRes = await fetch(
      `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current_weather=true&hourly=relativehumidity_2m`
    );
    const weatherData = await weatherRes.json();
    
    const currentHour = new Date().getHours();
    const humidity = weatherData.hourly?.relativehumidity_2m?.[currentHour] || 0;

    weather.value = {
      temp: weatherData.current_weather.temperature,
      code: weatherData.current_weather.weathercode,
      windSpeed: weatherData.current_weather.windspeed,
      humidity
    };
    
    lastUpdated.value = new Date().toLocaleTimeString();

  } catch (e: any) {
    console.error('Weather error:', e);
    error.value = e.message || 'Error loading data';
  } finally {
    loading.value = false;
  }
};

const handleSaveLocation = () => {
  isEditingLocation.value = false;
  localStorage.setItem('vfdir-weather-city', manualLocation.value);
  fetchLocationAndWeather();
};

const toggleEdit = () => {
  if (isEditingLocation.value) {
    isEditingLocation.value = false;
    const saved = localStorage.getItem('vfdir-weather-city');
    manualLocation.value = saved || '';
  } else {
    isEditingLocation.value = true;
  }
};

let refreshInterval: number | null = null;

onMounted(() => {
  const saved = localStorage.getItem('vfdir-weather-city');
  if (saved) {
    manualLocation.value = saved;
  }
  
  fetchLocationAndWeather();
  refreshInterval = window.setInterval(fetchLocationAndWeather, 30 * 60 * 1000);
});

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval);
});
</script>

<template>
  <BaseWidget
    :visible="visible"
    :id="id"
    :layout="layout"
    title="Weather"
    @close="$emit('close')"
    @update:layout="$emit('update:layout', $event)"
  >
    <template #actions>
      <button 
        @click="fetchLocationAndWeather" 
        class="text-[var(--vf-text-secondary)] hover:text-[var(--vf-accent-primary)] transition-colors"
        title="Refresh"
      >
        ↻
      </button>
    </template>

    <div class="p-4 text-[var(--vf-text-primary)] h-full overflow-y-auto">
      <!-- Location Header -->
      <div class="flex items-center justify-between mb-4 border-b border-[var(--vf-border-subtle)] pb-2 shrink-0">
        <div v-if="!isEditingLocation" class="flex items-center gap-2 overflow-hidden">
          <span class="font-bold truncate" title="Location">{{ location || 'Detecting...' }}</span>
          <button 
            @click="toggleEdit" 
            class="text-[10px] text-[var(--vf-text-tertiary)] hover:text-[var(--vf-accent-primary)]"
          >
            Edit
          </button>
        </div>
        <div v-else class="flex w-full gap-1">
          <input 
            v-model="manualLocation" 
            @keydown.enter="handleSaveLocation"
            @keydown.esc="toggleEdit"
            placeholder="City Name"
            class="flex-1 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border-default)] rounded px-1 text-xs focus:outline-none focus:border-[var(--vf-accent-primary)]"
            autoFocus
          />
          <button @click="handleSaveLocation" class="text-green-500 hover:text-green-400">✓</button>
          <button @click="toggleEdit" class="text-red-500 hover:text-red-400">✕</button>
        </div>
      </div>

      <!-- Loading/Error States -->
      <div v-if="loading && !weather" class="flex justify-center py-4">
        <span class="animate-pulse text-[var(--vf-text-secondary)]">Loading weather...</span>
      </div>
      
      <div v-else-if="error" class="text-red-400 text-center py-2 text-xs">
        {{ error }}
      </div>

      <!-- Weather Data -->
      <div v-else-if="weather" class="space-y-4">
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-4xl font-bold">{{ weather.temp }}°C</span>
            <span class="text-sm text-[var(--vf-text-secondary)]">{{ getWeatherDescription(weather.code) }}</span>
          </div>
          <div class="text-4xl">
            {{ getWeatherIcon(weather.code) }}
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2 text-xs text-[var(--vf-text-secondary)] bg-[var(--vf-bg-secondary)] p-2 rounded">
          <div class="flex flex-col items-center">
            <span class="mb-1">Wind</span>
            <span class="font-bold text-[var(--vf-text-primary)]">{{ weather.windSpeed }} km/h</span>
          </div>
          <div class="flex flex-col items-center border-l border-[var(--vf-border-subtle)]">
            <span class="mb-1">Humidity</span>
            <span class="font-bold text-[var(--vf-text-primary)]">{{ weather.humidity }}%</span>
          </div>
        </div>
        
        <div class="text-[9px] text-[var(--vf-text-tertiary)] text-center">
          Updated: {{ lastUpdated }}
        </div>
      </div>
    </div>
  </BaseWidget>
</template>