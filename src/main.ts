import { createApp } from "vue";
import App from "./App.vue";
import { i18n } from "./i18n";
import "./assets/styles/global.css";

// Apply theme on load
const savedTheme = localStorage.getItem('theme') || 'dark';
if (savedTheme === 'system') {
  const isLight = window.matchMedia('(prefers-color-scheme: light)').matches;
  document.documentElement.setAttribute('data-theme', isLight ? 'light' : 'dark');
} else {
  document.documentElement.setAttribute('data-theme', savedTheme);
}

const app = createApp(App);
app.use(i18n);
app.mount("#app");
