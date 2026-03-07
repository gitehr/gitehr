// @ts-check
// SPDX-License-Identifier: CC-BY-SA-4.0

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  mainSidebar: [
    {
      type: 'doc',
      id: 'getting-started',
      label: 'Getting Started',
    },
    {
      type: 'category',
      label: 'About',
      items: [
        'about/the-gitehr-story',
        'about/databasechester',
        'about/contributors',
        'about/foundation',
      ],
    },
    {
      type: 'category',
      label: 'Design',
      items: [
        'design/design',
        'design/patient-centricity',
        'design/structure',
        'design/audit-trail',
        'design/provenance',
        'design/forensics',
        'design/security',
        'design/redundancy',
        'design/portability',
        'design/no-lock-in',
        'design/longevity',
        'design/simplicity',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/gui-walkthrough',
        'guides/repository-structure',
      ],
    },
    {
      type: 'category',
      label: 'Developers',
      items: [
        'developers/developers',
        'developers/cli',
      ],
    },
    {
      type: 'category',
      label: 'More',
      items: [
        'glossary',
        'references/references',
        'mcp-usage',
      ],
    },
  ],
};

export default sidebars;
