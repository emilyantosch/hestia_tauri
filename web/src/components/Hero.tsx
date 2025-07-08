import { ArrowRight, FileText, Search, Database, Zap, Shield, Users } from 'lucide-react'
import { cn } from '../lib/utils'

export default function Hero() {
  return (
    <section className="relative min-h-screen bg-black flex items-center justify-center pt-16">
      {/* Subtle gradient overlay */}
      <div className="absolute inset-0 bg-gradient-to-b from-black via-gray-900/50 to-black" />
      
      {/* Content */}
      <div className="relative z-10 text-center px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div className="max-w-4xl mx-auto">
          {/* Main Headline */}
          <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-bold text-white mb-6 leading-tight">
            Scale your
            <br />
            <span className="bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
              file management
            </span>
            <br />
            velocity performance
          </h1>
          
          {/* Subheading */}
          <p className="text-xl sm:text-2xl text-gray-400 mb-12 max-w-3xl mx-auto leading-relaxed">
            Transform your file organization workflow with intelligent automation, 
            lightning-fast search, and seamless collaboration tools.
          </p>

          {/* Primary CTA */}
          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-20">
            <button className={cn(
              "group relative px-8 py-4 bg-white text-black font-semibold rounded-md",
              "hover:bg-gray-100 transition-all duration-300 flex items-center gap-3"
            )}>
              Get a demo
              <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
            </button>
            
            <button className={cn(
              "group px-8 py-4 bg-transparent text-white font-semibold rounded-md border border-gray-600",
              "hover:bg-gray-900 transition-all duration-300"
            )}>
              Sign In
            </button>
          </div>

          {/* Feature Grid */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 max-w-6xl mx-auto">
            {/* Feature 1 */}
            <div className="text-left">
              <div className="w-12 h-12 bg-gradient-to-r from-blue-500 to-purple-500 rounded-lg flex items-center justify-center mb-4">
                <Search className="w-6 h-6 text-white" />
              </div>
              <h3 className="text-xl font-semibold text-white mb-3">Intelligent Search</h3>
              <p className="text-gray-400 leading-relaxed">
                Find any file instantly with AI-powered search that understands content, 
                context, and relationships between your documents.
              </p>
            </div>
            
            {/* Feature 2 */}
            <div className="text-left">
              <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mb-4">
                <Database className="w-6 h-6 text-white" />
              </div>
              <h3 className="text-xl font-semibold text-white mb-3">Smart Organization</h3>
              <p className="text-gray-400 leading-relaxed">
                Automatically categorize and organize files based on content, 
                usage patterns, and your personal workflow preferences.
              </p>
            </div>
            
            {/* Feature 3 */}
            <div className="text-left">
              <div className="w-12 h-12 bg-gradient-to-r from-green-500 to-blue-500 rounded-lg flex items-center justify-center mb-4">
                <Users className="w-6 h-6 text-white" />
              </div>
              <h3 className="text-xl font-semibold text-white mb-3">Team Collaboration</h3>
              <p className="text-gray-400 leading-relaxed">
                Share, collaborate, and manage permissions seamlessly across 
                teams with real-time sync and version control.
              </p>
            </div>
          </div>

          {/* Secondary features */}
          <div className="mt-16 grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
            <div className="flex items-center gap-3 text-gray-400">
              <Zap className="w-5 h-5 text-blue-400" />
              <span className="text-sm">Lightning fast performance</span>
            </div>
            <div className="flex items-center gap-3 text-gray-400">
              <Shield className="w-5 h-5 text-green-400" />
              <span className="text-sm">Enterprise-grade security</span>
            </div>
            <div className="flex items-center gap-3 text-gray-400">
              <FileText className="w-5 h-5 text-purple-400" />
              <span className="text-sm">All file types supported</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}