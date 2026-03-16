/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{svelte,ts}"],
  theme: {
    extend: {
      colors: {
        base: "var(--bg-base)",
        panel: "var(--bg-panel)",
        surface: "var(--bg-surface)",
        elevated: "var(--bg-elevated)",
        border: "var(--border)",
        "border-subtle": "var(--border-subtle)",
        "border-focus": "var(--border-focus)",
        text: {
          primary: "var(--text-primary)",
          secondary: "var(--text-secondary)",
          muted: "var(--text-muted)",
        },
        accent: "var(--accent)",
        "accent-bright": "var(--accent-bright)",
        "accent-subtle": "var(--accent-subtle)",
        "accent-glow": "var(--accent-glow)",
        accent2: "var(--accent2)",
        "accent2-subtle": "var(--accent2-subtle)",
        success: "var(--success)",
        warning: "var(--warning)",
        danger: "var(--danger)",
      },
      fontFamily: {
        ui: "var(--font-ui)",
        editor: "var(--font-editor)",
        mono: "var(--font-mono)",
      },
      borderRadius: {
        sm: "var(--radius-sm)",
        md: "var(--radius-md)",
        lg: "var(--radius-lg)",
      },
      transitionDuration: {
        base: "140ms",
      },
    },
  },
};
