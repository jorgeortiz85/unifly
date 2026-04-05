---
layout: home

hero:
  name: Unifly
  text: Your UniFi Network, at Your Fingertips
  tagline: "26 commands, 10 TUI screens, dual-API engine. Manage your entire UniFi network from the terminal."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/
    - theme: alt
      text: View on GitHub
      link: https://github.com/hyperb1iss/unifly

features:
  - icon: "\u26A1"
    title: Dual API Engine
    details: Speaks both Integration API (REST, API key) and Legacy API (session, cookie/CSRF). You never think about which endpoint to hit.
  - icon: "\uD83D\uDCCA"
    title: 10-Screen TUI
    details: btop-inspired dashboard with Braille traffic charts, CPU/MEM bars, zoomable topology, and live event streaming.
  - icon: "\uD83E\uDDE0"
    title: 26 Top-Level Commands
    details: Devices, clients, networks, WiFi, firewall, NAT, DNS, DPI, topology, raw API passthrough, and more.
  - icon: "\uD83D\uDD12"
    title: Secure Credentials
    details: OS keyring by default for API keys and passwords. Plaintext config available as an opt-in fallback.
  - icon: "\uD83C\uDF10"
    title: Multi-Profile
    details: Named profiles for multiple controllers with instant switching. Great for managing home, office, and remote sites.
  - icon: "\uD83E\uDD16"
    title: AI Agent Skill
    details: Ships with a skill bundle that teaches coding agents to provision VLANs, audit firewalls, and diagnose connectivity.
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: linear-gradient(135deg, #e135ff 0%, #80ffea 100%);
}

.dark {
  --vp-home-hero-image-background-image: linear-gradient(135deg, rgba(225, 53, 255, 0.2) 0%, rgba(128, 255, 234, 0.2) 100%);
  --vp-home-hero-image-filter: blur(56px);
}
</style>
