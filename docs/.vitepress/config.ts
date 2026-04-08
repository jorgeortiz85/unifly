import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Unifly',
  description: 'CLI + TUI for UniFi Network Controllers',
  base: '/unifly/',
  lastUpdated: true,

  head: [
    ['meta', { name: 'theme-color', content: '#e135ff' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Unifly Documentation' }],
    ['meta', { property: 'og:description', content: 'CLI + TUI for UniFi Network Controllers' }],
    ['meta', { property: 'og:site_name', content: 'Unifly' }],
    ['meta', { name: 'twitter:card', content: 'summary' }],
  ],

  themeConfig: {
    nav: [
      { text: 'Guide', link: '/guide/' },
      {
        text: 'Reference',
        items: [
          { text: 'CLI Commands', link: '/reference/cli' },
          { text: 'TUI Dashboard', link: '/reference/tui' },
          { text: 'Library API', link: '/reference/library' },
        ]
      },
      { text: 'Architecture', link: '/architecture/' },
      { text: 'Troubleshooting', link: '/troubleshooting' },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/guide/' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Quick Start', link: '/guide/quick-start' },
            { text: 'Configuration', link: '/guide/configuration' },
            { text: 'Authentication', link: '/guide/authentication' },
            { text: 'AI Agent Skill', link: '/guide/agents' },
          ]
        }
      ],
      '/reference/': [
        {
          text: 'Reference',
          items: [
            { text: 'CLI Commands', link: '/reference/cli' },
            { text: 'TUI Dashboard', link: '/reference/tui' },
            { text: 'Library API', link: '/reference/library' },
          ]
        }
      ],
      '/architecture/': [
        {
          text: 'Architecture',
          items: [
            { text: 'Overview', link: '/architecture/' },
            { text: 'Crate Structure', link: '/architecture/crates' },
            { text: 'Data Flow', link: '/architecture/data-flow' },
            { text: 'API Surface', link: '/architecture/api-surface' },
          ]
        }
      ],
      '/troubleshooting': [
        {
          text: 'Help',
          items: [
            { text: 'Troubleshooting', link: '/troubleshooting' },
          ]
        }
      ],
    },

    editLink: {
      pattern: 'https://github.com/hyperb1iss/unifly/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/hyperb1iss/unifly' },
      { icon: { svg: '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/></svg>' }, link: 'https://github.com/sponsors/hyperb1iss' }
    ],

    footer: {
      message: 'Released under the Apache 2.0 License.',
      copyright: 'Copyright \u00a9 2025 Stefanie Jane'
    },

    search: {
      provider: 'local'
    }
  },

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'one-dark-pro'
    }
  }
})
