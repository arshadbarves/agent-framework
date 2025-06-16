# 🚀 AgentGraph Studio UI

**Advanced Visual Debugging Interface for AgentGraph Workflows**

A modern, high-performance Next.js 15 frontend that provides comprehensive visual debugging, monitoring, and analytics for AgentGraph agent workflows. Built with the latest React 19, TypeScript, and Tailwind CSS.

## ✨ Features

### 🎯 **Core Capabilities**
- **Real-time Workflow Visualization** - Interactive graph representation of agent workflows
- **Execution Tracing & Debugging** - Step-by-step execution monitoring with detailed event logs
- **Performance Analytics** - Comprehensive metrics, charts, and performance insights
- **Agent Monitoring** - Individual agent performance tracking and behavior analysis
- **Tool Integration Tracking** - Monitor tool usage, performance, and integration patterns
- **Command Routing Visualization** - See dynamic routing decisions in real-time

### 🔧 **Technical Features**
- **Next.js 15 App Router** - Latest Next.js with React 19 and modern features
- **Real-time Updates** - WebSocket integration for live data streaming
- **Responsive Design** - Works perfectly on desktop, tablet, and mobile
- **Dark/Light Theme** - System-aware theme switching
- **Performance Optimized** - Optimized for speed and efficiency
- **Type-Safe** - Full TypeScript coverage with strict typing

### 🎨 **UI/UX Features**
- **Modern Design System** - Beautiful, consistent design with Radix UI components
- **Interactive Visualizations** - Drag, zoom, and explore workflow graphs
- **Advanced Animations** - Smooth transitions and micro-interactions
- **Accessibility** - WCAG compliant with keyboard navigation
- **Glass Morphism Effects** - Modern visual effects and styling
- **Responsive Grid Layouts** - Adaptive layouts for all screen sizes

## 🏗️ **Architecture**

### **Tech Stack**
- **Framework**: Next.js 15 (App Router)
- **React**: React 19 with latest features
- **TypeScript**: Full type safety
- **Styling**: Tailwind CSS with custom design system
- **UI Components**: Radix UI primitives
- **State Management**: Zustand + React Context
- **Real-time**: Socket.IO client
- **Charts**: Recharts for data visualization
- **Animations**: Framer Motion
- **Icons**: Lucide React

### **Project Structure**
```
agentgraph-studio-ui/
├── app/                    # Next.js 15 App Router
│   ├── layout.tsx         # Root layout with providers
│   ├── page.tsx           # Main dashboard page
│   └── globals.css        # Global styles and design system
├── components/            # React components
│   ├── ui/               # Base UI components (Radix UI)
│   ├── dashboard/        # Dashboard-specific components
│   ├── providers/        # Context providers
│   └── charts/           # Chart components
├── lib/                  # Utilities and configurations
│   ├── types.ts          # TypeScript type definitions
│   ├── utils.ts          # Utility functions
│   └── constants.ts      # App constants
├── hooks/                # Custom React hooks
├── styles/               # Additional styles
└── public/               # Static assets
```

## 🚀 **Getting Started**

### **Prerequisites**
- Node.js 18+ 
- npm, yarn, or pnpm
- AgentGraph backend running on `localhost:8080`

### **Installation**

1. **Install dependencies**:
```bash
npm install
# or
yarn install
# or
pnpm install
```

2. **Set up environment variables**:
```bash
cp .env.example .env.local
```

Edit `.env.local`:
```env
AGENTGRAPH_API_URL=http://localhost:8080
AGENTGRAPH_WS_URL=ws://localhost:8080
NEXT_PUBLIC_APP_URL=http://localhost:3000
```

3. **Start the development server**:
```bash
npm run dev
# or
yarn dev
# or
pnpm dev
```

4. **Open your browser**:
Navigate to [http://localhost:3000](http://localhost:3000)

### **Production Build**

```bash
npm run build
npm run start
```

## 📊 **Dashboard Features**

### **Overview Tab**
- System metrics overview with real-time updates
- Quick stats cards (executions, success rate, performance)
- Recent activity feed
- Connection status monitoring

### **Workflows Tab**
- Interactive workflow visualization
- Node status indicators (running, completed, failed, pending)
- Drag and zoom capabilities
- Multiple layout algorithms (force-directed, hierarchical, circular, grid)
- Real-time execution progress

### **Traces Tab**
- Detailed execution traces with event timelines
- Step-by-step debugging information
- Agent response tracking
- Tool execution monitoring
- Command routing visualization

### **Agents Tab**
- Individual agent performance metrics
- Token usage and cost tracking
- Response time analytics
- Success rate monitoring
- Activity timelines

### **Performance Tab**
- System resource monitoring (CPU, memory, network)
- Execution time trends
- Throughput analytics
- Performance bottleneck identification
- Historical performance data

### **Events Tab**
- Real-time event stream
- Event filtering and search
- Event details and context
- Export capabilities

## 🎨 **Design System**

### **Color Palette**
- **Primary**: Blue gradient (`#3B82F6` to `#1D4ED8`)
- **Success**: Green (`#10B981`)
- **Warning**: Orange (`#F59E0B`)
- **Error**: Red (`#EF4444`)
- **Node Types**: 
  - Agent: Green gradient
  - Tool: Blue gradient
  - Routing: Orange gradient
  - Quality Gate: Purple gradient

### **Typography**
- **Primary Font**: Inter (system font)
- **Monospace**: JetBrains Mono (code and metrics)
- **Sizes**: Responsive scale from 12px to 48px

### **Components**
- **Cards**: Glass morphism with subtle shadows
- **Buttons**: Multiple variants with hover states
- **Badges**: Status indicators with color coding
- **Charts**: Interactive with hover tooltips
- **Animations**: Smooth transitions and micro-interactions

## 🔌 **API Integration**

### **REST Endpoints**
```typescript
GET /api/agentgraph/metrics      // System metrics
GET /api/agentgraph/workflows    // Workflow definitions
GET /api/agentgraph/traces       // Execution traces
GET /api/agentgraph/agents       // Agent information
GET /api/agentgraph/tools        // Tool metrics
```

### **WebSocket Events**
```typescript
// Real-time event types
'event'     // Execution events
'metrics'   // Metrics updates
'workflow'  // Workflow changes
'trace'     // Trace updates
'error'     // Error notifications
```

## 🧪 **Development**

### **Code Quality**
```bash
npm run lint          # ESLint
npm run type-check    # TypeScript checking
npm run format        # Prettier formatting
```

### **Testing**
```bash
npm run test          # Jest tests
npm run test:watch    # Watch mode
npm run test:coverage # Coverage report
```

### **Performance**
- **Bundle Analysis**: `npm run analyze`
- **Lighthouse**: Built-in performance monitoring
- **Core Web Vitals**: Optimized for excellent scores

## 🌐 **Deployment**

### **Vercel (Recommended)**
```bash
npm install -g vercel
vercel
```

### **Docker**
```bash
docker build -t agentgraph-studio-ui .
docker run -p 3000:3000 agentgraph-studio-ui
```

### **Static Export**
```bash
npm run build
npm run export
```

## 🔧 **Configuration**

### **Theme Customization**
Edit `tailwind.config.ts` to customize colors, fonts, and spacing.

### **Component Customization**
All UI components are in `components/ui/` and can be customized.

### **API Configuration**
Update `lib/utils.ts` to modify API endpoints and WebSocket URLs.

## 📈 **Performance**

### **Optimizations**
- **Code Splitting**: Automatic route-based splitting
- **Image Optimization**: Next.js Image component
- **Font Optimization**: Google Fonts optimization
- **Bundle Size**: Optimized with tree shaking
- **Caching**: Aggressive caching strategies

### **Metrics**
- **First Contentful Paint**: < 1.5s
- **Largest Contentful Paint**: < 2.5s
- **Cumulative Layout Shift**: < 0.1
- **First Input Delay**: < 100ms

## 🤝 **Contributing**

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 **Support**

- **Documentation**: [docs.agentgraph.dev](https://docs.agentgraph.dev)
- **Issues**: [GitHub Issues](https://github.com/agentgraph/studio-ui/issues)
- **Discussions**: [GitHub Discussions](https://github.com/agentgraph/studio-ui/discussions)
- **Discord**: [AgentGraph Community](https://discord.gg/agentgraph)

---

**Built with ❤️ by the AgentGraph Team**
