// @ts-check
// SPDX-License-Identifier: CC-BY-SA-4.0

import { themes as prismThemes } from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'GitEHR',
  tagline: 'Decentralised, lossless health records',
  favicon: 'img/gitehr-logo.svg',

  url: 'https://gitehr.org',
  baseUrl: '/',

  organizationName: 'gitehr',
  projectName: 'gitehr',

  onBrokenLinks: 'warn',

  future: {
    v4: true,
  },

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/gitehr/gitehr/edit/main/docs/',
          routeBasePath: '/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      image: 'img/gitehr-logo.svg',
      navbar: {
        title: 'GitEHR',
        logo: {
          alt: 'GitEHR Logo',
          src: 'img/gitehr-logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'mainSidebar',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/gitehr/gitehr',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              { label: 'Getting Started', to: '/getting-started' },
              { label: 'Design', to: '/design/' },
              { label: 'Developer Guide', to: '/developers/' },
            ],
          },
          {
            title: 'More',
            items: [
              { label: 'Glossary', to: '/glossary' },
              { label: 'References', to: '/references/' },
              {
                label: 'GitHub',
                href: 'https://github.com/gitehr/gitehr',
              },
            ],
          },
        ],
        copyright: `GitEHR is copyright &copy; 2022-25 Baw Medical Ltd, licensed under <a rel="license" href="https://creativecommons.org/licenses/by-sa/4.0/">CC-BY-SA 4.0</a> (GitEHR concepts and data) or <a rel="license" href="https://www.gnu.org/licenses/agpl-3.0.en.html">AGPL-3.0</a> (GitEHR code).`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
      },
      colorMode: {
        defaultMode: 'light',
        respectPrefersColorScheme: true,
      },
    }),
};

export default config;
