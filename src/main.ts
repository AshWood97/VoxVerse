import { createApp } from "vue";
import App from "./App.vue";
import { i18n } from "./i18n";
import { applyThemePreference } from "./theme";
import "./assets/styles/global.css";

// Apply and migrate persisted theme preference on load.
const normalizedTheme = applyThemePreference(localStorage.getItem('theme'));
localStorage.setItem('theme', normalizedTheme);

const app = createApp(App);
app.use(i18n);
app.mount("#app");
