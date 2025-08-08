import './App.css'
import Navigation from './components/Navigation'
import Hero from './components/Hero'
import Testimonials from './components/Testimonials'

function App() {
  return (
    <div className="min-h-screen bg-black">
      <Navigation />
      <Hero />
      <Testimonials />
    </div>
  )
}

export default App
