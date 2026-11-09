import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  define: {
    'process.env.NODE_ENV': JSON.stringify('production'),
    appName: JSON.stringify('mealshop'),
    appVersion: JSON.stringify('0.1.0'),
  },
  build: {
    lib: {
      entry: 'src/main.ts',
      name: 'Mealshop',
      formats: ['iife'],
      fileName: () => 'mealshop.js',
      cssFileName: 'mealshop',
    },
    cssCodeSplit: false,
    sourcemap: false,
  },
})
