import { Menu, X } from 'lucide-react'
import { useState } from 'react'
import { cn } from '../lib/utils'

export default function Navigation() {
  const [isOpen, setIsOpen] = useState(false)

  const navigationItems = [
    { name: 'Features', href: '#features' },
    { name: 'Customers', href: '#customers' },
    { name: 'Pricing', href: '#pricing' },
  ]

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-black/80 backdrop-blur-md border-b border-gray-800">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <div className="flex items-center">
            <div className="text-2xl font-bold text-white">
              Hestia
            </div>
          </div>

          {/* Desktop Navigation */}
          <div className="hidden md:block">
            <div className="ml-10 flex items-baseline space-x-8">
              {navigationItems.map((item) => (
                <a
                  key={item.name}
                  href={item.href}
                  className="text-gray-300 hover:text-white px-3 py-2 text-sm font-medium transition-colors"
                >
                  {item.name}
                </a>
              ))}
            </div>
          </div>

          {/* CTA Buttons */}
          <div className="hidden md:flex items-center space-x-4">
            <button className="text-gray-300 hover:text-white px-4 py-2 text-sm font-medium transition-colors">
              Sign In
            </button>
            <button className="bg-white text-black px-4 py-2 text-sm font-medium rounded-md hover:bg-gray-100 transition-colors">
              Get a demo
            </button>
          </div>

          {/* Mobile menu button */}
          <div className="md:hidden">
            <button
              onClick={() => setIsOpen(!isOpen)}
              className="text-gray-300 hover:text-white p-2"
            >
              {isOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Navigation */}
      {isOpen && (
        <div className="md:hidden">
          <div className="px-2 pt-2 pb-3 space-y-1 sm:px-3 bg-black/90 backdrop-blur-md">
            {navigationItems.map((item) => (
              <a
                key={item.name}
                href={item.href}
                className="text-gray-300 hover:text-white block px-3 py-2 text-base font-medium"
                onClick={() => setIsOpen(false)}
              >
                {item.name}
              </a>
            ))}
            <div className="pt-4 border-t border-gray-700">
              <button className="text-gray-300 hover:text-white block px-3 py-2 text-base font-medium w-full text-left">
                Sign In
              </button>
              <button className="bg-white text-black px-3 py-2 text-base font-medium rounded-md hover:bg-gray-100 transition-colors mx-3 mt-2">
                Get a demo
              </button>
            </div>
          </div>
        </div>
      )}
    </nav>
  )
}