import type { Metadata } from 'next'
import { Inter, JetBrains_Mono } from 'next/font/google'
import './globals.css'
import { ThemeProvider } from '@/components/providers/theme-provider'
import { ToastProvider } from '@/components/providers/toast-provider'
import { AgentGraphProvider } from '@/components/providers/agentgraph-provider'
import { cn } from '@/lib/utils'

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-sans',
  display: 'swap',
})

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
  display: 'swap',
})

export const metadata: Metadata = {
  title: {
    default: 'AgentGraph Studio',
    template: '%s | AgentGraph Studio',
  },
  description: 'Advanced visual debugging and monitoring interface for AgentGraph workflows. Real-time execution tracing, performance analytics, and workflow visualization.',
  keywords: [
    'AgentGraph',
    'AI Agents',
    'Workflow Debugging',
    'Visual Interface',
    'Performance Monitoring',
    'Real-time Analytics',
    'Agent Orchestration',
    'LangGraph Alternative',
  ],
  authors: [
    {
      name: 'AgentGraph Team',
      url: 'https://agentgraph.dev',
    },
  ],
  creator: 'AgentGraph',
  openGraph: {
    type: 'website',
    locale: 'en_US',
    url: 'https://studio.agentgraph.dev',
    title: 'AgentGraph Studio',
    description: 'Advanced visual debugging and monitoring interface for AgentGraph workflows',
    siteName: 'AgentGraph Studio',
    images: [
      {
        url: '/og-image.png',
        width: 1200,
        height: 630,
        alt: 'AgentGraph Studio - Visual Debugging Interface',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'AgentGraph Studio',
    description: 'Advanced visual debugging and monitoring interface for AgentGraph workflows',
    images: ['/og-image.png'],
    creator: '@agentgraph',
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  manifest: '/manifest.json',
  icons: {
    icon: '/favicon.ico',
    shortcut: '/favicon-16x16.png',
    apple: '/apple-touch-icon.png',
  },
  viewport: {
    width: 'device-width',
    initialScale: 1,
    maximumScale: 1,
  },
}

interface RootLayoutProps {
  children: React.ReactNode
}

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <meta name="theme-color" content="#3B82F6" />
        <meta name="color-scheme" content="light dark" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
      </head>
      <body
        className={cn(
          'min-h-screen bg-background font-sans antialiased',
          inter.variable,
          jetbrainsMono.variable
        )}
      >
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          <AgentGraphProvider>
            <div className="relative flex min-h-screen flex-col">
              <div className="flex-1">
                {children}
              </div>
            </div>
            <ToastProvider />
          </AgentGraphProvider>
        </ThemeProvider>
      </body>
    </html>
  )
}
