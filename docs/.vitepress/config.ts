import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "Sikuli-D Documentation",
  description: "GUI Automation Tool with Image Recognition",

  // Base configuration
  base: '/Sikuli-D/',
  lang: 'en-US',

  // Theme configuration
  appearance: 'dark',

  // Head configuration
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }]
  ],

  // Locales for bilingual support
  locales: {
    root: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/' },
          { text: 'Getting Started', link: '/getting-started/' },
          { text: 'API Reference', link: '/api/' },
          { text: 'Tutorials', link: '/tutorials/' }
        ],

        sidebar: [
          {
            text: 'Getting Started',
            collapsed: false,
            items: [
              { text: 'Introduction', link: '/getting-started/' },
              { text: 'Installation', link: '/getting-started/installation' },
              { text: 'Quick Start', link: '/getting-started/quick-start' }
            ]
          },
          {
            text: 'API Reference',
            collapsed: false,
            items: [
              { text: 'Overview', link: '/api/' },
              { text: 'Screen API', link: '/api/screen' },
              { text: 'Region API', link: '/api/region' },
              { text: 'Pattern API', link: '/api/pattern' },
              { text: 'Match API', link: '/api/match' }
            ]
          },
          {
            text: 'Tutorials',
            collapsed: false,
            items: [
              { text: 'Basic Examples', link: '/tutorials/' },
              { text: 'Image Recognition', link: '/tutorials/image-recognition' },
              { text: 'OCR Text Reading', link: '/tutorials/ocr' }
            ]
          },
          {
            text: 'Troubleshooting',
            collapsed: false,
            items: [
              { text: 'Common Issues', link: '/troubleshooting/' },
              { text: 'FAQ', link: '/troubleshooting/faq' }
            ]
          }
        ],

        socialLinks: [
          { icon: 'github', link: 'https://github.com/daitamu/Sikuli-D' }
        ],

        footer: {
          message: 'Released under the Apache License 2.0',
          copyright: 'Copyright © 2025 Sikuli-D Team'
        }
      }
    },

    ja: {
      label: '日本語',
      lang: 'ja',
      themeConfig: {
        nav: [
          { text: 'ホーム', link: '/ja/' },
          { text: '始め方', link: '/ja/getting-started/' },
          { text: 'API リファレンス', link: '/ja/api/' },
          { text: 'チュートリアル', link: '/ja/tutorials/' }
        ],

        sidebar: [
          {
            text: '始め方',
            collapsed: false,
            items: [
              { text: '概要', link: '/ja/getting-started/' },
              { text: 'インストール', link: '/ja/getting-started/installation' },
              { text: 'クイックスタート', link: '/ja/getting-started/quick-start' }
            ]
          },
          {
            text: 'API リファレンス',
            collapsed: false,
            items: [
              { text: '概要', link: '/ja/api/' },
              { text: 'Screen API', link: '/ja/api/screen' },
              { text: 'Region API', link: '/ja/api/region' },
              { text: 'Pattern API', link: '/ja/api/pattern' },
              { text: 'Match API', link: '/ja/api/match' }
            ]
          },
          {
            text: 'チュートリアル',
            collapsed: false,
            items: [
              { text: '基本例', link: '/ja/tutorials/' },
              { text: '画像認識', link: '/ja/tutorials/image-recognition' },
              { text: 'OCR テキスト読み取り', link: '/ja/tutorials/ocr' }
            ]
          },
          {
            text: 'トラブルシューティング',
            collapsed: false,
            items: [
              { text: 'よくある問題', link: '/ja/troubleshooting/' },
              { text: 'FAQ', link: '/ja/troubleshooting/faq' }
            ]
          }
        ],

        socialLinks: [
          { icon: 'github', link: 'https://github.com/daitamu/Sikuli-D' }
        ],

        footer: {
          message: 'Apache License 2.0 でリリース',
          copyright: 'Copyright © 2025 Sikuli-D チーム'
        },

        docFooter: {
          prev: '前のページ',
          next: '次のページ'
        },

        outlineTitle: '目次',
        returnToTopLabel: 'トップへ戻る',
        sidebarMenuLabel: 'メニュー',
        darkModeSwitchLabel: 'ダークモード'
      }
    }
  },

  // Markdown configuration
  markdown: {
    lineNumbers: true
  },

  // Theme configuration
  themeConfig: {
    search: {
      provider: 'local'
    }
  }
})
