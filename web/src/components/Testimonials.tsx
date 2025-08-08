import { Star } from 'lucide-react'

export default function Testimonials() {
  const testimonials = [
    {
      name: "Sarah Chen",
      role: "Product Manager",
      company: "TechCorp",
      content: "Hestia completely transformed how our team manages files. The AI-powered search finds exactly what I need in seconds, and the collaboration features have streamlined our workflow significantly.",
      rating: 5,
      avatar: "https://images.unsplash.com/photo-1494790108755-2616b25c5e98?w=64&h=64&fit=crop&crop=face&auto=format&q=80"
    },
    {
      name: "Michael Rodriguez",
      role: "Creative Director",
      company: "DesignStudio",
      content: "The smart organization feature is a game-changer. Files are automatically categorized perfectly, and I can find any project asset instantly. It's like having a personal assistant for file management.",
      rating: 5,
      avatar: "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?w=64&h=64&fit=crop&crop=face&auto=format&q=80"
    },
    {
      name: "Emily Johnson",
      role: "Operations Lead",
      company: "StartupXYZ",
      content: "Since implementing Hestia, our team productivity has increased by 40%. The real-time sync and version control features ensure everyone stays aligned without conflicts.",
      rating: 5,
      avatar: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=64&h=64&fit=crop&crop=face&auto=format&q=80"
    }
  ]

  return (
    <section id="customers" className="py-20 bg-gray-900">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="text-center mb-16">
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold text-white mb-4">
            Trusted by teams worldwide
          </h2>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto">
            Join thousands of professionals who have transformed their file management workflow with Hestia
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {testimonials.map((testimonial, index) => (
            <div key={index} className="bg-black/50 rounded-lg p-6 border border-gray-800">
              {/* Rating */}
              <div className="flex items-center mb-4">
                {[...Array(testimonial.rating)].map((_, i) => (
                  <Star key={i} className="w-5 h-5 text-yellow-400 fill-current" />
                ))}
              </div>

              {/* Content */}
              <blockquote className="text-gray-300 mb-6 leading-relaxed">
                "{testimonial.content}"
              </blockquote>

              {/* Author */}
              <div className="flex items-center">
                <img
                  src={testimonial.avatar}
                  alt={testimonial.name}
                  className="w-12 h-12 rounded-full mr-4"
                />
                <div>
                  <div className="font-semibold text-white">{testimonial.name}</div>
                  <div className="text-sm text-gray-400">
                    {testimonial.role} at {testimonial.company}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* CTA */}
        <div className="text-center mt-16">
          <button className="bg-white text-black px-8 py-4 font-semibold rounded-md hover:bg-gray-100 transition-colors">
            Join them today
          </button>
        </div>
      </div>
    </section>
  )
}