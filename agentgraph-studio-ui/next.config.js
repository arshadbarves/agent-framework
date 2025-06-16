/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    // Enable the latest Next.js features
    turbo: {
      rules: {
        '*.svg': {
          loaders: ['@svgr/webpack'],
          as: '*.js',
        },
      },
    },
  },
  // Enable React 19 features
  reactStrictMode: true,
  
  // Performance optimizations
  swcMinify: true,
  
  // Image optimization
  images: {
    domains: ['localhost', '127.0.0.1'],
    formats: ['image/webp', 'image/avif'],
  },
  
  // API configuration for backend communication
  async rewrites() {
    return [
      {
        source: '/api/agentgraph/:path*',
        destination: 'http://localhost:8080/api/:path*',
      },
    ];
  },
  
  // Headers for WebSocket and CORS
  async headers() {
    return [
      {
        source: '/api/:path*',
        headers: [
          { key: 'Access-Control-Allow-Origin', value: '*' },
          { key: 'Access-Control-Allow-Methods', value: 'GET, POST, PUT, DELETE, OPTIONS' },
          { key: 'Access-Control-Allow-Headers', value: 'Content-Type, Authorization' },
        ],
      },
    ];
  },
  
  // Webpack configuration for custom optimizations
  webpack: (config, { buildId, dev, isServer, defaultLoaders, webpack }) => {
    // Add custom webpack configurations if needed
    config.module.rules.push({
      test: /\.svg$/,
      use: ['@svgr/webpack'],
    });
    
    return config;
  },
  
  // Environment variables
  env: {
    AGENTGRAPH_API_URL: process.env.AGENTGRAPH_API_URL || 'http://localhost:8080',
    AGENTGRAPH_WS_URL: process.env.AGENTGRAPH_WS_URL || 'ws://localhost:8080',
  },
};

module.exports = nextConfig;
