# Flareup Feature Ideas

Features that differentiate Flareup from Raycast, inspired by Alfred and other tools.

## 🎹 Keyboard Maestro-like Macros ⭐ HIGH PRIORITY

Record and replay automation sequences.

**Core Features:**
- Record keyboard (and optionally mouse) actions
- Multiple trigger types: hotkey, typed string, time-based, clipboard, webhook, file watcher
- Action types: type text, key combos, delays, shell commands, open URLs, conditionals, loops
- Variable substitution: `{clipboard}`, `{date}`, `{input}`, `{shell:cmd}`, `{selected_text}`

**MVP Scope:**
1. Record keyboard sequences (no mouse)
2. Hotkey triggers only
3. Basic actions: type text, key combo, delay, shell command
4. Simple variables: `{clipboard}`, `{date}`, `{input}`

**Technical Notes:**
- Use `enigo` or `xdotool` bindings for input simulation
- `evdev` for recording (needs input group permissions)
- Wayland support via `ydotool`

---

## ⏰ Scheduled Actions / Automations

Run extensions or commands on a schedule.

- Run extensions on a timer (e.g., check GitHub PRs every hour)
- Delayed clipboard actions
- Daily digest commands
- Cron-like scheduling UI

---

## 🔗 Webhooks / Remote Triggers

HTTP endpoints that trigger commands remotely.

- `curl localhost:9999/trigger/my-command`
- Integration with n8n, Zapier, IFTTT
- GitHub Actions → Flareup for deployment notifications
- Authentication options for security

---

## 🤖 Headless / Background Extensions

Extensions that run without UI.

- True daemon mode for silent workers
- System tray integrations
- Background file/clipboard watchers
- Event-driven triggers

---

## 📂 File Actions (Contextual Actions)

Powerful file operations like Alfred.

- Right-click → Send to Flareup (file manager integration)
- Batch file operations (rename, convert, compress)
- File filters (find files → act on them)
- Drag-and-drop into Flareup

---

## 🔗 Chained Commands / Pipes

Connect command outputs to inputs.

- Command output → next command input
- Visual workflow builder (drag to connect)
- Conditional branching (if X, do Y)
- Save pipelines as reusable workflows

---

## 🐧 Linux-Native System Integration

DBus and system-level features.

- Systemd service control (start/stop services)
- KDE/GNOME desktop setting toggles
- Bluetooth/WiFi toggles via DBus
- Docker/Podman container management
- Flatpak/Snap integration

---

## ⏱️ Time Tracking Integration

Built-in time tracking.

- Start/stop timers from command palette
- Integrate with Toggl/Clockify APIs
- "What was I working on?" retrospective
- Pomodoro timer mode

---

## 🔥 Extension Hot Reload / Live Development

Smoother extension development than Raycast.

- File watcher with auto-reload
- In-app extension debugging (already have log viewer!)
- Template generator for new extensions
- Extension scaffolding CLI

---

## Priority Matrix

| Feature | User Value | Implementation Effort | Priority |
|---------|-----------|----------------------|----------|
| Macros (Keyboard Maestro) | ⭐⭐⭐⭐⭐ | High | 🔴 High |
| Scheduled Actions | ⭐⭐⭐⭐ | Medium | 🟡 Medium |
| Webhooks | ⭐⭐⭐⭐ | Medium | 🟡 Medium |
| Chained Commands | ⭐⭐⭐⭐ | High | 🟡 Medium |
| File Actions | ⭐⭐⭐ | Medium | 🟢 Low |
| Background Extensions | ⭐⭐⭐ | Medium | 🟢 Low |
| Linux System Integration | ⭐⭐⭐ | Low-Medium | 🟢 Low |
| Time Tracking | ⭐⭐ | Low | 🟢 Low |
| Extension Hot Reload | ⭐⭐ | Low | 🟢 Low |
