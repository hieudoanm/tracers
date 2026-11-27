import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/molecules/Footer';
import Navbar from '@/components/molecules/Navbar';
import './globals.css';

const THEME_INIT_SCRIPT = `(function(){try{var t=localStorage.getItem('tracers-theme');if(!t){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'tracers-dark':'tracers';}document.documentElement.setAttribute('data-theme',t);}catch(e){}})();`;

const inter = Inter({ subsets: ['latin'] });

const GITHUB = 'https://github.com/hieudoanm/tracers';
const BASE = process.env.NODE_ENV === 'production' ? '/tracers' : '';
const LOGO =
  process.env.NODE_ENV === 'production' ? '/tracers/logo.svg' : '/logo.svg';

const NAV_LINKS = [
  { label: 'Features', href: '#features' },
  { label: 'Demo', href: '#demo' },
  { label: 'How it works', href: '#how-it-works' },
  { label: 'Install', href: '#install' },
  { label: 'Docs', href: '/docs/' },
  { label: 'Downloads', href: '/downloads/' },
];

const DESCRIPTION =
  'Tracers parses 20+ languages with Tree-sitter and builds a unified graph of symbols, calls, imports, and dependencies — query callers, callees, impact, and cycles locally, with no cloud.';

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io/tracers/'),
  title: {
    default: 'tracers — code intelligence graphs for any repository',
    template: '%s — tracers',
  },
  description: DESCRIPTION,
  keywords: [
    'code intelligence',
    'code graph',
    'repository analysis',
    'Rust',
    'MCP server',
    'static analysis',
  ],
  authors: [{ name: 'hieudoan', url: 'https://github.com/hieudoanm/tracers' }],
  category: 'software',
  openGraph: {
    siteName: 'tracers',
    type: 'website',
    title: 'tracers — code intelligence graphs for any repository',
    description: DESCRIPTION,
    url: 'https://hieudoanm.github.io/tracers/',
    images: ['/og.png'],
  },
  twitter: { card: 'summary_large_image', images: ['/og.png'] },
  icons: {
    icon: [
      { url: `${BASE}/favicon.ico`, sizes: 'any' },
      { url: `${BASE}/favicon.svg`, type: 'image/svg+xml' },
    ],
  },
};

const JSON_LD = {
  '@context': 'https://schema.org',
  '@graph': [
    {
      '@type': 'WebSite',
      name: 'tracers',
      url: 'https://hieudoanm.github.io/tracers/',
      description:
        'Tracers parses 20+ languages with Tree-sitter and builds a unified graph of symbols, calls, imports, and dependencies — query callers, callees, impact, and cycles locally, with no cloud.',
    },
    {
      '@type': 'SoftwareApplication',
      name: 'tracers',
      applicationCategory: 'DeveloperApplication',
      description:
        'Tracers parses 20+ languages with Tree-sitter and builds a unified graph of symbols, calls, imports, and dependencies — query callers, callees, impact, and cycles locally, with no cloud.',
      url: 'https://hieudoanm.github.io/tracers/',
      codeRepository: 'https://github.com/hieudoanm/tracers',
      offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
    },
  ],
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => (
  <html
    lang="en"
    data-theme="tracers"
    suppressHydrationWarning
    className={`scroll-smooth antialiased motion-reduce:scroll-auto ${inter.className}`}>
    <body className="bg-base-100 text-base-content m-0 leading-relaxed">
      <script dangerouslySetInnerHTML={{ __html: THEME_INIT_SCRIPT }} />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(JSON_LD) }}
      />
      <Navbar
        name="tracers"
        logoSrc={LOGO}
        links={NAV_LINKS}
        githubUrl={GITHUB}
      />
      {children}
      <Footer
        name="tracers"
        tagline="Local code intelligence · MIT licensed"
        githubUrl={GITHUB}
      />
    </body>
  </html>
);

export default RootLayout;
