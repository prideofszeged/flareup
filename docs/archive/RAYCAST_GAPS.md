# Raycast Parity Tracker

| Rank | Initiative | Importance | Impact | Implementation Effort | Notes |
| ---: | ---------- | ---------- | ------ | --------------------- | ----- |
| 1 | Extension compatibility layer | Critical | High | High | Build shims for macOS APIs (AppleScript, `/Applications`) and curate a Flare-ready store list; unlocks most existing Raycast extensions. |
| 2 | Built-in workflow parity | High | High | Medium | Add window management, reminders, quick toggles, and system monitors to match Raycast's default commands. |
| 3 | First-party service integrations | High | High | High | Ship GitHub/Linear/Jira/Notion connectors with OAuth flows to cover the most used SaaS commands. |
| 4 | UI polish & customization | Medium | Medium | Medium | Theme packs, per-command hotkeys, and icon pipeline bring the app closer to Raycast's polish expectations. |
| 5 | Settings/clipboard sync | Medium | High | High | Provide encrypted sync (Supabase, etc.) for favorites, history, snippets, and quicklinks. |
| 6 | AI workflow templates | Medium | Medium | Low | Layer templated prompts, streaming replies, and code-gen macros on top of the existing OpenRouter bridge. |
| 7 | Search performance tuning | Medium | Medium | Medium | Profile indexing, expand beyond `.desktop` files, and keep fuzzy search under 100ms latency. |
| 8 | Distribution & auto-updates | Medium | Medium | Medium | Auto-update AppImage/Flatpak builds, verify signatures, and streamline install scripts. |
| 9 | Security & permissions UX | Low | Medium | Low | Guided setup for udev rules, portals, and secret storage reduces onboarding friction and support overhead. |
